#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_variables
)]

use crate::htslib_rs::{
    hfile::{
        hFILE_plugin, hFILE_scheme_handler, hclose_abruptly, hfile_add_scheme_handler,
        hfile_c_1317_hopen_vargs, htslib_hfile_h_247_hread,
    },
    hfile::HFileBackend,
    hts::{hFILE, hts_verbose, kstring_t},
};
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void, CStr, CString};
use std::ptr::NonNull;

const AUTH_LIFETIME: libc::time_t = 60;
const CREDENTIAL_LIFETIME: libc::time_t = 60;
const DIGEST_BUFSIZ: usize = 64;
const SHA256_DIGEST_BUFSIZE: usize = 32;
const HASH_LENGTH_SHA256: usize = SHA256_DIGEST_BUFSIZE * 2 + 1;
const MINIMUM_S3_WRITE_SIZE: c_int = 5_242_880;
const EXPAND_ON: c_int = 1112;
const S3_MOVED_PERMANENTLY: libc::c_long = 301;

trait NullablePtrExt<T> {
    fn as_ptr(&self) -> *mut T;
    fn is_null(&self) -> bool;
}

impl<T> NullablePtrExt<T> for Option<NonNull<T>> {
    #[inline]
    fn as_ptr(&self) -> *mut T {
        self.map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }

    #[inline]
    fn is_null(&self) -> bool {
        self.is_none()
    }
}

fn write_s3_date_header(buf: &mut [c_char], now: libc::time_t) {
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (year, month, day, hour, minute, second, weekday) =
        crate::htslib_rs::c_compat::unix_time_utc_parts(now);
    let text = format!(
        "Date: {}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
        WEEKDAYS[weekday],
        day,
        MONTHS[(month - 1) as usize],
        year,
        hour,
        minute,
        second
    );
    crate::htslib_rs::c_compat::write_c_str(buf, &text);
}

fn write_s3_v4_dates(
    date_long: &mut [c_char],
    date_short: &mut [c_char],
    now: libc::time_t,
) -> bool {
    let (year, month, day, hour, minute, second, _) =
        crate::htslib_rs::c_compat::unix_time_utc_parts(now);
    let long = format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        year, month, day, hour, minute, second
    );
    let short = format!("{:04}{:02}{:02}", year, month, day);
    crate::htslib_rs::c_compat::write_c_str(date_long, &long) == 16
        && crate::htslib_rs::c_compat::write_c_str(date_short, &short) == 8
}
const S3_TEMPORARY_REDIRECT: libc::c_long = 307;
const S3_BAD_REQUEST: libc::c_long = 400;
const HTS_LOG_INFO: c_int = 4;

const CURLE_OK: c_int = 0;
const CURLINFO_RESPONSE_CODE: c_int = 0x200000 + 2;
const CURLOPT_WRITEDATA: c_int = 10_001;
const CURLOPT_URL: c_int = 10_002;
const CURLOPT_READDATA: c_int = 10_009;
const CURLOPT_POSTFIELDS: c_int = 10_015;
const CURLOPT_WRITEFUNCTION: c_int = 20_011;
const CURLOPT_READFUNCTION: c_int = 20_012;
const CURLOPT_USERAGENT: c_int = 10_018;
const CURLOPT_HTTPHEADER: c_int = 10_023;
const CURLOPT_HEADERDATA: c_int = 10_029;
const CURLOPT_CUSTOMREQUEST: c_int = 10_036;
const CURLOPT_VERBOSE: c_int = 41;
const CURLOPT_UPLOAD: c_int = 46;
const CURLOPT_POST: c_int = 47;
const CURLOPT_POSTFIELDSIZE: c_int = 60;
const CURLOPT_HEADERFUNCTION: c_int = 20_079;
const CURLOPT_INFILESIZE_LARGE: c_int = 30_115;

type HFileOpenFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut hFILE;
type HFileIsRemoteFn = unsafe extern "C" fn(*const c_char) -> c_int;
type HFileVOpenFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE;
type HFilePluginDestroyFn = unsafe extern "C" fn();

// The old `#[repr(C)] struct hFILE_backend` vtable (read/write/seek/flush/close
// fn pointers) and the `HFileLayout` C base struct are gone: the five hFILE
// operations are now methods on `HFileBackend` (see hfile.rs) and the buffer
// state lives directly in the owned `hFILE` (see hts.rs).

#[repr(C)]
struct hFILE_scheme_handler_layout {
    open: Option<HFileOpenFn>,
    isremote: Option<HFileIsRemoteFn>,
    provider: *const c_char,
    priority: c_int,
    vopen: Option<HFileVOpenFn>,
}

unsafe impl Sync for hFILE_scheme_handler_layout {}

#[repr(C)]
struct hFILE_plugin_layout {
    api_version: c_int,
    obj: Option<NonNull<c_void>>,
    name: *const c_char,
    destroy: Option<HFilePluginDestroyFn>,
}

#[repr(C)]
struct HFileLibcurlCurlSlist {
    data: *mut c_char,
    next: *mut HFileLibcurlCurlSlist,
}

// hFILE_s3 no longer embeds `base: HFileLayout`; it is now the payload of the
// `HFileBackend::S3(Box<hFILE_s3>)` enum variant. The owning `hFILE` carries the
// buffer/begin/end/limit/offset/flags state directly (see hts.rs).
pub struct hFILE_s3 {
    curl: Option<NonNull<c_void>>,
    ret: c_int,
    au: Option<NonNull<S3AuthDataLayout>>,
    buffer: kstring_t,
    url: kstring_t,
    verbose: libc::c_long,
    write: c_int,
    part_size: c_int,
    content_hash: kstring_t,
    authorisation: kstring_t,
    content: kstring_t,
    date: kstring_t,
    token: kstring_t,
    range: kstring_t,
    upload_id: kstring_t,
    completion_message: kstring_t,
    part_no: c_int,
    aborted: c_int,
    index: usize,
    expand: c_int,
    last_read: usize,
    last_read_buffer: usize,
    file_size: i64,
    keep_going: c_int,
}

impl Default for hFILE_s3 {
    fn default() -> Self {
        Self {
            curl: None,
            ret: 0,
            au: None,
            buffer: kstring_t::default(),
            url: kstring_t::default(),
            verbose: 0,
            write: 0,
            part_size: 0,
            content_hash: kstring_t::default(),
            authorisation: kstring_t::default(),
            content: kstring_t::default(),
            date: kstring_t::default(),
            token: kstring_t::default(),
            range: kstring_t::default(),
            upload_id: kstring_t::default(),
            completion_message: kstring_t::default(),
            part_no: 0,
            aborted: 0,
            index: 0,
            expand: 0,
            last_read: 0,
            last_read_buffer: 0,
            file_size: 0,
            keep_going: 0,
        }
    }
}

// Concurrency note (audit 2026-05):
//
// `HFILE_S3_USERAGENT` is a `static mut kstring_t` that mirrors the file-scope
// `useragent` in `htslib/hfile_s3.c`. It is initialized exactly once inside
// `hfile_s3_c_2436_PLUGIN_GLOBAL`, which is dispatched from
// `hfile_c_1111_load_hfile_plugins` under the `hfile_plugin_state` mutex
// (see `src/hfile.rs`). After init the owned `data` Vec is not mutated
// (only its bytes are read, NUL-terminated into a temp for `CURLOPT_USERAGENT`)
// until `hfile_s3_c_2426_s3_exit` runs as the plugin destroy callback at
// process shutdown.
//
// SAFETY: init-once-then-read, protected by the plugin-load mutex. Read
// sites only copy `HFILE_S3_USERAGENT.data` into a temporary C string and
// hand that to libcurl, which performs its own internal synchronization for
// per-easy-handle option storage.
static mut HFILE_S3_USERAGENT: kstring_t = kstring_t { data: Vec::new() };

#[link(name = "curl")]
unsafe extern "C" {
    fn curl_easy_cleanup(curl: *mut c_void);
    fn curl_easy_init() -> *mut c_void;
    fn curl_easy_reset(curl: *mut c_void);
    fn curl_easy_perform(curl: *mut c_void) -> c_int;
    fn curl_easy_setopt(curl: *mut c_void, option: c_int, ...) -> c_int;
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_long(curl: *mut c_void, info: c_int, value: *mut libc::c_long) -> c_int;
    fn curl_slist_append(
        list: *mut HFileLibcurlCurlSlist,
        string: *const c_char,
    ) -> *mut HFileLibcurlCurlSlist;
    fn curl_slist_free_all(list: *mut HFileLibcurlCurlSlist);
}

#[link(name = "crypto")]
unsafe extern "C" {
    fn HMAC(
        evp_md: *const c_void,
        key: *const c_void,
        key_len: c_int,
        d: *const c_uchar,
        n: usize,
        md: *mut c_uchar,
        md_len: *mut c_uint,
    ) -> *mut c_uchar;
    fn EVP_sha1() -> *const c_void;
    fn EVP_sha256() -> *const c_void;
    fn SHA256(d: *const c_uchar, n: usize, md: *mut c_uchar) -> *mut c_uchar;
}

unsafe fn cstr_bytes(ptr: *const c_char) -> Vec<u8> {
    if ptr.is_null() {
        Vec::new()
    } else {
        CStr::from_ptr(ptr).to_bytes().to_vec()
    }
}

unsafe fn kput_cstring(s: &mut kstring_t, text: String) -> c_int {
    match CString::new(text) {
        Ok(cstr) => crate::htslib_rs::hts::kputs(cstr.as_bytes(), s),
        Err(_) => -1,
    }
}

unsafe fn kputs_literal(text: &[u8], s: &mut kstring_t) -> c_int {
    crate::htslib_rs::hts::kputsn(text, text.len(), s)
}

unsafe fn ks_release_or_free(s: &mut kstring_t) -> *mut c_char {
    if s.data.is_empty() {
        std::ptr::null_mut()
    } else {
        // Real FFI boundary: AuthHeaders stores raw NUL-terminated C strings
        // that libcurl reads, so build an owned malloc'd C string here.
        match CString::new(crate::htslib_rs::hts::ks_release(s)) {
            Ok(cstr) => libc::strdup(cstr.as_ptr()),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

// original: s3_sign (htslib/hfile_s3.c:142)
pub unsafe fn hfile_s3_c_142_s3_sign(
    digest: *mut c_uchar,
    key: *mut kstring_t,
    message: *mut kstring_t,
) -> usize {
    let mut len = 0 as c_uint;
    HMAC(
        EVP_sha1(),
        (*key).data.as_ptr().cast(),
        (*key).data.len() as c_int,
        (*message).data.as_ptr().cast(),
        (*message).data.len(),
        digest,
        &mut len,
    );
    len as usize
}

// original: s3_sha256 (htslib/hfile_s3.c:152)
pub unsafe fn hfile_s3_c_152_s3_sha256(in_: *const c_uchar, length: usize, out: *mut c_uchar) {
    SHA256(in_, length, out);
}

// original: s3_sign_sha256 (htslib/hfile_s3.c:157)
pub unsafe fn hfile_s3_c_157_s3_sign_sha256(
    key: *const c_void,
    key_len: c_int,
    d: *const c_uchar,
    n: c_int,
    md: *mut c_uchar,
    md_len: *mut c_uint,
) {
    HMAC(EVP_sha256(), key, key_len, d, n as usize, md, md_len);
}

// original: urldecode_kput (htslib/hfile_s3.c:165)
pub unsafe fn hfile_s3_c_165_urldecode_kput(s: *const c_char, len: c_int, str_: &mut kstring_t) {
    let mut buf = [0 as c_char; 3];
    let mut i = 0;

    while i < len {
        if *s.add(i as usize) == b'%' as c_char && i + 2 < len {
            buf[0] = *s.add((i + 1) as usize);
            buf[1] = *s.add((i + 2) as usize);
            crate::htslib_rs::hts::kputc(
                libc::strtol(buf.as_ptr(), std::ptr::null_mut(), 16) as c_int,
                str_,
            );
            i += 3;
        } else {
            crate::htslib_rs::hts::kputc(*s.add(i as usize) as c_int, str_);
            i += 1;
        }
    }
}

// original: base64_kput (htslib/hfile_s3.c:181)
pub unsafe fn hfile_s3_c_181_base64_kput(data: *const c_uchar, len: usize, str_: &mut kstring_t) {
    static BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut i = 0usize;
    let mut x = 0u32;
    let mut bits = 0;
    let mut pad = 0usize;

    while bits != 0 || i < len {
        if bits < 6 {
            x <<= 8;
            bits += 8;
            if i < len {
                x |= *data.add(i) as u32;
                i += 1;
            } else {
                pad += 1;
            }
        }

        bits -= 6;
        crate::htslib_rs::hts::kputc(BASE64[((x >> bits) & 63) as usize] as c_int, str_);
    }

    str_.data.truncate(str_.data.len() - pad);
    crate::htslib_rs::hts::kputsn(b"==", pad, str_);
}

// original: is_dns_compliant (htslib/hfile_s3.c:206)
pub unsafe fn hfile_s3_c_206_is_dns_compliant(
    s0: *const c_char,
    slim: *const c_char,
    is_https: c_int,
) -> c_int {
    let mut has_nondigit = 0;
    let mut len = 0;
    let mut s = s0;
    while s < slim {
        let c = *s as u8;
        if c.is_ascii_lowercase() {
            has_nondigit = 1;
        } else if c == b'-' {
            has_nondigit = 1;
            if s == s0 || s.add(1) == slim {
                return 0;
            }
        } else if c.is_ascii_digit() {
        } else if c == b'.' {
            if is_https != 0 {
                return 0;
            }
            if s == s0 || !(*s.sub(1) as u8).is_ascii_alphanumeric() {
                return 0;
            }
            if s.add(1) == slim || !(*s.add(1) as u8).is_ascii_alphanumeric() {
                return 0;
            }
        } else {
            return 0;
        }
        len += 1;
        s = s.add(1);
    }

    (has_nondigit != 0 && (3..=63).contains(&len)) as c_int
}

// original: expand_tilde_open (htslib/hfile_s3.c:231)
pub unsafe fn hfile_s3_c_231_expand_tilde_open(
    fname: *const c_char,
    mode: *const c_char,
) -> *mut libc::FILE {
    if libc::strncmp(fname, c"~/".as_ptr(), 2) == 0 {
        let mut full_fname = kstring_t::default();
        let home = libc::getenv(c"HOME".as_ptr());
        if home.is_null() {
            return std::ptr::null_mut();
        }

        crate::htslib_rs::hts::kputs(&cstr_bytes(home), &mut full_fname);
        crate::htslib_rs::hts::kputs(&cstr_bytes(fname.add(1)), &mut full_fname);

        // Real syscall boundary: fopen needs a NUL-terminated path.
        let fp = match CString::new(full_fname.data.clone()) {
            Ok(path) => libc::fopen(path.as_ptr(), mode),
            Err(_) => std::ptr::null_mut(),
        };
        fp
    } else {
        libc::fopen(fname, mode)
    }
}

unsafe fn hfile_s3_c_252_parse_ini(
    fname: *const c_char,
    section: *const c_char,
    pairs: &[(*const c_char, *mut kstring_t)],
) {
    let mut line = kstring_t::default();
    let mut active = 1;
    let fp = hfile_s3_c_231_expand_tilde_open(fname, c"r".as_ptr());
    if fp.is_null() {
        return;
    }

    while {
        line.data.clear();
        crate::htslib_rs::hts::kfgetline(&mut line, fp) >= 0
    } {
        // The line parsing below relies on C-string pointer arithmetic, so build
        // a NUL-terminated working copy of the owned bytes at this boundary.
        let mut work: Vec<u8> = line.data.clone();
        work.push(0);
        let work_ptr = work.as_mut_ptr().cast::<c_char>();
        if !work_ptr.is_null() && *work_ptr == b'[' as c_char {
            let s = libc::strchr(work_ptr, b']' as c_int);
            if !s.is_null() {
                *s = 0;
                active = (libc::strcmp(work_ptr.add(1), section) == 0) as c_int;
            }
        } else if active != 0 {
            let s = libc::strpbrk(work_ptr, c":=".as_ptr());
            if !s.is_null() {
                let mut key = work_ptr;
                let mut value = s.add(1);
                while libc::isspace(*key as c_uchar as c_int) != 0 {
                    key = key.add(1);
                }
                let mut end = s;
                while end > key && libc::isspace(*end.sub(1) as c_uchar as c_int) != 0 {
                    end = end.sub(1);
                }
                *end = 0;
                while libc::isspace(*value as c_uchar as c_int) != 0 {
                    value = value.add(1);
                }
                let mut vlen = line.data.len();
                while vlen > 0 && libc::isspace(*work_ptr.add(vlen - 1) as c_uchar as c_int) != 0 {
                    vlen -= 1;
                    *work_ptr.add(vlen) = 0;
                }

                for &(akey, avar) in pairs {
                    if libc::strcmp(key, akey) == 0 {
                        (*avar).data.clear();
                        crate::htslib_rs::hts::kputs(&cstr_bytes(value), &mut *avar);
                        break;
                    }
                }
            }
        }
    }

    libc::fclose(fp);
}

// original: parse_simple (htslib/hfile_s3.c:294)
pub unsafe fn hfile_s3_c_294_parse_simple(
    fname: *const c_char,
    id: *mut kstring_t,
    secret: *mut kstring_t,
) {
    let mut text = kstring_t::default();
    let fp = hfile_s3_c_231_expand_tilde_open(fname, c"r".as_ptr());
    if fp.is_null() {
        return;
    }

    while crate::htslib_rs::hts::kfgetline(&mut text, fp) >= 0 {
        crate::htslib_rs::hts::kputc(b' ' as c_int, &mut text);
    }
    libc::fclose(fp);

    // The whitespace scanning below uses C-string pointer arithmetic, so build a
    // NUL-terminated working copy of the owned bytes at this boundary.
    let mut work: Vec<u8> = text.data.clone();
    work.push(0);
    let mut s = work.as_ptr().cast::<c_char>();
    while libc::isspace(*s as c_uchar as c_int) != 0 {
        s = s.add(1);
    }
    let len = libc::strcspn(s, c" \t".as_ptr());
    crate::htslib_rs::hts::kputsn(
        std::slice::from_raw_parts(s.cast::<u8>(), len),
        len,
        &mut *id,
    );

    s = s.add(len);
    while libc::isspace(*s as c_uchar as c_int) != 0 {
        s = s.add(1);
    }
    let slen = libc::strcspn(s, c" \t".as_ptr());
    crate::htslib_rs::hts::kputsn(
        std::slice::from_raw_parts(s.cast::<u8>(), slen),
        slen,
        &mut *secret,
    );
}

unsafe fn hfile_s3_copy_auth_headers(
    ad: &mut S3AuthDataLayout,
    hdrs: *mut *mut *mut c_char,
) -> c_int {
    ad.headers.clear();
    if ad.headers.push_strdup(ad.date.as_ptr()).is_err() {
        ad.headers.free_all_untransferred();
        return -1;
    }

    if ad.token.data.len() != 0 {
        let mut token_hdr = kstring_t::default();
        crate::htslib_rs::hts::kputs(b"X-Amz-Security-Token: ", &mut token_hdr);
        let token_bytes = ad.token.data.clone();
        crate::htslib_rs::hts::kputs(&token_bytes, &mut token_hdr);
        if token_hdr.data.is_empty() {
            ad.headers.free_all_untransferred();
            return -1;
        }
        if ad.headers.push_released_kstring(&mut token_hdr).is_err() {
            crate::htslib_rs::hts::ks_free(&mut token_hdr);
            ad.headers.free_all_untransferred();
            return -1;
        }
    }

    if ad.auth_hdr.data.len() != 0 {
        // push_strdup needs a NUL-terminated C string; build one at this boundary.
        let auth_c = CString::new(ad.auth_hdr.data.clone()).unwrap_or_default();
        if ad.headers.push_strdup(auth_c.as_ptr()).is_err() {
            ad.headers.free_all_untransferred();
            return -1;
        }
    }

    *hdrs = ad.headers.as_raw_mut_ptr();
    0
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct NullableHeaderPtr(Option<NonNull<c_char>>);

impl NullableHeaderPtr {
    fn none() -> Self {
        Self(None)
    }
}

#[derive(Default)]
struct AuthHeaders {
    raw: Vec<NullableHeaderPtr>,
}

impl AuthHeaders {
    fn clear(&mut self) {
        self.raw.clear();
    }

    unsafe fn free_all_untransferred(&mut self) {
        for value in &mut self.raw {
            if let Some(ptr) = value.0.take() {
                libc::free(ptr.as_ptr().cast());
            }
        }
        self.raw.clear();
    }

    unsafe fn push_strdup(&mut self, text: *const c_char) -> Result<(), ()> {
        let Some(value) = NonNull::new(libc::strdup(text).cast()) else {
            return Err(());
        };
        self.raw.push(NullableHeaderPtr(Some(value)));
        Ok(())
    }

    unsafe fn push_released_kstring(&mut self, s: &mut kstring_t) -> Result<(), ()> {
        let Some(value) = NonNull::new(ks_release_or_free(s)) else {
            return Err(());
        };
        self.raw.push(NullableHeaderPtr(Some(value)));
        Ok(())
    }

    fn as_raw_mut_ptr(&mut self) -> *mut *mut c_char {
        if !matches!(self.raw.last(), Some(NullableHeaderPtr(None))) {
            self.raw.push(NullableHeaderPtr::none());
        }
        self.raw.as_mut_ptr().cast()
    }
}

pub struct S3AuthDataLayout {
    id: kstring_t,
    token: kstring_t,
    secret: kstring_t,
    region: kstring_t,
    canonical_query_string: kstring_t,
    user_query_string: kstring_t,
    host: kstring_t,
    profile: kstring_t,
    creds_expiry_time: libc::time_t,
    bucket: CString,
    auth_hdr: kstring_t,
    auth_time: libc::time_t,
    date: [c_char; 40],
    date_long: [c_char; 17],
    date_short: [c_char; 9],
    date_html: kstring_t,
    mode: c_char,
    headers: AuthHeaders,
    refcount: c_int,
}

impl Default for S3AuthDataLayout {
    fn default() -> Self {
        Self {
            id: kstring_t::default(),
            token: kstring_t::default(),
            secret: kstring_t::default(),
            region: kstring_t::default(),
            canonical_query_string: kstring_t::default(),
            user_query_string: kstring_t::default(),
            host: kstring_t::default(),
            profile: kstring_t::default(),
            creds_expiry_time: 0,
            bucket: CString::new("").expect("empty CString"),
            auth_hdr: kstring_t::default(),
            auth_time: 0,
            date: [0; 40],
            date_long: [0; 17],
            date_short: [0; 9],
            date_html: kstring_t::default(),
            mode: 0,
            headers: AuthHeaders::default(),
            refcount: 0,
        }
    }
}

#[allow(non_camel_case_types)]
pub type s3_auth_data = S3AuthDataLayout;

// original: free_auth_data (htslib/hfile_s3.c:319)
pub unsafe fn hfile_s3_c_319_free_auth_data(ad: *mut s3_auth_data) {
    let ad = ad.cast::<S3AuthDataLayout>();
    if (*ad).refcount > 0 {
        (*ad).refcount -= 1;
        return;
    }
    // The owned kstring_t Vec buffers (and the bucket CString) are released when
    // this Box drops; no manual libc::free of buffer pointers is needed.
    drop(Box::from_raw(ad));
}

// original: parse_rfc3339_date (htslib/hfile_s3.c:333)
pub unsafe fn hfile_s3_c_333_parse_rfc3339_date(datetime: *mut kstring_t) -> libc::time_t {
    let mut offset = 0;
    let mut should_be_t = 0 as c_char;
    let mut timezone = [0 as c_char; 10];
    let mut year = 0 as c_uint;
    let mut mon = 0 as c_uint;
    let mut day = 0 as c_uint;
    let mut hour = 0 as c_uint;
    let mut min = 0 as c_uint;
    let mut sec = 0 as c_uint;

    if (*datetime).data.is_empty() {
        return 0;
    }

    // sscanf needs a NUL-terminated C string; build one from the owned bytes.
    let Ok(datetime_c) = CString::new((*datetime).data.clone()) else {
        return 0;
    };
    let num = libc::sscanf(
        datetime_c.as_ptr(),
        c"%4u-%2u-%2u%c%2u:%2u:%2u%9s".as_ptr(),
        &mut year,
        &mut mon,
        &mut day,
        &mut should_be_t,
        &mut hour,
        &mut min,
        &mut sec,
        timezone.as_mut_ptr(),
    );
    if num < 8 {
        return 0;
    }
    if should_be_t != b'T' as c_char
        && should_be_t != b't' as c_char
        && should_be_t != b' ' as c_char
    {
        return 0;
    }

    let mut parsed = libc::tm {
        tm_sec: sec as c_int,
        tm_min: min as c_int,
        tm_hour: hour as c_int,
        tm_mday: day as c_int,
        tm_mon: mon as c_int - 1,
        tm_year: year as c_int - 1900,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        #[cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "l4re"
        ))]
        tm_gmtoff: 0,
        #[cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "l4re"
        ))]
        tm_zone: std::ptr::null(),
    };

    match timezone[0] as u8 {
        b'Z' | b'z' | 0 => {}
        b'+' | b'-' => {
            let mut hr_off = 0 as c_uint;
            let mut min_off = 0 as c_uint;
            if libc::sscanf(
                timezone.as_ptr().add(1),
                c"%2u:%2u".as_ptr(),
                &mut hr_off,
                &mut min_off,
            ) != 0
                && hr_off < 24
                && min_off <= 60
            {
                offset = ((hr_off * 60 + min_off) as c_int)
                    * if timezone[0] == b'+' as c_char {
                        -60
                    } else {
                        60
                    };
            }
        }
        _ => return 0,
    }

    let when = crate::htslib_rs::hts::hts_time_gm(&mut parsed);
    if when >= 0 {
        when + offset as libc::time_t
    } else {
        0
    }
}

// original: refresh_auth_data (htslib/hfile_s3.c:378)
pub unsafe fn hfile_s3_c_378_refresh_auth_data(ad: *mut s3_auth_data) {
    let ad = ad.cast::<S3AuthDataLayout>();
    let v = libc::getenv(c"AWS_SHARED_CREDENTIALS_FILE".as_ptr());
    let mut expiry_time = kstring_t::default();
    // parse_ini wants a NUL-terminated section name; build one from profile bytes.
    let profile_c = CString::new((*ad).profile.data.clone()).unwrap_or_default();
    hfile_s3_c_252_parse_ini(
        if v.is_null() {
            c"~/.aws/credentials".as_ptr()
        } else {
            v
        },
        profile_c.as_ptr(),
        &[
            (c"aws_access_key_id".as_ptr(), &mut (*ad).id),
            (c"aws_secret_access_key".as_ptr(), &mut (*ad).secret),
            (c"aws_session_token".as_ptr(), &mut (*ad).token),
            (c"expiry_time".as_ptr(), &mut expiry_time),
        ],
    );
    if expiry_time.data.len() != 0 {
        (*ad).creds_expiry_time = hfile_s3_c_333_parse_rfc3339_date(&mut expiry_time);
    }
    crate::htslib_rs::hts::ks_free(&mut expiry_time);
}

// original: escape_query (htslib/hfile_s3.c:396)
unsafe fn hfile_s3_escape_query_owned(qs: &CStr) -> Option<CString> {
    let mut escaped = Vec::with_capacity(qs.to_bytes().len() * 3);
    for &c in qs.to_bytes() {
        if c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'~' | b'.' | b'/' | b'=' | b'&')
        {
            escaped.push(c);
        } else {
            escaped.extend_from_slice(format!("%{c:02X}").as_bytes());
        }
    }
    CString::new(escaped).ok()
}

pub unsafe fn hfile_s3_c_396_escape_query(qs: *const c_char) -> *mut c_char {
    if qs.is_null() {
        return std::ptr::null_mut();
    }
    hfile_s3_escape_query_owned(CStr::from_ptr(qs)).map_or(std::ptr::null_mut(), CString::into_raw)
}

// original: escape_path (htslib/hfile_s3.c:424)
unsafe fn hfile_s3_escape_path_owned(path: &CStr) -> Option<CString> {
    let bytes = path.to_bytes();
    let mut escaped = Vec::with_capacity(bytes.len() * 3);
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'?' {
            break;
        }

        if c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'~' | b'.' | b'/') {
            escaped.push(c);
        } else {
            escaped.extend_from_slice(format!("%{c:02X}").as_bytes());
        }
        i += 1;
    }

    if i != bytes.len() {
        escaped.extend_from_slice(&bytes[i..]);
    }
    CString::new(escaped).ok()
}

pub unsafe fn hfile_s3_c_424_escape_path(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    hfile_s3_escape_path_owned(CStr::from_ptr(path)).map_or(std::ptr::null_mut(), CString::into_raw)
}

// original: is_escaped (htslib/hfile_s3.c:460)
pub unsafe fn hfile_s3_c_460_is_escaped(str_: *const c_char) -> c_int {
    let mut c = str_;
    let mut escaped = 0;
    let mut needs_escape = 0;

    while *c != 0 {
        let ch = *c as u8;
        if ch == b'%' && *c.add(1) != 0 && *c.add(2) != 0 {
            if (*c.add(1) as u8).is_ascii_hexdigit() && (*c.add(2) as u8).is_ascii_hexdigit() {
                escaped = 1;
                c = c.add(3);
                continue;
            } else {
                escaped = 0;
            }
        }
        if !(ch.is_ascii_digit()
            || ch.is_ascii_uppercase()
            || ch.is_ascii_lowercase()
            || ch == b'_'
            || ch == b'-'
            || ch == b'~'
            || ch == b'.'
            || ch == b'/')
        {
            needs_escape = 1;
        }
        c = c.add(1);
    }

    (escaped != 0 || needs_escape == 0) as c_int
}

// original: redirect_endpoint (htslib/hfile_s3.c:488)
pub unsafe extern "C" fn hfile_s3_c_488_redirect_endpoint(
    auth: *mut c_void,
    response: libc::c_long,
    header: *mut kstring_t,
    url: *mut kstring_t,
) -> c_int {
    let ad = auth.cast::<S3AuthDataLayout>();
    let mut ret = -1;
    if header.is_null() || (*header).data.is_empty() {
        return ret;
    }

    // The header scanning relies on C-string pointer arithmetic, so build a
    // NUL-terminated working copy of the owned bytes at this boundary.
    let mut work: Vec<u8> = (*header).data.clone();
    work.push(0);
    let work_ptr = work.as_mut_ptr().cast::<c_char>();
    let mut new_region = libc::strstr(work_ptr, c"x-amz-bucket-region: ".as_ptr());
    if !new_region.is_null() {
        new_region = new_region.add(c"x-amz-bucket-region: ".to_bytes().len());
        let mut end = new_region;
        while libc::isalnum(*end as c_uchar as c_int) != 0
            || libc::ispunct(*end as c_uchar as c_int) != 0
        {
            end = end.add(1);
        }
        *end = 0;

        let host_c = CString::new((*ad).host.data.clone()).unwrap_or_default();
        if libc::strstr(host_c.as_ptr(), c"amazonaws.com".as_ptr()).is_null() {
            return ret;
        }
        let new_region_bytes = cstr_bytes(new_region);
        (*ad).region.data.clear();
        crate::htslib_rs::hts::kputs(&new_region_bytes, &mut (*ad).region);
        (*ad).host.data.clear();
        if kput_cstring(
            &mut (*ad).host,
            format!(
                "s3.{}.amazonaws.com",
                String::from_utf8_lossy(&new_region_bytes)
            ),
        ) < 0
        {
            return ret;
        }

        if (*ad).region.data.len() != 0 && (*ad).host.data.len() != 0 {
            (*url).data.clear();
            let host_bytes = (*ad).host.data.clone();
            crate::htslib_rs::hts::kputs(&host_bytes, &mut *url);
            let bucket = (*ad).bucket.as_ptr();
            crate::htslib_rs::hts::kputsn(
                std::slice::from_raw_parts(bucket.cast::<u8>(), libc::strlen(bucket)),
                libc::strlen(bucket),
                &mut *url,
            );
            if (*ad).user_query_string.data.len() != 0 {
                crate::htslib_rs::hts::kputc(b'?' as c_int, &mut *url);
                let uqs = (*ad).user_query_string.data.clone();
                crate::htslib_rs::hts::kputsn(&uqs, uqs.len(), &mut *url);
            }
            ret = 0;
        }
    }

    ret
}

// original: setup_auth_data (htslib/hfile_s3.c:545)
pub unsafe fn hfile_s3_c_545_setup_auth_data(
    s3url: *const c_char,
    mode: *const c_char,
    sigver: c_int,
    url: *mut kstring_t,
) -> *mut s3_auth_data {
    let ad = Box::into_raw(Box::<S3AuthDataLayout>::default());
    (*ad).mode = if libc::strchr(mode, b'r' as c_int).is_null() {
        b'w' as c_char
    } else {
        b'r' as c_char
    };

    let mut is_https = 1;
    let mut address_style = 0;
    let mut bucket: *const c_char;
    if *s3url.add(2) == b'+' as c_char {
        bucket = libc::strchr(s3url, b':' as c_int);
        if bucket.is_null() {
            drop(Box::from_raw(ad));
            return std::ptr::null_mut();
        }
        bucket = bucket.add(1);
        let prefix_len = bucket.offset_from(s3url.add(3)) as usize;
        crate::htslib_rs::hts::kputsn(
            std::slice::from_raw_parts(s3url.add(3).cast::<u8>(), prefix_len),
            prefix_len,
            &mut *url,
        );
        is_https = {
            let ud = &(*url).data;
            (ud.len() >= 6 && &ud[..6] == b"https:") as c_int
        };
    } else {
        crate::htslib_rs::hts::kputs(b"https:", &mut *url);
        bucket = s3url.add(3);
    }
    while *bucket == b'/' as c_char {
        crate::htslib_rs::hts::kputc(*bucket as c_int, &mut *url);
        bucket = bucket.add(1);
    }

    let mut path = bucket.add(libc::strcspn(bucket, c"/?#@".as_ptr()));
    if *path == b'@' as c_char {
        let colon = libc::strpbrk(bucket, c":@".as_ptr());
        if *colon != b':' as c_char {
            hfile_s3_c_165_urldecode_kput(
                bucket,
                colon.offset_from(bucket) as c_int,
                &mut (*ad).profile,
            );
        } else {
            let colon2 = libc::strpbrk(colon.add(1), c":@".as_ptr());
            hfile_s3_c_165_urldecode_kput(
                bucket,
                colon.offset_from(bucket) as c_int,
                &mut (*ad).id,
            );
            hfile_s3_c_165_urldecode_kput(
                colon.add(1),
                colon2.offset_from(colon.add(1)) as c_int,
                &mut (*ad).secret,
            );
            if *colon2 == b':' as c_char {
                hfile_s3_c_165_urldecode_kput(
                    colon2.add(1),
                    path.offset_from(colon2.add(1)) as c_int,
                    &mut (*ad).token,
                );
            }
        }
        bucket = path.add(1);
        path = bucket.add(libc::strcspn(bucket, c"/?#".as_ptr()));
    } else {
        let mut v = libc::getenv(c"AWS_ACCESS_KEY_ID".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v), &mut (*ad).id);
        }
        v = libc::getenv(c"AWS_SECRET_ACCESS_KEY".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v), &mut (*ad).secret);
        }
        v = libc::getenv(c"AWS_SESSION_TOKEN".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v), &mut (*ad).token);
        }
        v = libc::getenv(c"AWS_DEFAULT_REGION".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v), &mut (*ad).region);
        }
        v = libc::getenv(c"HTS_S3_HOST".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v), &mut (*ad).host);
        }
        v = libc::getenv(c"AWS_DEFAULT_PROFILE".as_ptr());
        if v.is_null() {
            v = libc::getenv(c"AWS_PROFILE".as_ptr());
        }
        if v.is_null() {
            v = c"default".as_ptr().cast_mut();
        }
        crate::htslib_rs::hts::kputs(&cstr_bytes(v), &mut (*ad).profile);
        v = libc::getenv(c"HTS_S3_ADDRESS_STYLE".as_ptr());
        if !v.is_null() {
            if libc::strcasecmp(v, c"virtual".as_ptr()) == 0 {
                address_style = 1;
            } else if libc::strcasecmp(v, c"path".as_ptr()) == 0 {
                address_style = 2;
            }
        }
    }

    if (*ad).id.data.len() == 0 {
        let mut url_style = kstring_t::default();
        let mut expiry_time = kstring_t::default();
        let v = libc::getenv(c"AWS_SHARED_CREDENTIALS_FILE".as_ptr());
        let profile_c = CString::new((*ad).profile.data.clone()).unwrap_or_default();
        hfile_s3_c_252_parse_ini(
            if v.is_null() {
                c"~/.aws/credentials".as_ptr()
            } else {
                v
            },
            profile_c.as_ptr(),
            &[
                (c"aws_access_key_id".as_ptr(), &mut (*ad).id),
                (c"aws_secret_access_key".as_ptr(), &mut (*ad).secret),
                (c"aws_session_token".as_ptr(), &mut (*ad).token),
                (c"region".as_ptr(), &mut (*ad).region),
                (c"addressing_style".as_ptr(), &mut url_style),
                (c"expiry_time".as_ptr(), &mut expiry_time),
            ],
        );
        if url_style.data.len() != 0 {
            if url_style.data == b"virtual" {
                address_style = 1;
            } else if url_style.data == b"path" {
                address_style = 2;
            } else {
                address_style = 0;
            }
        }
        if expiry_time.data.len() != 0 {
            (*ad).creds_expiry_time = hfile_s3_c_333_parse_rfc3339_date(&mut expiry_time);
        }
        crate::htslib_rs::hts::ks_free(&mut url_style);
        crate::htslib_rs::hts::ks_free(&mut expiry_time);
    }

    if (*ad).id.data.len() == 0 {
        let mut url_style = kstring_t::default();
        let v = libc::getenv(c"HTS_S3_S3CFG".as_ptr());
        let profile_c = CString::new((*ad).profile.data.clone()).unwrap_or_default();
        hfile_s3_c_252_parse_ini(
            if v.is_null() { c"~/.s3cfg".as_ptr() } else { v },
            profile_c.as_ptr(),
            &[
                (c"access_key".as_ptr(), &mut (*ad).id),
                (c"secret_key".as_ptr(), &mut (*ad).secret),
                (c"access_token".as_ptr(), &mut (*ad).token),
                (c"host_base".as_ptr(), &mut (*ad).host),
                (c"bucket_location".as_ptr(), &mut (*ad).region),
                (c"host_bucket".as_ptr(), &mut url_style),
            ],
        );
        if url_style.data.len() != 0 {
            address_style = if url_style
                .data
                .windows(b"%(bucket)s".len())
                .any(|w| w == b"%(bucket)s")
            {
                0
            } else {
                2
            };
        }
        crate::htslib_rs::hts::ks_free(&mut url_style);
    }

    if (*ad).id.data.len() == 0 {
        hfile_s3_c_294_parse_simple(c"~/.awssecret".as_ptr(), &mut (*ad).id, &mut (*ad).secret);
    }

    let dns_compliant = match address_style {
        1 => 1,
        2 => 0,
        _ => hfile_s3_c_206_is_dns_compliant(bucket, path, is_https),
    };
    if (*ad).host.data.len() == 0 {
        crate::htslib_rs::hts::kputs(b"s3.amazonaws.com", &mut (*ad).host);
    }
    if dns_compliant == 0
        && (*ad).region.data.len() > 0
        && (*ad).host.data == b"s3.amazonaws.com"
    {
        let region_str = String::from_utf8_lossy(&(*ad).region.data).into_owned();
        (*ad).host.data.clear();
        if kput_cstring(
            &mut (*ad).host,
            format!("s3.{}.amazonaws.com", region_str),
        ) < 0
        {
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        }
    }
    if (*ad).region.data.len() == 0 {
        crate::htslib_rs::hts::kputs(b"us-east-1", &mut (*ad).region);
    }

    let mut escaped: Option<CString> = None;
    if hfile_s3_c_460_is_escaped(path) == 0 {
        escaped = hfile_s3_escape_path_owned(CStr::from_ptr(path));
        if escaped.is_none() {
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        }
    }

    let bucket_len = path.offset_from(bucket) as usize;
    let url_path_pos: usize;
    if dns_compliant != 0 {
        let url_host_pos = (*url).data.len();
        crate::htslib_rs::hts::kputsn_(
            std::slice::from_raw_parts(bucket.cast::<u8>(), bucket_len),
            bucket_len,
            &mut *url,
        );
        crate::htslib_rs::hts::kputc(b'.' as c_int, &mut *url);
        let host_bytes = (*ad).host.data.clone();
        crate::htslib_rs::hts::kputsn(&host_bytes, host_bytes.len(), &mut *url);
        url_path_pos = (*url).data.len();
        if sigver == 4 {
            let host_from_url = (&(*url).data)[url_host_pos..].to_vec();
            (*ad).host.data.clear();
            crate::htslib_rs::hts::kputsn(&host_from_url, host_from_url.len(), &mut (*ad).host);
        }
    } else {
        let host_bytes = (*ad).host.data.clone();
        crate::htslib_rs::hts::kputsn(&host_bytes, host_bytes.len(), &mut *url);
        url_path_pos = (*url).data.len();
        crate::htslib_rs::hts::kputc(b'/' as c_int, &mut *url);
        crate::htslib_rs::hts::kputsn(
            std::slice::from_raw_parts(bucket.cast::<u8>(), bucket_len),
            bucket_len,
            &mut *url,
        );
    }
    let escaped_path = escaped.as_ref().map_or(path, |escaped| escaped.as_ptr());
    crate::htslib_rs::hts::kputs(&cstr_bytes(escaped_path), &mut *url);

    let bucket = if sigver == 4 || dns_compliant == 0 {
        (&(*url).data)[url_path_pos..].to_vec()
    } else {
        let source_bucket = bucket;
        let mut bucket_bytes = Vec::with_capacity((*url).data.len() - url_path_pos + bucket_len + 1);
        bucket_bytes.push(b'/');
        bucket_bytes.extend_from_slice(std::slice::from_raw_parts(
            source_bucket.cast::<u8>(),
            bucket_len,
        ));
        bucket_bytes.extend_from_slice(&(&(*url).data)[url_path_pos..]);
        bucket_bytes
    };
    let mut bucket = match CString::new(bucket) {
        Ok(bucket) => bucket,
        Err(_) => {
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        }
    };
    if let Some(query_offset) = bucket.as_bytes().iter().position(|&b| b == b'?') {
        let bytes = bucket.as_bytes();
        if let Ok(query) = CString::new(&bytes[query_offset + 1..]) {
            crate::htslib_rs::hts::kputs(query.as_bytes(), &mut (*ad).user_query_string);
        }
        let mut bytes = bucket.into_bytes();
        bytes.truncate(query_offset);
        bucket = match CString::new(bytes) {
            Ok(bucket) => bucket,
            Err(_) => {
                hfile_s3_c_319_free_auth_data(ad.cast());
                return std::ptr::null_mut();
            }
        };
    }
    (*ad).bucket = bucket;
    ad.cast()
}

// original: v2_authorisation (htslib/hfile_s3.c:774)
pub unsafe extern "C" fn hfile_s3_c_774_v2_authorisation(
    ctx: *mut c_void,
    hdrs: *mut *mut *mut c_char,
) -> c_int {
    let ad = ctx.cast::<S3AuthDataLayout>();
    let now = libc::time(std::ptr::null_mut());
    let mut message = kstring_t::default();
    let mut digest = [0u8; DIGEST_BUFSIZ];

    if hdrs.is_null() {
        hfile_s3_c_319_free_auth_data(ad.cast());
        return 0;
    }
    if (*ad).creds_expiry_time > 0 && (*ad).creds_expiry_time - now < CREDENTIAL_LIFETIME {
        hfile_s3_c_378_refresh_auth_data(ad.cast());
    } else if now - (*ad).auth_time < AUTH_LIFETIME {
        *hdrs = std::ptr::null_mut();
        return 0;
    }

    write_s3_date_header(&mut (*ad).date, now);
    if (*ad).id.data.len() == 0 || (*ad).secret.data.len() == 0 {
        (*ad).auth_time = now;
        return hfile_s3_copy_auth_headers(&mut *ad, hdrs);
    }
    let method = if (*ad).mode == b'r' as c_char {
        "GET"
    } else {
        "PUT"
    };
    let token_prefix = if (*ad).token.data.len() != 0 {
        "x-amz-security-token:"
    } else {
        ""
    };
    let token = if (*ad).token.data.len() != 0 {
        String::from_utf8_lossy(&(*ad).token.data).into_owned()
    } else {
        String::new()
    };
    let token_nl = if (*ad).token.data.len() != 0 { "\n" } else { "" };
    if kput_cstring(
        &mut message,
        format!(
            "{}\n\n\n{}\n{}{}{}{}",
            method,
            CStr::from_ptr((*ad).date.as_ptr().add(6)).to_string_lossy(),
            token_prefix,
            token,
            token_nl,
            CStr::from_ptr((*ad).bucket.as_ptr()).to_string_lossy()
        ),
    ) < 0
    {
        return -1;
    }
    let digest_len = hfile_s3_c_142_s3_sign(digest.as_mut_ptr(), &mut (*ad).secret, &mut message);
    (*ad).auth_hdr.data.clear();
    if kput_cstring(
        &mut (*ad).auth_hdr,
        format!(
            "Authorization: AWS {}:",
            String::from_utf8_lossy(&(*ad).id.data)
        ),
    ) < 0
    {
        return -1;
    }
    hfile_s3_c_181_base64_kput(digest.as_ptr(), digest_len, &mut (*ad).auth_hdr);
    (*ad).auth_time = now;
    hfile_s3_copy_auth_headers(&mut *ad, hdrs)
}

// original: hash_string (htslib/hfile_s3.c:836)
pub unsafe fn hfile_s3_c_836_hash_string(
    in_: *mut c_char,
    length: usize,
    out: *mut c_char,
    out_len: usize,
) {
    let mut hashed = [0u8; SHA256_DIGEST_BUFSIZE];
    hfile_s3_c_152_s3_sha256(in_.cast(), length, hashed.as_mut_ptr());
    for (i, byte) in hashed.iter().enumerate() {
        libc::snprintf(
            out.add(i * 2),
            out_len - i * 2,
            c"%02x".as_ptr(),
            *byte as c_int,
        );
    }
}

// original: make_signature (htslib/hfile_s3.c:848)
pub unsafe fn hfile_s3_c_848_make_signature(
    ad: *mut s3_auth_data,
    string_to_sign: *mut kstring_t,
    signature_string: *mut c_char,
    sig_string_len: usize,
) -> c_int {
    let ad = ad.cast::<S3AuthDataLayout>();
    let mut date_key = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut date_region_key = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut date_region_service_key = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut signing_key = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut signature = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut secret_access_key = kstring_t::default();
    let mut len = 0 as c_uint;

    if kput_cstring(
        &mut secret_access_key,
        format!("AWS4{}", String::from_utf8_lossy(&(*ad).secret.data)),
    ) < 0
        || secret_access_key.data.len() == 0
    {
        return -1;
    }
    hfile_s3_c_157_s3_sign_sha256(
        secret_access_key.data.as_ptr().cast(),
        secret_access_key.data.len() as c_int,
        (*ad).date_short.as_ptr().cast(),
        libc::strlen((*ad).date_short.as_ptr()) as c_int,
        date_key.as_mut_ptr(),
        &mut len,
    );
    hfile_s3_c_157_s3_sign_sha256(
        date_key.as_ptr().cast(),
        len as c_int,
        (*ad).region.data.as_ptr().cast(),
        (*ad).region.data.len() as c_int,
        date_region_key.as_mut_ptr(),
        &mut len,
    );
    hfile_s3_c_157_s3_sign_sha256(
        date_region_key.as_ptr().cast(),
        len as c_int,
        c"s3".as_ptr().cast(),
        2,
        date_region_service_key.as_mut_ptr(),
        &mut len,
    );
    hfile_s3_c_157_s3_sign_sha256(
        date_region_service_key.as_ptr().cast(),
        len as c_int,
        c"aws4_request".as_ptr().cast(),
        12,
        signing_key.as_mut_ptr(),
        &mut len,
    );
    hfile_s3_c_157_s3_sign_sha256(
        signing_key.as_ptr().cast(),
        len as c_int,
        (*string_to_sign).data.as_ptr().cast(),
        (*string_to_sign).data.len() as c_int,
        signature.as_mut_ptr(),
        &mut len,
    );
    for (i, byte) in signature.iter().take(len as usize).enumerate() {
        libc::snprintf(
            signature_string.add(i * 2),
            sig_string_len - i * 2,
            c"%02x".as_ptr(),
            *byte as c_int,
        );
    }
    crate::htslib_rs::hts::ks_free(&mut secret_access_key);
    0
}

// original: make_authorisation (htslib/hfile_s3.c:884)
pub unsafe fn hfile_s3_c_884_make_authorisation(
    ad: *mut s3_auth_data,
    http_request: *mut c_char,
    content: *mut c_char,
    auth: *mut kstring_t,
) -> c_int {
    let ad = ad.cast::<S3AuthDataLayout>();
    let mut signed_headers = kstring_t::default();
    let mut canonical_headers = kstring_t::default();
    let mut canonical_request = kstring_t::default();
    let mut scope = kstring_t::default();
    let mut string_to_sign = kstring_t::default();
    let mut cr_hash = [0 as c_char; HASH_LENGTH_SHA256];
    let mut signature_string = [0 as c_char; HASH_LENGTH_SHA256];
    let mut ret = -1;

    if (*ad).token.data.len() == 0 {
        kputs_literal(b"host;x-amz-content-sha256;x-amz-date", &mut signed_headers);
    } else {
        kputs_literal(
            b"host;x-amz-content-sha256;x-amz-date;x-amz-security-token",
            &mut signed_headers,
        );
    }
    if signed_headers.data.len() == 0 {
        return -1;
    }

    if (*ad).token.data.len() == 0 {
        kput_cstring(
            &mut canonical_headers,
            format!(
                "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
                String::from_utf8_lossy(&(*ad).host.data),
                CStr::from_ptr(content).to_string_lossy(),
                CStr::from_ptr((*ad).date_long.as_ptr()).to_string_lossy()
            ),
        );
    } else {
        kput_cstring(
            &mut canonical_headers,
            format!(
                "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\nx-amz-security-token:{}\n",
                String::from_utf8_lossy(&(*ad).host.data),
                CStr::from_ptr(content).to_string_lossy(),
                CStr::from_ptr((*ad).date_long.as_ptr()).to_string_lossy(),
                String::from_utf8_lossy(&(*ad).token.data)
            ),
        );
    }
    if canonical_headers.data.len() != 0 {
        kput_cstring(
            &mut canonical_request,
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}",
                CStr::from_ptr(http_request).to_string_lossy(),
                CStr::from_ptr((*ad).bucket.as_ptr()).to_string_lossy(),
                String::from_utf8_lossy(&(*ad).canonical_query_string.data),
                String::from_utf8_lossy(&canonical_headers.data),
                String::from_utf8_lossy(&signed_headers.data),
                CStr::from_ptr(content).to_string_lossy()
            ),
        );
        if canonical_request.data.len() != 0 {
            hfile_s3_c_836_hash_string(
                canonical_request.data.as_mut_ptr().cast(),
                canonical_request.data.len(),
                cr_hash.as_mut_ptr(),
                cr_hash.len(),
            );
            kput_cstring(
                &mut scope,
                format!(
                    "{}/{}/s3/aws4_request",
                    CStr::from_ptr((*ad).date_short.as_ptr()).to_string_lossy(),
                    String::from_utf8_lossy(&(*ad).region.data)
                ),
            );
            if scope.data.len() != 0 {
                kput_cstring(
                    &mut string_to_sign,
                    format!(
                        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
                        CStr::from_ptr((*ad).date_long.as_ptr()).to_string_lossy(),
                        String::from_utf8_lossy(&scope.data),
                        CStr::from_ptr(cr_hash.as_ptr()).to_string_lossy()
                    ),
                );
                if string_to_sign.data.len() != 0
                    && hfile_s3_c_848_make_signature(
                        ad.cast(),
                        &mut string_to_sign,
                        signature_string.as_mut_ptr(),
                        signature_string.len(),
                    ) == 0
                {
                    kput_cstring(
                        &mut *auth,
                        format!(
                            "Authorization: AWS4-HMAC-SHA256 Credential={}/{}/{}/s3/aws4_request,SignedHeaders={},Signature={}",
                            String::from_utf8_lossy(&(*ad).id.data),
                            CStr::from_ptr((*ad).date_short.as_ptr()).to_string_lossy(),
                            String::from_utf8_lossy(&(*ad).region.data),
                            String::from_utf8_lossy(&signed_headers.data),
                            CStr::from_ptr(signature_string.as_ptr()).to_string_lossy()
                        ),
                    );
                    if (*auth).data.len() != 0 {
                        ret = 0;
                    }
                }
            }
        }
    }

    crate::htslib_rs::hts::ks_free(&mut signed_headers);
    crate::htslib_rs::hts::ks_free(&mut canonical_headers);
    crate::htslib_rs::hts::ks_free(&mut canonical_request);
    crate::htslib_rs::hts::ks_free(&mut scope);
    crate::htslib_rs::hts::ks_free(&mut string_to_sign);
    ret
}

// original: update_time (htslib/hfile_s3.c:968)
pub unsafe fn hfile_s3_c_968_update_time(ad: *mut s3_auth_data, now: libc::time_t) -> c_int {
    const AUTH_LIFETIME: libc::time_t = 60;
    let ad = ad.cast::<S3AuthDataLayout>();
    let mut ret = -1;

    if now - (*ad).auth_time > AUTH_LIFETIME {
        (*ad).auth_time = now;

        if !write_s3_v4_dates(&mut (*ad).date_long, &mut (*ad).date_short, now) {
            return -1;
        }

        (*ad).date_html.data.clear();
        crate::htslib_rs::hts::kputs(b"x-amz-date: ", &mut (*ad).date_html);
        crate::htslib_rs::hts::kputs(&cstr_bytes((*ad).date_long.as_ptr()), &mut (*ad).date_html);
    }

    if (*ad).date_html.data.len() != 0 {
        ret = 0;
    }

    ret
}

// original: query_cmp (htslib/hfile_s3.c:999)
pub unsafe fn hfile_s3_c_999_query_cmp(p1: *const c_void, p2: *const c_void) -> c_int {
    let q1 = p1.cast::<*const c_char>();
    let q2 = p2.cast::<*const c_char>();
    libc::strcmp(*q1, *q2)
}

// original: order_query_string (htslib/hfile_s3.c:1009)
pub unsafe fn hfile_s3_c_1009_order_query_string(qs: *mut kstring_t) -> c_int {
    if qs.is_null() {
        return -1;
    }

    if (*qs).data.is_empty() {
        return -1;
    }

    // ksplit on '&' yields each maximal non-empty run between delimiters; build
    // the field list directly from the owned bytes (no in-buffer NUL splitting).
    let mut queries: Vec<Vec<u8>> = (*qs)
        .data
        .split(|&b| b == b'&')
        .filter(|field| !field.is_empty())
        .map(|field| field.to_vec())
        .collect();
    queries.sort();

    let mut ordered = kstring_t::default();
    let mut ret = -1;

    for (i, query) in queries.iter().enumerate() {
        if i != 0 {
            crate::htslib_rs::hts::kputs(b"&", &mut ordered);
        }
        crate::htslib_rs::hts::kputs(query, &mut ordered);
    }

    let escaped = if ordered.data.is_empty() {
        None
    } else {
        let ordered_c = CString::new(ordered.data.clone()).unwrap_or_default();
        hfile_s3_escape_query_owned(&ordered_c)
    };
    if let Some(escaped) = escaped {
        (*qs).data.clear();
        crate::htslib_rs::hts::kputs(escaped.as_bytes(), &mut *qs);
        ret = 0;
    }

    ret
}

// original: v4_authorisation (htslib/hfile_s3.c:1055)
pub unsafe extern "C" fn hfile_s3_c_1055_v4_authorisation(
    auth: *mut c_void,
    request: *mut c_char,
    content: *mut kstring_t,
    cqs: *mut c_char,
    hash: *mut kstring_t,
    auth_str: *mut kstring_t,
    date: *mut kstring_t,
    token: *mut kstring_t,
    uqs: c_int,
) -> c_int {
    let ad = auth.cast::<S3AuthDataLayout>();
    let mut content_hash = [0 as c_char; HASH_LENGTH_SHA256];
    if request.is_null() {
        hfile_s3_c_319_free_auth_data(ad.cast());
        return 0;
    }
    let now = libc::time(std::ptr::null_mut());
    if hfile_s3_c_968_update_time(ad.cast(), now) != 0 {
        return -1;
    }
    if (*ad).creds_expiry_time > 0 && (*ad).creds_expiry_time - now < CREDENTIAL_LIFETIME {
        hfile_s3_c_378_refresh_auth_data(ad.cast());
    }
    if !content.is_null() {
        hfile_s3_c_836_hash_string(
            (*content).data.as_mut_ptr().cast(),
            (*content).data.len(),
            content_hash.as_mut_ptr(),
            content_hash.len(),
        );
    } else {
        hfile_s3_c_836_hash_string(
            c"".as_ptr().cast_mut(),
            0,
            content_hash.as_mut_ptr(),
            content_hash.len(),
        );
    }
    (*ad).canonical_query_string.data.clear();
    crate::htslib_rs::hts::kputs(&cstr_bytes(cqs), &mut (*ad).canonical_query_string);
    if (*ad).canonical_query_string.data.len() == 0 {
        return -1;
    }
    if uqs != 0 {
        crate::htslib_rs::hts::kputs(b"&", &mut (*ad).canonical_query_string);
        let uqs_bytes = (*ad).user_query_string.data.clone();
        crate::htslib_rs::hts::kputs(&uqs_bytes, &mut (*ad).canonical_query_string);
        if hfile_s3_c_1009_order_query_string(&mut (*ad).canonical_query_string) != 0 {
            return -1;
        }
    }
    if hfile_s3_c_884_make_authorisation(ad.cast(), request, content_hash.as_mut_ptr(), auth_str)
        != 0
    {
        return -1;
    }
    let date_html_bytes = (*ad).date_html.data.clone();
    crate::htslib_rs::hts::kputs(&date_html_bytes, &mut *date);
    crate::htslib_rs::hts::kputsn(
        std::slice::from_raw_parts(content_hash.as_ptr().cast::<u8>(), HASH_LENGTH_SHA256),
        HASH_LENGTH_SHA256,
        &mut *hash,
    );
    if (*date).data.len() == 0 || (*hash).data.len() == 0 {
        return -1;
    }
    if (*ad).token.data.len() != 0 {
        kput_cstring(
            &mut *token,
            format!(
                "x-amz-security-token: {}",
                String::from_utf8_lossy(&(*ad).token.data)
            ),
        );
    }
    0
}

pub unsafe extern "C" fn hfile_s3_c_1055_v4_auth_header_callback(
    ctx: *mut c_void,
    hdrs: *mut *mut *mut c_char,
) -> c_int {
    let ad = ctx.cast::<S3AuthDataLayout>();
    let mut content_hash = [0 as c_char; HASH_LENGTH_SHA256];
    let mut content = kstring_t::default();
    let mut authorisation = kstring_t::default();
    let mut token_hdr = kstring_t::default();
    if hdrs.is_null() {
        hfile_s3_c_319_free_auth_data(ad.cast());
        return 0;
    }
    let now = libc::time(std::ptr::null_mut());
    if hfile_s3_c_968_update_time(ad.cast(), now) != 0 {
        return -1;
    }
    if (*ad).creds_expiry_time > 0 && (*ad).creds_expiry_time - now < CREDENTIAL_LIFETIME {
        hfile_s3_c_378_refresh_auth_data(ad.cast());
    }
    if (*ad).id.data.len() == 0 || (*ad).secret.data.len() == 0 {
        return hfile_s3_copy_auth_headers(&mut *ad, hdrs);
    }
    hfile_s3_c_836_hash_string(
        c"".as_ptr().cast_mut(),
        0,
        content_hash.as_mut_ptr(),
        content_hash.len(),
    );
    (*ad).canonical_query_string.data.clear();
    if (*ad).user_query_string.data.len() > 0 {
        let uqs_bytes = (*ad).user_query_string.data.clone();
        crate::htslib_rs::hts::kputs(&uqs_bytes, &mut (*ad).canonical_query_string);
        if hfile_s3_c_1009_order_query_string(&mut (*ad).canonical_query_string) != 0 {
            return -1;
        }
    } else {
        crate::htslib_rs::hts::kputs(b"", &mut (*ad).canonical_query_string);
    }
    if hfile_s3_c_884_make_authorisation(
        ad.cast(),
        c"GET".as_ptr().cast_mut(),
        content_hash.as_mut_ptr(),
        &mut authorisation,
    ) != 0
    {
        return -1;
    }
    kput_cstring(
        &mut content,
        format!(
            "x-amz-content-sha256: {}",
            CStr::from_ptr(content_hash.as_ptr()).to_string_lossy()
        ),
    );
    if (*ad).token.data.len() > 0 {
        crate::htslib_rs::hts::kputs(b"X-Amz-Security-Token: ", &mut token_hdr);
        let token_bytes = (*ad).token.data.clone();
        crate::htslib_rs::hts::kputs(&token_bytes, &mut token_hdr);
    }
    if content.data.len() == 0 {
        crate::htslib_rs::hts::ks_free(&mut authorisation);
        crate::htslib_rs::hts::ks_free(&mut content);
        crate::htslib_rs::hts::ks_free(&mut token_hdr);
        return -1;
    }
    // push_strdup needs a NUL-terminated C string; build one at this boundary.
    let date_html_c = CString::new((*ad).date_html.data.clone()).unwrap_or_default();
    (*ad).headers.clear();
    if (*ad)
        .headers
        .push_released_kstring(&mut authorisation)
        .is_err()
        || (*ad).headers.push_strdup(date_html_c.as_ptr()).is_err()
        || (*ad).headers.push_released_kstring(&mut content).is_err()
    {
        crate::htslib_rs::hts::ks_free(&mut authorisation);
        crate::htslib_rs::hts::ks_free(&mut content);
        crate::htslib_rs::hts::ks_free(&mut token_hdr);
        (*ad).headers.free_all_untransferred();
        return -1;
    }
    if !token_hdr.data.is_empty() && (*ad).headers.push_released_kstring(&mut token_hdr).is_err() {
        crate::htslib_rs::hts::ks_free(&mut token_hdr);
        (*ad).headers.free_all_untransferred();
        return -1;
    }
    *hdrs = (*ad).headers.as_raw_mut_ptr();
    0
}

pub unsafe fn hfile_s3_c_1055_handle_400_response(fp: *mut hFILE, ad: *mut s3_auth_data) -> c_int {
    let ad = ad.cast::<S3AuthDataLayout>();
    let mut buffer = [0 as c_char; 1024];
    let bytes = htslib_hfile_h_247_hread(fp, buffer.as_mut_ptr().cast(), buffer.len() - 1);
    if bytes < 0 {
        return -1;
    }
    buffer[bytes as usize] = 0;
    let mut region = libc::strstr(buffer.as_mut_ptr(), c"<Region>".as_ptr());
    if region.is_null() {
        return -1;
    }
    region = region.add(8);
    while libc::isspace(*region as c_uchar as c_int) != 0 {
        region = region.add(1);
    }
    let mut reg_end = libc::strchr(region, b'<' as c_int);
    if reg_end.is_null() || libc::strncmp(reg_end.add(1), c"/Region>".as_ptr(), 8) != 0 {
        return -1;
    }
    while reg_end > region && libc::isspace(*reg_end.sub(1) as c_uchar as c_int) != 0 {
        reg_end = reg_end.sub(1);
    }
    (*ad).region.data.clear();
    let region_len = reg_end.offset_from(region) as usize;
    crate::htslib_rs::hts::kputsn(
        std::slice::from_raw_parts(region.cast::<u8>(), region_len),
        region_len,
        &mut (*ad).region,
    );
    if (*ad).region.data.len() == 0 {
        -1
    } else {
        0
    }
}

// original: set_region (htslib/hfile_s3.c:1112)
pub unsafe fn hfile_s3_c_1112_set_region(ad: *mut s3_auth_data, region: *mut kstring_t) -> c_int {
    let ad = ad.cast::<S3AuthDataLayout>();
    (*ad).region.data.clear();
    let region_bytes = (*region).data.clone();
    (crate::htslib_rs::hts::kputsn(&region_bytes, region_bytes.len(), &mut (*ad).region) < 0)
        as c_int
}

// original: stristr (htslib/hfile_s3.c:1176)
pub unsafe fn hfile_s3_c_1176_stristr(
    mut haystack: *mut c_char,
    needle: *mut c_char,
) -> *mut c_char {
    while *haystack != 0 {
        let mut h = haystack;
        let mut n = needle;

        while (*h as u8).eq_ignore_ascii_case(&(*n as u8)) {
            h = h.add(1);
            n = n.add(1);
            if *h == 0 || *n == 0 {
                break;
            }
        }

        if *n == 0 {
            break;
        }

        haystack = haystack.add(1);
    }

    if *haystack == 0 {
        std::ptr::null_mut()
    } else {
        haystack
    }
}

// original: get_entry (htslib/hfile_s3.c:1198)
pub unsafe fn hfile_s3_c_1198_get_entry(
    in_: *mut c_char,
    start_tag: *mut c_char,
    end_tag: *mut c_char,
    out: *mut kstring_t,
) -> c_int {
    if in_.is_null() {
        return libc::EOF;
    }

    let mut start = hfile_s3_c_1176_stristr(in_, start_tag);
    if start.is_null() {
        return libc::EOF;
    }

    start = start.add(libc::strlen(start_tag));
    let end = hfile_s3_c_1176_stristr(start, end_tag);
    if end.is_null() {
        return libc::EOF;
    }

    let entry_len = end.offset_from(start) as usize;
    crate::htslib_rs::hts::kputsn(
        std::slice::from_raw_parts(start.cast::<u8>(), entry_len),
        entry_len,
        &mut *out,
    )
}

// original: report_s3_error (htslib/hfile_s3.c:1218)
pub unsafe fn hfile_s3_c_1218_report_s3_error(
    body: *mut kstring_t,
    resp_code: libc::c_long,
) -> c_int {
    let mut entry = kstring_t::default();
    // get_entry needs a NUL-terminated input string; build one from body bytes.
    let body_c = CString::new((*body).data.clone()).unwrap_or_default();

    if hfile_s3_c_1198_get_entry(
        body_c.as_ptr().cast_mut(),
        c"<Code>".as_ptr().cast_mut(),
        c"</Code>".as_ptr().cast_mut(),
        &mut entry,
    ) == libc::EOF
    {
        return -1;
    }

    let entry_c = CString::new(entry.data.clone()).unwrap_or_default();
    libc::fprintf(
        crate::htslib_rs::c_compat::stderr.cast(),
        c"hfile_s3: S3 error %ld: %s\n".as_ptr(),
        resp_code,
        entry_c.as_ptr(),
    );

    entry.data.clear();

    if hfile_s3_c_1198_get_entry(
        body_c.as_ptr().cast_mut(),
        c"<Message>".as_ptr().cast_mut(),
        c"</Message>".as_ptr().cast_mut(),
        &mut entry,
    ) == libc::EOF
    {
        return -1;
    }

    if entry.data.len() != 0 {
        let entry_c = CString::new(entry.data.clone()).unwrap_or_default();
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"%s\n".as_ptr(),
            entry_c.as_ptr(),
        );
    }

    0
}

// original: http_status_errno (htslib/hfile_s3.c:1242)
pub unsafe fn hfile_s3_c_1242_http_status_errno(status: c_int) -> c_int {
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
    } else if status >= 300 {
        libc::EIO
    } else {
        0
    }
}

// original: cleanup_local (htslib/hfile_s3.c:1276)
unsafe fn hfile_s3_cleanup_local(fp: &mut hFILE_s3) {
    crate::htslib_rs::hts::ks_free(&mut fp.buffer);
    crate::htslib_rs::hts::ks_free(&mut fp.url);
    crate::htslib_rs::hts::ks_free(&mut fp.upload_id);
    crate::htslib_rs::hts::ks_free(&mut fp.completion_message);
    crate::htslib_rs::hts::ks_free(&mut fp.content_hash);
    crate::htslib_rs::hts::ks_free(&mut fp.authorisation);
    crate::htslib_rs::hts::ks_free(&mut fp.content);
    crate::htslib_rs::hts::ks_free(&mut fp.date);
    crate::htslib_rs::hts::ks_free(&mut fp.token);
    crate::htslib_rs::hts::ks_free(&mut fp.range);
    if let Some(curl) = fp.curl.take() {
        curl_easy_cleanup(curl.as_ptr());
    }
}

// original: cleanup (htslib/hfile_s3.c:1286)
unsafe fn hfile_s3_cleanup(fp: &mut hFILE_s3) {
    if let Some(au) = fp.au.take() {
        hfile_s3_c_319_free_auth_data(au.as_ptr().cast());
    }
    hfile_s3_cleanup_local(fp);
}

unsafe fn hfile_s3_clear_authorisation_values(fp: &mut hFILE_s3) {
    crate::htslib_rs::hts::ks_clear(&mut fp.content_hash);
    crate::htslib_rs::hts::ks_clear(&mut fp.authorisation);
    crate::htslib_rs::hts::ks_clear(&mut fp.content);
    crate::htslib_rs::hts::ks_clear(&mut fp.date);
    crate::htslib_rs::hts::ks_clear(&mut fp.token);
    crate::htslib_rs::hts::ks_clear(&mut fp.range);
}

pub unsafe extern "C" fn hfile_s3_c_1293_response_callback(
    contents: *mut c_void,
    size: usize,
    nmemb: usize,
    userp: *mut c_void,
) -> usize {
    let Some(realsize) = size.checked_mul(nmemb) else {
        return 0;
    };
    if realsize == 0 {
        return 0;
    }
    if userp.is_null() {
        return 0;
    }
    let resp = userp.cast::<kstring_t>();
    if crate::htslib_rs::hts::kputsn(
        std::slice::from_raw_parts(contents.cast::<u8>(), realsize),
        realsize,
        &mut *resp,
    ) == libc::EOF
    {
        0
    } else {
        realsize
    }
}

unsafe fn hfile_s3_add_header(
    head: &mut Option<NonNull<HFileLibcurlCurlSlist>>,
    value: *const c_char,
) -> c_int {
    let tmp = match NonNull::new(curl_slist_append(head.as_ptr(), value)) {
        Some(tmp) => tmp,
        None => return 1,
    };
    *head = Some(tmp);
    0
}

unsafe fn hfile_s3_free_headers(headers: Option<NonNull<HFileLibcurlCurlSlist>>) {
    curl_slist_free_all(headers.as_ptr());
}

unsafe fn hfile_s3_set_html_headers(
    curl: NonNull<c_void>,
    auth: &kstring_t,
    date: &kstring_t,
    content: &kstring_t,
    token: &kstring_t,
    range: Option<&kstring_t>,
) -> Option<NonNull<HFileLibcurlCurlSlist>> {
    let mut headers: Option<NonNull<HFileLibcurlCurlSlist>> = None;
    let mut err = 0;

    // curl_slist_append needs NUL-terminated C strings; build temps from the
    // owned bytes at this FFI boundary.
    let auth_c = CString::new(auth.data.clone()).unwrap_or_default();
    let date_c = CString::new(date.data.clone()).unwrap_or_default();
    let content_c = CString::new(content.data.clone()).unwrap_or_default();
    let token_c = CString::new(token.data.clone()).unwrap_or_default();

    err |= hfile_s3_add_header(&mut headers, c"Content-Type:".as_ptr());
    err |= hfile_s3_add_header(&mut headers, c"Expect:".as_ptr());
    if err == 0 && auth.data.len() != 0 {
        err |= hfile_s3_add_header(&mut headers, auth_c.as_ptr());
    }
    if err == 0 {
        err |= hfile_s3_add_header(&mut headers, date_c.as_ptr());
    }
    if err == 0 && content.data.len() != 0 {
        err |= hfile_s3_add_header(&mut headers, content_c.as_ptr());
    }
    if err == 0 {
        if let Some(range) = range {
            let range_c = CString::new(range.data.clone()).unwrap_or_default();
            err |= hfile_s3_add_header(&mut headers, range_c.as_ptr());
        }
    }
    if err == 0 && token.data.len() != 0 {
        err |= hfile_s3_add_header(&mut headers, token_c.as_ptr());
    }
    if err == 0 {
        err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_HTTPHEADER, headers.as_ptr());
    }

    if err != 0 {
        hfile_s3_free_headers(headers);
        None
    } else {
        headers
    }
}

unsafe fn hfile_s3_response_code(fp: *mut hFILE_s3, response_code: *mut libc::c_long) -> c_int {
    let Some(curl) = (*fp).curl else {
        return -1;
    };
    curl_easy_getinfo_long(curl.as_ptr(), CURLINFO_RESPONSE_CODE, response_code)
}

unsafe fn hfile_s3_finish_uploaded_part(fp: *mut hFILE_s3, response: *mut kstring_t) -> c_int {
    let mut response_code: libc::c_long = 0;
    if hfile_s3_response_code(fp, &mut response_code) != CURLE_OK || response_code > 200 {
        *crate::htslib_rs::c_compat::__errno_location() =
            hfile_s3_c_1242_http_status_errno(response_code as c_int);
        -1
    } else {
        hfile_s3_append_completed_upload_part(fp, response)
    }
}

// original: get_upload_id (htslib/hfile_s3.c:1837)
unsafe fn hfile_s3_c_1837_get_upload_id(fp: *mut hFILE_s3, resp: *mut kstring_t) -> c_int {
    // get_entry needs a NUL-terminated input string; build one from resp bytes.
    let resp_c = CString::new((*resp).data.clone()).unwrap_or_default();
    if hfile_s3_c_1198_get_entry(
        resp_c.as_ptr().cast_mut(),
        c"<UploadId>".as_ptr().cast_mut(),
        c"</UploadId>".as_ptr().cast_mut(),
        &mut (*fp).upload_id,
    ) == libc::EOF
    {
        -1
    } else {
        0
    }
}

unsafe fn hfile_s3_append_completed_upload_part(fp: *mut hFILE_s3, resp: *mut kstring_t) -> c_int {
    let mut etag = kstring_t::default();
    // get_entry needs a NUL-terminated input string; build one from resp bytes.
    let resp_c = CString::new((*resp).data.clone()).unwrap_or_default();
    if hfile_s3_c_1198_get_entry(
        resp_c.as_ptr().cast_mut(),
        c"ETag: \"".as_ptr().cast_mut(),
        c"\"".as_ptr().cast_mut(),
        &mut etag,
    ) == libc::EOF
    {
        return -1;
    }

    let ret = kput_cstring(
        &mut (*fp).completion_message,
        format!(
            "\t<Part>\n\t\t<PartNumber>{}</PartNumber>\n\t\t<ETag>{}</ETag>\n\t</Part>\n",
            (*fp).part_no,
            String::from_utf8_lossy(&etag.data)
        ),
    );
    crate::htslib_rs::hts::ks_free(&mut etag);

    if ret < 0 {
        -1
    } else {
        0
    }
}

// original: abort_upload (htslib/hfile_s3.c:1417)
pub unsafe fn hfile_s3_c_1417_abort_upload(fp: *mut hFILE_s3) -> c_int {
    let mut url = kstring_t::default();
    let mut canonical_query_string = kstring_t::default();
    let mut ret = -1;
    let save_errno = *crate::htslib_rs::c_compat::__errno_location();
    let mut headers: Option<NonNull<HFileLibcurlCurlSlist>> = None;

    hfile_s3_clear_authorisation_values(&mut *fp);
    let (Some(curl), Some(au)) = ((*fp).curl, (*fp).au) else {
        goto_abort_out(
            fp,
            ret,
            save_errno,
            &mut url,
            &mut canonical_query_string,
            headers,
        );
        return ret;
    };
    if kput_cstring(
        &mut canonical_query_string,
        format!(
            "uploadId={}",
            String::from_utf8_lossy(&(*fp).upload_id.data)
        ),
    ) < 0
    {
        goto_abort_out(
            fp,
            ret,
            save_errno,
            &mut url,
            &mut canonical_query_string,
            headers,
        );
        return ret;
    }
    // v4_authorisation's cqs is a C string; build a NUL-terminated temp.
    let cqs_c = CString::new(canonical_query_string.data.clone()).unwrap_or_default();
    if hfile_s3_c_1055_v4_authorisation(
        au.as_ptr().cast(),
        c"DELETE".as_ptr().cast_mut(),
        std::ptr::null_mut(),
        cqs_c.as_ptr().cast_mut(),
        &mut (*fp).content_hash,
        &mut (*fp).authorisation,
        &mut (*fp).date,
        &mut (*fp).token,
        0,
    ) != 0
    {
        goto_abort_out(
            fp,
            ret,
            save_errno,
            &mut url,
            &mut canonical_query_string,
            headers,
        );
        return ret;
    }
    if kput_cstring(
        &mut url,
        format!(
            "{}?{}",
            String::from_utf8_lossy(&(*fp).url.data),
            String::from_utf8_lossy(&canonical_query_string.data)
        ),
    ) < 0
        || kput_cstring(
            &mut (*fp).content,
            format!(
                "x-amz-content-sha256: {}",
                String::from_utf8_lossy(&(*fp).content_hash.data)
            ),
        ) < 0
    {
        goto_abort_out(
            fp,
            ret,
            save_errno,
            &mut url,
            &mut canonical_query_string,
            headers,
        );
        return ret;
    }

    // CURLOPT_URL/USERAGENT need NUL-terminated C strings; build temps here.
    let url_c = CString::new(url.data.clone()).unwrap_or_default();
    let useragent_c = CString::new(HFILE_S3_USERAGENT.data.clone()).unwrap_or_default();
    curl_easy_reset(curl.as_ptr());
    let mut err = curl_easy_setopt(curl.as_ptr(), CURLOPT_CUSTOMREQUEST, c"DELETE".as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_USERAGENT, useragent_c.as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_URL, url_c.as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_VERBOSE, (*fp).verbose);
    if err == CURLE_OK {
        headers = hfile_s3_set_html_headers(
            curl,
            &(*fp).authorisation,
            &(*fp).date,
            &(*fp).content,
            &(*fp).token,
            None,
        );
        if headers.is_some() {
            (*fp).ret = curl_easy_perform(curl.as_ptr());
            if (*fp).ret == CURLE_OK {
                ret = 0;
            }
        }
    }
    goto_abort_out(
        fp,
        ret,
        save_errno,
        &mut url,
        &mut canonical_query_string,
        headers,
    );
    ret
}

unsafe fn goto_abort_out(
    fp: *mut hFILE_s3,
    _ret: c_int,
    save_errno: c_int,
    url: *mut kstring_t,
    canonical_query_string: *mut kstring_t,
    headers: Option<NonNull<HFileLibcurlCurlSlist>>,
) {
    crate::htslib_rs::hts::ks_free(&mut *url);
    crate::htslib_rs::hts::ks_free(&mut *canonical_query_string);
    hfile_s3_free_headers(headers);
    (*fp).aborted = 1;
    hfile_s3_cleanup(&mut *fp);
    *crate::htslib_rs::c_compat::__errno_location() = save_errno;
}

// original: complete_upload (htslib/hfile_s3.c:1479)
pub unsafe fn hfile_s3_c_1479_complete_upload(fp: *mut hFILE_s3, resp: *mut kstring_t) -> c_int {
    let mut url = kstring_t::default();
    let mut canonical_query_string = kstring_t::default();
    let mut ret = -1;
    let mut headers: Option<NonNull<HFileLibcurlCurlSlist>> = None;

    hfile_s3_clear_authorisation_values(&mut *fp);
    let (Some(curl), Some(au)) = ((*fp).curl, (*fp).au) else {
        goto_complete_out(&mut url, &mut canonical_query_string, headers);
        return -1;
    };
    if kput_cstring(
        &mut canonical_query_string,
        format!(
            "uploadId={}",
            String::from_utf8_lossy(&(*fp).upload_id.data)
        ),
    ) < 0
        || kputs_literal(
            b"</CompleteMultipartUpload>\n",
            &mut (*fp).completion_message,
        ) < 0
    {
        goto_complete_out(&mut url, &mut canonical_query_string, headers);
        return -1;
    }
    // v4_authorisation's cqs is a C string; build a NUL-terminated temp.
    let cqs_c = CString::new(canonical_query_string.data.clone()).unwrap_or_default();
    if hfile_s3_c_1055_v4_authorisation(
        au.as_ptr().cast(),
        c"POST".as_ptr().cast_mut(),
        &mut (*fp).completion_message,
        cqs_c.as_ptr().cast_mut(),
        &mut (*fp).content_hash,
        &mut (*fp).authorisation,
        &mut (*fp).date,
        &mut (*fp).token,
        0,
    ) != 0
        || kput_cstring(
            &mut url,
            format!(
                "{}?{}",
                String::from_utf8_lossy(&(*fp).url.data),
                String::from_utf8_lossy(&canonical_query_string.data)
            ),
        ) < 0
        || kput_cstring(
            &mut (*fp).content,
            format!(
                "x-amz-content-sha256: {}",
                String::from_utf8_lossy(&(*fp).content_hash.data)
            ),
        ) < 0
    {
        goto_complete_out(&mut url, &mut canonical_query_string, headers);
        return -1;
    }

    // CURLOPT_URL/USERAGENT need NUL-terminated C strings; build temps here.
    let url_c = CString::new(url.data.clone()).unwrap_or_default();
    let useragent_c = CString::new(HFILE_S3_USERAGENT.data.clone()).unwrap_or_default();
    curl_easy_reset(curl.as_ptr());
    let mut err = curl_easy_setopt(curl.as_ptr(), CURLOPT_POST, 1 as libc::c_long);
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_POSTFIELDS,
        (*fp).completion_message.data.as_ptr(),
    );
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_POSTFIELDSIZE,
        (*fp).completion_message.data.len() as libc::c_long,
    );
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_WRITEFUNCTION,
        hfile_s3_c_1293_response_callback as usize,
    );
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_WRITEDATA, resp.cast::<c_void>());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_URL, url_c.as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_USERAGENT, useragent_c.as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_VERBOSE, (*fp).verbose);
    if err == CURLE_OK {
        headers = hfile_s3_set_html_headers(
            curl,
            &(*fp).authorisation,
            &(*fp).date,
            &(*fp).content,
            &(*fp).token,
            None,
        );
        if headers.is_some() {
            (*fp).ret = curl_easy_perform(curl.as_ptr());
            if (*fp).ret == CURLE_OK {
                ret = 0;
            }
        }
    }

    goto_complete_out(&mut url, &mut canonical_query_string, headers);
    ret
}

unsafe fn goto_complete_out(
    url: *mut kstring_t,
    canonical_query_string: *mut kstring_t,
    headers: Option<NonNull<HFileLibcurlCurlSlist>>,
) {
    crate::htslib_rs::hts::ks_free(&mut *url);
    crate::htslib_rs::hts::ks_free(&mut *canonical_query_string);
    hfile_s3_free_headers(headers);
}

// original: upload_callback (htslib/hfile_s3.c:1546)
pub unsafe extern "C" fn hfile_s3_c_1546_upload_callback(
    ptr: *mut c_void,
    size: usize,
    nmemb: usize,
    stream: *mut c_void,
) -> usize {
    let Some(realsize) = size.checked_mul(nmemb) else {
        return 0;
    };
    let fp = stream.cast::<hFILE_s3>();
    let remaining = (*fp).buffer.data.len().saturating_sub((*fp).index);
    let read_length = remaining.min(realsize);
    if read_length != 0 {
        libc::memcpy(
            ptr,
            (*fp).buffer.data.as_ptr().add((*fp).index).cast(),
            read_length,
        );
        (*fp).index += read_length;
    }
    read_length
}

// original: upload_part (htslib/hfile_s3.c:1563)
pub unsafe fn hfile_s3_c_1563_upload_part(fp: *mut hFILE_s3, resp: *mut kstring_t) -> c_int {
    let mut url = kstring_t::default();
    let mut canonical_query_string = kstring_t::default();
    let mut ret = -1;
    let mut headers: Option<NonNull<HFileLibcurlCurlSlist>> = None;

    hfile_s3_clear_authorisation_values(&mut *fp);
    let (Some(curl), Some(au)) = ((*fp).curl, (*fp).au) else {
        goto_complete_out(&mut url, &mut canonical_query_string, headers);
        return -1;
    };
    if kput_cstring(
        &mut canonical_query_string,
        format!(
            "partNumber={}&uploadId={}",
            (*fp).part_no,
            String::from_utf8_lossy(&(*fp).upload_id.data)
        ),
    ) < 0
    {
        goto_complete_out(&mut url, &mut canonical_query_string, headers);
        return -1;
    }
    // v4_authorisation's cqs is a C string; build a NUL-terminated temp.
    let cqs_c = CString::new(canonical_query_string.data.clone()).unwrap_or_default();
    if hfile_s3_c_1055_v4_authorisation(
        au.as_ptr().cast(),
        c"PUT".as_ptr().cast_mut(),
        &mut (*fp).buffer,
        cqs_c.as_ptr().cast_mut(),
        &mut (*fp).content_hash,
        &mut (*fp).authorisation,
        &mut (*fp).date,
        &mut (*fp).token,
        0,
    ) != 0
        || kput_cstring(
            &mut url,
            format!(
                "{}?{}",
                String::from_utf8_lossy(&(*fp).url.data),
                String::from_utf8_lossy(&canonical_query_string.data)
            ),
        ) < 0
        || kput_cstring(
            &mut (*fp).content,
            format!(
                "x-amz-content-sha256: {}",
                String::from_utf8_lossy(&(*fp).content_hash.data)
            ),
        ) < 0
    {
        goto_complete_out(&mut url, &mut canonical_query_string, headers);
        return -1;
    }

    // CURLOPT_URL/USERAGENT need NUL-terminated C strings; build temps here.
    let url_c = CString::new(url.data.clone()).unwrap_or_default();
    let useragent_c = CString::new(HFILE_S3_USERAGENT.data.clone()).unwrap_or_default();
    (*fp).index = 0;
    curl_easy_reset(curl.as_ptr());
    let mut err = curl_easy_setopt(curl.as_ptr(), CURLOPT_UPLOAD, 1 as libc::c_long);
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_READFUNCTION,
        hfile_s3_c_1546_upload_callback as usize,
    );
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_READDATA, fp.cast::<c_void>());
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_INFILESIZE_LARGE,
        (*fp).buffer.data.len() as libc::off_t,
    );
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_HEADERFUNCTION,
        hfile_s3_c_1293_response_callback as usize,
    );
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_HEADERDATA, resp.cast::<c_void>());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_URL, url_c.as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_USERAGENT, useragent_c.as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_VERBOSE, (*fp).verbose);
    if err == CURLE_OK {
        headers = hfile_s3_set_html_headers(
            curl,
            &(*fp).authorisation,
            &(*fp).date,
            &(*fp).content,
            &(*fp).token,
            None,
        );
        if headers.is_some() {
            (*fp).ret = curl_easy_perform(curl.as_ptr());
            if (*fp).ret == CURLE_OK {
                ret = 0;
            }
        }
    }

    goto_complete_out(&mut url, &mut canonical_query_string, headers);
    ret
}

// original: recv_callback (htslib/hfile_s3.c:1854)
pub unsafe extern "C" fn hfile_s3_c_1854_recv_callback(
    ptr: *mut c_char,
    size: usize,
    nmemb: usize,
    fpv: *mut c_void,
) -> usize {
    let Some(n) = size.checked_mul(nmemb) else {
        return 0;
    };
    if n != 0 {
        let fp = fpv.cast::<hFILE_s3>();
        if crate::htslib_rs::hts::kputsn(
            std::slice::from_raw_parts(ptr.cast::<u8>(), n),
            n,
            &mut (*fp).buffer,
        ) == libc::EOF
        {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"hfile_s3: error: unable to allocate memory to read data.\n".as_ptr(),
            );
            return 0;
        }
    }
    n
}

// original: s3_read_close (htslib/hfile_s3.c:1869)
pub unsafe fn hfile_s3_c_1869_s3_read_close(fp: &mut hFILE) -> c_int {
    let HFileBackend::S3(s3) = &mut fp.backend else {
        return -1;
    };
    hfile_s3_cleanup(&mut **s3);
    0
}

// original: s3_write (htslib/hfile_s3.c:1625)
pub unsafe fn hfile_s3_c_1625_s3_write(
    fp: &mut hFILE,
    bufferv: *const c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let HFileBackend::S3(s3) = &mut fp.backend else {
        return -1;
    };
    let fp = &mut **s3 as *mut hFILE_s3;
    if crate::htslib_rs::hts::kputsn(
        std::slice::from_raw_parts(bufferv.cast::<u8>(), nbytes),
        nbytes,
        &mut (*fp).buffer,
    ) == libc::EOF
    {
        return -1;
    }

    if (*fp).buffer.data.len() > (*fp).part_size as usize {
        let mut response = kstring_t::default();
        let mut ret = hfile_s3_c_1563_upload_part(fp, &mut response);
        if ret == 0 {
            ret = hfile_s3_finish_uploaded_part(fp, &mut response);
        }
        crate::htslib_rs::hts::ks_free(&mut response);

        if ret != 0 {
            hfile_s3_c_1417_abort_upload(fp);
            return -1;
        }

        (*fp).part_no += 1;
        (*fp).buffer.data.clear();
        if (*fp).expand != 0 && (*fp).part_no % EXPAND_ON == 0 {
            (*fp).part_size *= 2;
        }
    }

    nbytes as libc::ssize_t
}

// original: s3_write_close (htslib/hfile_s3.c:1682)
pub unsafe fn hfile_s3_c_1682_s3_write_close(fp: &mut hFILE) -> c_int {
    let HFileBackend::S3(s3) = &mut fp.backend else {
        return -1;
    };
    let fp = &mut **s3 as *mut hFILE_s3;
    let mut response = kstring_t::default();
    let mut ret = 0;

    if (*fp).aborted == 0 {
        if (*fp).buffer.data.len() != 0 {
            ret = hfile_s3_c_1563_upload_part(fp, &mut response);
            if ret == 0 {
                ret = hfile_s3_finish_uploaded_part(fp, &mut response);
            }
            crate::htslib_rs::hts::ks_free(&mut response);
            response = kstring_t::default();
            if ret != 0 {
                hfile_s3_c_1417_abort_upload(fp);
                return -1;
            }
            (*fp).part_no += 1;
        }

        if (*fp).part_no > 1 {
            ret = hfile_s3_c_1479_complete_upload(fp, &mut response);
            // strstr needs a NUL-terminated input; build one from response bytes.
            let response_c = CString::new(response.data.clone()).unwrap_or_default();
            if ret == 0
                && (response.data.is_empty()
                    || libc::strstr(
                        response_c.as_ptr(),
                        c"CompleteMultipartUploadResult".as_ptr(),
                    )
                    .is_null())
            {
                ret = -1;
                let mut response_code: libc::c_long = 0;
                if hfile_s3_response_code(fp, &mut response_code) == CURLE_OK {
                    if hts_verbose >= HTS_LOG_INFO
                        && hfile_s3_c_1218_report_s3_error(&mut response, response_code) != 0
                    {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"hfile_s3: warning, unable to report full S3 error status.\n".as_ptr(),
                        );
                    }
                    *crate::htslib_rs::c_compat::__errno_location() =
                        hfile_s3_c_1242_http_status_errno(response_code as c_int);
                }
            }
        } else {
            ret = -1;
        }

        if ret != 0 {
            hfile_s3_c_1417_abort_upload(fp);
        } else {
            hfile_s3_cleanup(&mut *fp);
        }
    }

    crate::htslib_rs::hts::ks_free(&mut response);
    ret
}

unsafe fn hfile_s3_handle_bad_request(fp: *mut hFILE_s3, resp: *mut kstring_t) -> c_int {
    let mut region = kstring_t::default();
    // get_entry needs a NUL-terminated input string; build one from resp bytes.
    let resp_c = CString::new((*resp).data.clone()).unwrap_or_default();
    if hfile_s3_c_1198_get_entry(
        resp_c.as_ptr().cast_mut(),
        c"<Region>".as_ptr().cast_mut(),
        c"</Region>".as_ptr().cast_mut(),
        &mut region,
    ) == libc::EOF
    {
        return -1;
    }
    let Some(au) = (*fp).au else {
        crate::htslib_rs::hts::ks_free(&mut region);
        return -1;
    };
    let ret = hfile_s3_c_1112_set_region(au.as_ptr().cast(), &mut region);
    crate::htslib_rs::hts::ks_free(&mut region);
    ret
}

// original: initialise_upload (htslib/hfile_s3.c:1779)
pub unsafe fn hfile_s3_c_1779_initialise_upload(
    fp: *mut hFILE_s3,
    head: *mut kstring_t,
    resp: *mut kstring_t,
    user_query: c_int,
) -> c_int {
    let mut url = kstring_t::default();
    let mut ret = -1;
    let mut headers: Option<NonNull<HFileLibcurlCurlSlist>> = None;
    let delimiter = if user_query != 0 { '&' } else { '?' };

    hfile_s3_clear_authorisation_values(&mut *fp);
    let (Some(curl), Some(au)) = ((*fp).curl, (*fp).au) else {
        crate::htslib_rs::hts::ks_free(&mut url);
        return -1;
    };
    if hfile_s3_c_1055_v4_authorisation(
        au.as_ptr().cast(),
        c"POST".as_ptr().cast_mut(),
        std::ptr::null_mut(),
        c"uploads=".as_ptr().cast_mut(),
        &mut (*fp).content_hash,
        &mut (*fp).authorisation,
        &mut (*fp).date,
        &mut (*fp).token,
        user_query,
    ) != 0
        || kput_cstring(
            &mut url,
            format!(
                "{}{}uploads",
                String::from_utf8_lossy(&(*fp).url.data),
                delimiter
            ),
        ) < 0
        || kput_cstring(
            &mut (*fp).content,
            format!(
                "x-amz-content-sha256: {}",
                String::from_utf8_lossy(&(*fp).content_hash.data)
            ),
        ) < 0
    {
        crate::htslib_rs::hts::ks_free(&mut url);
        return -1;
    }

    // CURLOPT_URL/USERAGENT need NUL-terminated C strings; build temps here.
    let url_c = CString::new(url.data.clone()).unwrap_or_default();
    let useragent_c = CString::new(HFILE_S3_USERAGENT.data.clone()).unwrap_or_default();
    let mut err = curl_easy_setopt(curl.as_ptr(), CURLOPT_URL, url_c.as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_POST, 1 as libc::c_long);
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_POSTFIELDS, c"".as_ptr());
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_WRITEFUNCTION,
        hfile_s3_c_1293_response_callback as usize,
    );
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_WRITEDATA, resp.cast::<c_void>());
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_HEADERFUNCTION,
        hfile_s3_c_1293_response_callback as usize,
    );
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_HEADERDATA, head.cast::<c_void>());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_USERAGENT, useragent_c.as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_VERBOSE, (*fp).verbose);
    if err == CURLE_OK {
        headers = hfile_s3_set_html_headers(
            curl,
            &(*fp).authorisation,
            &(*fp).date,
            &(*fp).content,
            &(*fp).token,
            None,
        );
        if headers.is_some() {
            (*fp).ret = curl_easy_perform(curl.as_ptr());
            if (*fp).ret == CURLE_OK {
                ret = 0;
            }
        }
    }

    hfile_s3_free_headers(headers);
    crate::htslib_rs::hts::ks_free(&mut url);
    ret
}

pub unsafe fn hfile_s3_c_2072_s3_close(fp: &mut hFILE) -> c_int {
    let HFileBackend::S3(s3) = &fp.backend else {
        return -1;
    };
    if s3.write == 0 {
        hfile_s3_c_1869_s3_read_close(fp)
    } else {
        hfile_s3_c_1682_s3_write_close(fp)
    }
}

// original: s3_seek (htslib/hfile_s3.c:2015)
pub unsafe fn hfile_s3_c_2015_s3_seek(
    fp: &mut hFILE,
    offset: libc::off_t,
    whence: c_int,
) -> libc::off_t {
    let HFileBackend::S3(s3) = &mut fp.backend else {
        return -1;
    };
    let fp = &mut **s3 as *mut hFILE_s3;

    if (*fp).write != 0 {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
        return -1;
    }

    let origin = match whence {
        libc::SEEK_SET => 0i64,
        libc::SEEK_CUR => {
            *crate::htslib_rs::c_compat::__errno_location() = libc::ENOSYS;
            return -1;
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

    let Some(pos_i64) = origin.checked_add({
        #[cfg(windows)]
        {
            i64::from(offset)
        }
        #[cfg(not(windows))]
        {
            offset
        }
    }) else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return -1;
    };
    if pos_i64 < 0 || ((*fp).file_size >= 0 && pos_i64 > (*fp).file_size) {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }

    let pos = pos_i64 as usize;
    (*fp).keep_going = 1;

    let buffer_start = (*fp).last_read.saturating_sub((*fp).buffer.data.len());
    if pos <= (*fp).last_read && pos > buffer_start {
        (*fp).last_read_buffer = pos - buffer_start;
    } else {
        (*fp).last_read = pos;
        crate::htslib_rs::hts::ks_clear(&mut (*fp).buffer);
    }

    (*fp).last_read as libc::off_t
}

// original: s3_write_open (htslib/hfile_s3.c:2102)
pub unsafe fn hfile_s3_c_2102_s3_write_open(
    url: *const c_char,
    auth: *mut s3_auth_data,
) -> *mut hFILE {
    // The S3 subclass state is now an owned `Box<hFILE_s3>` (the payload of the
    // HFileBackend::S3 variant). Build it raw so the existing `(*fp).field`
    // accesses and the helper calls that take `*mut hFILE_s3` keep working; on
    // error we reclaim it with `Box::from_raw` (its Drop frees the kstrings).
    let fp = Box::into_raw(Box::new(hFILE_s3::default()));
    let Some(curl) = NonNull::new(curl_easy_init()) else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
        drop(Box::from_raw(fp));
        return std::ptr::null_mut();
    };
    let Some(au) = NonNull::new(auth.cast::<S3AuthDataLayout>()) else {
        curl_easy_cleanup(curl.as_ptr());
        drop(Box::from_raw(fp));
        return std::ptr::null_mut();
    };
    (*fp).curl = Some(curl);
    (*fp).au = Some(au);
    (*fp).aborted = 0;
    (*fp).part_size = MINIMUM_S3_WRITE_SIZE;
    (*fp).expand = 1;
    (*fp).write = 1;
    if let Some(env) = std::ptr::NonNull::new(libc::getenv(c"HTS_S3_PART_SIZE".as_ptr())) {
        let part_size = libc::atoi(env.as_ptr()) * 1024 * 1024;
        if part_size > (*fp).part_size {
            (*fp).part_size = part_size;
        }
        (*fp).expand = 0;
    }
    (*fp).verbose = if hts_verbose >= 8 { 1 } else { 0 };
    if crate::htslib_rs::hts::kputs(&cstr_bytes(url), &mut (*fp).url) < 0 {
        hfile_s3_cleanup_local(&mut *fp);
        drop(Box::from_raw(fp));
        return std::ptr::null_mut();
    }

    let query_start = (*fp).url.data.iter().position(|&b| b == b'?');
    let has_user_query = query_start.is_some() as c_int;
    let mut response = kstring_t::default();
    let mut header = kstring_t::default();

    if hfile_s3_c_1779_initialise_upload(fp, &mut header, &mut response, has_user_query) != 0 {
        goto_write_open_error(fp, &mut response, &mut header);
        return std::ptr::null_mut();
    }

    let mut response_code: libc::c_long = 0;
    let mut cret = hfile_s3_response_code(fp, &mut response_code);
    if cret == CURLE_OK {
        if response_code == S3_MOVED_PERMANENTLY || response_code == S3_TEMPORARY_REDIRECT {
            if hfile_s3_c_488_redirect_endpoint(
                au.as_ptr().cast(),
                response_code,
                &mut header,
                &mut (*fp).url,
            ) == 0
            {
                crate::htslib_rs::hts::ks_clear(&mut response);
                crate::htslib_rs::hts::ks_clear(&mut header);
                if hfile_s3_c_1779_initialise_upload(fp, &mut header, &mut response, has_user_query)
                    != 0
                {
                    goto_write_open_error(fp, &mut response, &mut header);
                    return std::ptr::null_mut();
                }
            }
        } else if response_code == S3_BAD_REQUEST
            && hfile_s3_handle_bad_request(fp, &mut response) == 0
        {
            crate::htslib_rs::hts::ks_clear(&mut response);
            crate::htslib_rs::hts::ks_clear(&mut header);
            if hfile_s3_c_1779_initialise_upload(fp, &mut header, &mut response, has_user_query)
                != 0
            {
                goto_write_open_error(fp, &mut response, &mut header);
                return std::ptr::null_mut();
            }
        }
        cret = hfile_s3_response_code(fp, &mut response_code);
    } else {
        goto_write_open_error(fp, &mut response, &mut header);
        return std::ptr::null_mut();
    }

    if response_code >= 300 {
        if cret == CURLE_OK {
            if hts_verbose >= HTS_LOG_INFO
                && hfile_s3_c_1218_report_s3_error(&mut response, response_code) != 0
            {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"hfile_s3: warning, unable to report full S3 error status.\n".as_ptr(),
                );
            }
            *crate::htslib_rs::c_compat::__errno_location() =
                hfile_s3_c_1242_http_status_errno(response_code as c_int);
        }
        goto_write_open_error(fp, &mut response, &mut header);
        return std::ptr::null_mut();
    }

    if hfile_s3_c_1837_get_upload_id(fp, &mut response) != 0
        || kputs_literal(
            b"<CompleteMultipartUpload>\n",
            &mut (*fp).completion_message,
        ) == libc::EOF
    {
        goto_write_open_error(fp, &mut response, &mut header);
        return std::ptr::null_mut();
    }
    (*fp).part_no = 1;
    if let Some(query_start) = query_start {
        (*fp).url.data.truncate(query_start);
    }
    crate::htslib_rs::hts::ks_free(&mut response);
    crate::htslib_rs::hts::ks_free(&mut header);

    // Reclaim the subclass Box and hand it to the owning hFILE via the
    // HFileBackend::S3 variant (replaces the old `base.backend = &S3_BACKEND`).
    let s3 = Box::from_raw(fp);
    let hfile = Box::new(hFILE {
        buffer: Vec::new(),
        begin: 0,
        end: 0,
        limit: 0,
        backend: HFileBackend::S3(s3),
        offset: 0,
        flags: 0,
        has_errno: 0,
    });
    Box::into_raw(hfile)
}

unsafe fn goto_write_open_error(
    fp: *mut hFILE_s3,
    response: *mut kstring_t,
    header: *mut kstring_t,
) {
    crate::htslib_rs::hts::ks_free(&mut *response);
    crate::htslib_rs::hts::ks_free(&mut *header);
    hfile_s3_cleanup_local(&mut *fp);
    drop(Box::from_raw(fp));
}

unsafe fn hfile_s3_hopen_vargs(
    url: *const c_char,
    mode: *const c_char,
    words: &[usize],
) -> *mut hFILE {
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

    if libc::strchr(mode, b':' as c_int).is_null() {
        let mut mode_colon = kstring_t::default();
        if crate::htslib_rs::hts::kputs(&cstr_bytes(mode), &mut mode_colon) < 0
            || crate::htslib_rs::hts::kputc(b':' as c_int, &mut mode_colon) < 0
        {
            return std::ptr::null_mut();
        }
        // hopen_vargs needs a NUL-terminated mode string; build one here.
        let mode_colon_c = CString::new(mode_colon.data.clone()).unwrap_or_default();
        hfile_c_1317_hopen_vargs(url, mode_colon_c.as_ptr(), &mut args)
    } else {
        hfile_c_1317_hopen_vargs(url, mode, &mut args)
    }
}

unsafe fn hfile_s3_c_2348_hopen_v4_read(
    url: *const c_char,
    mode: *const c_char,
    argsp: *mut crate::htslib_rs::c_compat::__va_list_tag,
    ad: *mut S3AuthDataLayout,
    http_response: *mut libc::c_long,
    fail_on_error: c_int,
) -> *mut hFILE {
    let mut words = Vec::with_capacity(if argsp.is_null() { 13 } else { 15 });
    if !argsp.is_null() {
        words.push(c"va_list".as_ptr() as usize);
        words.push(argsp as usize);
    }
    words.extend_from_slice(&[
        c"httphdr_callback".as_ptr() as usize,
        hfile_s3_c_1055_v4_auth_header_callback as usize,
        c"httphdr_callback_data".as_ptr() as usize,
        ad as usize,
        c"redirect_callback".as_ptr() as usize,
        hfile_s3_c_488_redirect_endpoint as usize,
        c"redirect_callback_data".as_ptr() as usize,
        ad as usize,
        c"http_response_ptr".as_ptr() as usize,
        http_response as usize,
        c"fail_on_error".as_ptr() as usize,
        fail_on_error as usize,
        0,
    ]);
    hfile_s3_hopen_vargs(url, mode, &words)
}

unsafe fn hfile_s3_c_774_hopen_v2_read(
    url: *const c_char,
    mode: *const c_char,
    argsp: *mut crate::htslib_rs::c_compat::__va_list_tag,
    ad: *mut s3_auth_data,
) -> *mut hFILE {
    let mut words = Vec::with_capacity(if argsp.is_null() { 9 } else { 11 });
    if !argsp.is_null() {
        words.push(c"va_list".as_ptr() as usize);
        words.push(argsp as usize);
    }
    words.extend_from_slice(&[
        c"httphdr_callback".as_ptr() as usize,
        hfile_s3_c_774_v2_authorisation as usize,
        c"httphdr_callback_data".as_ptr() as usize,
        ad as usize,
        c"redirect_callback".as_ptr() as usize,
        hfile_s3_c_488_redirect_endpoint as usize,
        c"redirect_callback_data".as_ptr() as usize,
        ad as usize,
        0,
    ]);
    hfile_s3_hopen_vargs(url, mode, &words)
}

unsafe fn hfile_s3_c_2348_hopen_v4_write(
    url: *const c_char,
    mode: *const c_char,
    argsp: *mut crate::htslib_rs::c_compat::__va_list_tag,
    ad: *mut S3AuthDataLayout,
) -> *mut hFILE {
    let _ = (mode, argsp);
    hfile_s3_c_2102_s3_write_open(url, ad.cast())
}

unsafe fn hfile_s3_c_774_s3_rewrite(
    s3url: *const c_char,
    mode: *const c_char,
    argsp: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    let mut url = kstring_t::default();
    let Some(ad) = NonNull::new(hfile_s3_c_545_setup_auth_data(s3url, mode, 2, &mut url)) else {
        return std::ptr::null_mut();
    };
    // hopen_v2_read needs a NUL-terminated url; build one from the owned bytes.
    let url_c = CString::new(url.data.clone()).unwrap_or_default();
    let fp = NonNull::new(hfile_s3_c_774_hopen_v2_read(
        url_c.as_ptr(),
        mode,
        argsp,
        ad.as_ptr(),
    ));
    let Some(fp) = fp else {
        hfile_s3_c_319_free_auth_data(ad.as_ptr());
        return std::ptr::null_mut();
    };
    fp.as_ptr()
}

// original: s3_open_v4 (htslib/hfile_s3.c:2348)
pub unsafe fn hfile_s3_c_2348_s3_open_v4(
    s3url: *const c_char,
    mode: *const c_char,
    argsp: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    let mut url = kstring_t::default();
    let Some(ad_nn) = NonNull::new(
        hfile_s3_c_545_setup_auth_data(s3url, mode, 4, &mut url).cast::<S3AuthDataLayout>(),
    ) else {
        return std::ptr::null_mut();
    };
    let ad = ad_nn.as_ptr();
    // The hopen_v4 helpers need a NUL-terminated url; build one from owned bytes.
    let url_c = CString::new(url.data.clone()).unwrap_or_default();
    let fp: Option<NonNull<hFILE>>;
    if (*ad).mode == b'r' as c_char {
        let mut http_response: libc::c_long = 0;
        let Some(first_fp) = NonNull::new(hfile_s3_c_2348_hopen_v4_read(
            url_c.as_ptr(),
            mode,
            argsp,
            ad,
            &mut http_response,
            0,
        )) else {
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        };
        if http_response == 400 {
            (*ad).refcount = 1;
            if hfile_s3_c_1055_handle_400_response(first_fp.as_ptr(), ad.cast()) != 0 {
                hclose_abruptly(first_fp.as_ptr());
                hfile_s3_c_319_free_auth_data(ad.cast());
                return std::ptr::null_mut();
            }
            hclose_abruptly(first_fp.as_ptr());
            fp = NonNull::new(hfile_s3_c_2348_hopen_v4_read(
                url_c.as_ptr(),
                mode,
                argsp,
                ad,
                std::ptr::null_mut(),
                1,
            ));
        } else if http_response > 400 {
            (*ad).refcount = 1;
            *crate::htslib_rs::c_compat::__errno_location() =
                hfile_s3_c_1242_http_status_errno(http_response as c_int);
            hclose_abruptly(first_fp.as_ptr());
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        } else {
            fp = Some(first_fp);
        }
    } else {
        fp = NonNull::new(hfile_s3_c_2348_hopen_v4_write(url_c.as_ptr(), mode, argsp, ad));
    }

    let Some(fp) = fp else {
        hfile_s3_c_319_free_auth_data(ad.cast());
        return std::ptr::null_mut();
    };
    fp.as_ptr()
}

// original: s3_open_v2 (htslib/hfile_s3.c:2374)
pub unsafe fn hfile_s3_c_2374_s3_open_v2(
    s3url: *const c_char,
    mode: *const c_char,
    argsp: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    hfile_s3_c_774_s3_rewrite(s3url, mode, argsp)
}

// original: hopen_s3 (htslib/hfile_s3.c:2400)
unsafe extern "C" fn hfile_s3_c_2400_hopen_s3(
    url: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    if libc::getenv(c"HTS_S3_V2".as_ptr()).is_null() {
        hfile_s3_c_2348_s3_open_v4(url, mode, std::ptr::null_mut())
    } else {
        hfile_s3_c_2374_s3_open_v2(url, mode, std::ptr::null_mut())
    }
}

// original: vhopen_s3 (htslib/hfile_s3.c:2414)
unsafe extern "C" fn hfile_s3_c_2414_vhopen_s3(
    url: *const c_char,
    mode_colon: *const c_char,
    args0: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    let mut args = std::mem::MaybeUninit::<crate::htslib_rs::c_compat::__va_list_tag>::uninit();
    std::ptr::copy_nonoverlapping(args0, args.as_mut_ptr(), 1);

    if libc::getenv(c"HTS_S3_V2".as_ptr()).is_null() {
        hfile_s3_c_2348_s3_open_v4(url, mode_colon, args.as_mut_ptr())
    } else {
        hfile_s3_c_2374_s3_open_v2(url, mode_colon, args.as_mut_ptr())
    }
}

// original: s3_exit (htslib/hfile_s3.c:2426)
pub unsafe extern "C" fn hfile_s3_c_2426_s3_exit() {
    HFILE_S3_USERAGENT.data = Vec::new();
}

// original: PLUGIN_GLOBAL (htslib/hfile_s3.c:2436)
pub unsafe fn hfile_s3_c_2436_PLUGIN_GLOBAL(self_: *mut hFILE_plugin) -> c_int {
    static HANDLER: hFILE_scheme_handler_layout = hFILE_scheme_handler_layout {
        open: Some(hfile_s3_c_2400_hopen_s3),
        isremote: Some(crate::htslib_rs::hfile::hfile_c_1342_hfile_always_remote),
        provider: c"Amazon S3".as_ptr(),
        priority: 2000 + 50,
        vopen: Some(hfile_s3_c_2414_vhopen_s3),
    };

    (*self_.cast::<hFILE_plugin_layout>()).name = c"Amazon S3".as_ptr();
    (*self_.cast::<hFILE_plugin_layout>()).destroy = Some(hfile_s3_c_2426_s3_exit);
    hfile_s3_c_2426_s3_exit();
    crate::htslib_rs::kstring::ksprintf(
        &mut *std::ptr::addr_of_mut!(HFILE_S3_USERAGENT),
        b"htslib/%s",
        &[crate::htslib_rs::kstring::KsPrintfArg::Str(
            crate::htslib_rs::hts::hts_version(),
        )],
    );
    hfile_add_scheme_handler(
        c"s3".as_ptr(),
        (&HANDLER as *const hFILE_scheme_handler_layout).cast::<hFILE_scheme_handler>(),
    );
    hfile_add_scheme_handler(
        c"s3+http".as_ptr(),
        (&HANDLER as *const hFILE_scheme_handler_layout).cast::<hFILE_scheme_handler>(),
    );
    hfile_add_scheme_handler(
        c"s3+https".as_ptr(),
        (&HANDLER as *const hFILE_scheme_handler_layout).cast::<hFILE_scheme_handler>(),
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    unsafe fn kstring_from_bytes(bytes: &[u8]) -> kstring_t {
        let mut s = kstring_t::default();
        crate::htslib_rs::hts::kputsn(bytes, bytes.len(), &mut s);
        s
    }

    // hFILE_s3 now embeds owned kstring_t (Vec) fields that must not be left
    // zeroed, so build a zeroed POD shell and write valid Vecs into each.
    unsafe fn zeroed_s3() -> hFILE_s3 {
        let mut fp = std::mem::MaybeUninit::<hFILE_s3>::zeroed();
        let ptr = fp.as_mut_ptr();
        std::ptr::write(std::ptr::addr_of_mut!((*ptr).buffer), kstring_t::default());
        std::ptr::write(std::ptr::addr_of_mut!((*ptr).url), kstring_t::default());
        std::ptr::write(
            std::ptr::addr_of_mut!((*ptr).content_hash),
            kstring_t::default(),
        );
        std::ptr::write(
            std::ptr::addr_of_mut!((*ptr).authorisation),
            kstring_t::default(),
        );
        std::ptr::write(std::ptr::addr_of_mut!((*ptr).content), kstring_t::default());
        std::ptr::write(std::ptr::addr_of_mut!((*ptr).date), kstring_t::default());
        std::ptr::write(std::ptr::addr_of_mut!((*ptr).token), kstring_t::default());
        std::ptr::write(std::ptr::addr_of_mut!((*ptr).range), kstring_t::default());
        std::ptr::write(
            std::ptr::addr_of_mut!((*ptr).upload_id),
            kstring_t::default(),
        );
        std::ptr::write(
            std::ptr::addr_of_mut!((*ptr).completion_message),
            kstring_t::default(),
        );
        fp.assume_init()
    }

    // The S3 backend functions now take `&mut hFILE` and recover their payload
    // from `HFileBackend::S3(Box<hFILE_s3>)`. Wrap a test `hFILE_s3` into an
    // owning `hFILE` so the dispatch entry points can be driven directly.
    fn wrap_s3(s3: hFILE_s3) -> hFILE {
        hFILE {
            buffer: Vec::new(),
            begin: 0,
            end: 0,
            limit: 0,
            backend: HFileBackend::S3(Box::new(s3)),
            offset: 0,
            flags: 0,
            has_errno: 0,
        }
    }

    // Borrow the S3 payload back out of the owning hFILE for assertions.
    fn s3_of(fp: &mut hFILE) -> &mut hFILE_s3 {
        match &mut fp.backend {
            HFileBackend::S3(s3) => s3,
            _ => unreachable!("expected S3 backend"),
        }
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn s3_crypto_helpers_match_known_sha_and_hmac_vectors() {
        unsafe {
            let mut key = kstring_from_bytes(b"key");
            let mut message = kstring_from_bytes(b"The quick brown fox jumps over the lazy dog");

            let mut sha1_digest = [0u8; DIGEST_BUFSIZ];
            let sha1_len = hfile_s3_c_142_s3_sign(sha1_digest.as_mut_ptr(), &mut key, &mut message);
            assert_eq!(sha1_len, 20);
            assert_eq!(
                hex(&sha1_digest[..sha1_len]),
                "de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9"
            );

            let mut sha256_digest = [0u8; SHA256_DIGEST_BUFSIZE];
            hfile_s3_c_152_s3_sha256(
                message.data.as_ptr().cast(),
                message.data.len(),
                sha256_digest.as_mut_ptr(),
            );
            assert_eq!(
                hex(&sha256_digest),
                "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
            );

            let mut hmac_sha256 = [0u8; SHA256_DIGEST_BUFSIZE];
            let mut hmac_len = 0;
            hfile_s3_c_157_s3_sign_sha256(
                key.data.as_ptr().cast(),
                key.data.len() as c_int,
                message.data.as_ptr().cast(),
                message.data.len() as c_int,
                hmac_sha256.as_mut_ptr(),
                &mut hmac_len,
            );
            assert_eq!(hmac_len as usize, SHA256_DIGEST_BUFSIZE);
            assert_eq!(
                hex(&hmac_sha256),
                "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
            );

            crate::htslib_rs::hts::ks_free(&mut key);
            crate::htslib_rs::hts::ks_free(&mut message);
        }
    }

    #[test]
    fn s3_setup_auth_data_builds_v4_virtual_url_and_query_state() {
        unsafe {
            let mut url = kstring_t::default();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET:TOKEN@bucket-name/path/to file.bam?b=2&a=1".as_ptr(),
                c"r".as_ptr(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());
            assert_eq!(
                url.data.as_slice(),
                b"https://bucket-name.s3.amazonaws.com/path/to%20file.bam?b=2&a=1"
            );

            let ad_layout = ad.cast::<S3AuthDataLayout>();
            assert_eq!((*ad_layout).id.data.as_slice(), b"AKID");
            assert_eq!((*ad_layout).secret.data.as_slice(), b"SECRET");
            assert_eq!((*ad_layout).token.data.as_slice(), b"TOKEN");
            assert_eq!(
                (*ad_layout).host.data.as_slice(),
                b"bucket-name.s3.amazonaws.com"
            );
            assert_eq!(
                CStr::from_ptr((*ad_layout).bucket.as_ptr()).to_bytes(),
                b"/path/to%20file.bam"
            );
            assert_eq!(
                (*ad_layout).user_query_string.data.as_slice(),
                b"b=2&a=1"
            );

            hfile_s3_c_319_free_auth_data(ad);
        }
    }

    #[test]
    fn s3_write_authorisation_callback_builds_v4_upload_headers() {
        unsafe {
            let mut url = kstring_t::default();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET:TOKEN@bucket-name/path/out.bam?z=9&a=1".as_ptr(),
                c"w".as_ptr(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());

            let mut content = kstring_from_bytes(b"abc");
            let mut hash = kstring_t::default();
            let mut auth = kstring_t::default();
            let mut date = kstring_t::default();
            let mut token = kstring_t::default();

            assert_eq!(
                hfile_s3_c_1055_v4_authorisation(
                    ad.cast(),
                    c"PUT".as_ptr().cast_mut(),
                    &mut content,
                    c"partNumber=1&uploadId=upload-1".as_ptr().cast_mut(),
                    &mut hash,
                    &mut auth,
                    &mut date,
                    &mut token,
                    1,
                ),
                0
            );

            // hash carries a trailing NUL byte from the fixed hex buffer copy.
            assert_eq!(
                CStr::from_bytes_until_nul(&hash.data).unwrap().to_bytes(),
                b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            );
            let auth_text = String::from_utf8_lossy(&auth.data);
            assert!(auth_text.starts_with("Authorization: AWS4-HMAC-SHA256 Credential=AKID/"));
            assert!(auth_text.contains("/us-east-1/s3/aws4_request"));
            assert!(auth_text.contains(
                "SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-security-token"
            ));
            assert!(auth_text.contains("Signature="));
            assert!(date.data.starts_with(b"x-amz-date: "));
            assert_eq!(token.data.as_slice(), b"x-amz-security-token: TOKEN");

            crate::htslib_rs::hts::ks_free(&mut content);
            crate::htslib_rs::hts::ks_free(&mut hash);
            crate::htslib_rs::hts::ks_free(&mut auth);
            crate::htslib_rs::hts::ks_free(&mut date);
            crate::htslib_rs::hts::ks_free(&mut token);
            hfile_s3_c_319_free_auth_data(ad);
        }
    }

    #[test]
    fn s3_v4_read_auth_header_callback_builds_sorted_query_headers() {
        unsafe {
            let mut url = kstring_t::default();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET:TOKEN@bucket-name/path/in.bam?z=9&a=1".as_ptr(),
                c"r".as_ptr(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());

            let mut hdrv: *mut *mut c_char = std::ptr::null_mut();
            assert_eq!(
                hfile_s3_c_1055_v4_auth_header_callback(ad.cast(), &mut hdrv),
                0
            );
            assert!(!hdrv.is_null());

            let auth_text = CStr::from_ptr(*hdrv).to_string_lossy();
            assert!(auth_text.starts_with("Authorization: AWS4-HMAC-SHA256 Credential=AKID/"));
            assert!(auth_text.contains("/us-east-1/s3/aws4_request"));
            assert!(auth_text.contains(
                "SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-security-token"
            ));
            assert!(auth_text.contains("Signature="));
            assert!(CStr::from_ptr(*hdrv.add(1))
                .to_bytes()
                .starts_with(b"x-amz-date: "));
            assert_eq!(
                CStr::from_ptr(*hdrv.add(2)).to_bytes(),
                b"x-amz-content-sha256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
            assert_eq!(
                CStr::from_ptr(*hdrv.add(3)).to_bytes(),
                b"X-Amz-Security-Token: TOKEN"
            );
            assert!((*hdrv.add(4)).is_null());

            let ad_layout = ad.cast::<S3AuthDataLayout>();
            assert_eq!(
                (*ad_layout).canonical_query_string.data.as_slice(),
                b"a=1&z=9"
            );

            let mut i = 0usize;
            while !(*hdrv.add(i)).is_null() {
                libc::free((*hdrv.add(i)).cast());
                *hdrv.add(i) = std::ptr::null_mut();
                i += 1;
            }
            hfile_s3_c_319_free_auth_data(ad);
        }
    }

    #[test]
    fn s3_set_region_callback_updates_auth_region_state() {
        unsafe {
            let mut url = kstring_t::default();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET@bucket-name/path/out.bam".as_ptr(),
                c"w".as_ptr(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());
            let ad_layout = ad.cast::<S3AuthDataLayout>();
            assert_eq!((*ad_layout).region.data.as_slice(), b"us-east-1");

            let mut region = kstring_from_bytes(b"eu-west-1");
            assert_eq!(hfile_s3_c_1112_set_region(ad, &mut region), 0);
            assert_eq!((*ad_layout).region.data.as_slice(), b"eu-west-1");

            crate::htslib_rs::hts::ks_free(&mut region);
            hfile_s3_c_319_free_auth_data(ad);
        }
    }

    #[test]
    fn s3_error_report_extracts_case_insensitive_xml_entries() {
        unsafe {
            let mut body = kstring_from_bytes(
                b"<?xml version=\"1.0\"?><Error><code>NoSuchKey</code><MESSAGE>missing object</MESSAGE></Error>",
            );
            assert_eq!(hfile_s3_c_1218_report_s3_error(&mut body, 404), 0);
            crate::htslib_rs::hts::ks_free(&mut body);
        }
    }

    #[test]
    fn s3_error_report_rejects_incomplete_xml_body() {
        unsafe {
            let mut body = kstring_from_bytes(b"<Error><Code>AccessDenied</Code></Error>");
            assert_eq!(hfile_s3_c_1218_report_s3_error(&mut body, 403), -1);
            crate::htslib_rs::hts::ks_free(&mut body);
        }
    }

    #[test]
    fn s3_redirect_endpoint_callback_rewrites_region_host_and_url() {
        unsafe {
            let mut setup_url = kstring_t::default();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET@bucket-name/path/in.bam?z=9&a=1".as_ptr(),
                c"r".as_ptr(),
                4,
                &mut setup_url,
            );
            assert!(!ad.is_null());

            let mut header = kstring_from_bytes(
                b"HTTP/1.1 301 Moved Permanently\r\nx-amz-bucket-region: eu-west-1\r\n\r\n",
            );
            let mut redirect_url =
                kstring_from_bytes(b"https://bucket-name.s3.amazonaws.com/path/in.bam");

            assert_eq!(
                hfile_s3_c_488_redirect_endpoint(ad.cast(), 301, &mut header, &mut redirect_url,),
                0
            );

            let ad_layout = ad.cast::<S3AuthDataLayout>();
            assert_eq!((*ad_layout).region.data.as_slice(), b"eu-west-1");
            assert_eq!(
                (*ad_layout).host.data.as_slice(),
                b"s3.eu-west-1.amazonaws.com"
            );
            assert_eq!(
                redirect_url.data.as_slice(),
                b"s3.eu-west-1.amazonaws.com/path/in.bam?z=9&a=1"
            );

            crate::htslib_rs::hts::ks_free(&mut header);
            crate::htslib_rs::hts::ks_free(&mut redirect_url);
            hfile_s3_c_319_free_auth_data(ad);
        }
    }

    #[test]
    fn s3_http_status_errno_keeps_enosys_only_for_501_not_implemented() {
        unsafe {
            assert_eq!(hfile_s3_c_1242_http_status_errno(501), libc::ENOSYS);
            assert_eq!(hfile_s3_c_1242_http_status_errno(500), libc::EIO);
            assert_eq!(hfile_s3_c_1242_http_status_errno(503), libc::EBUSY);
            assert_eq!(hfile_s3_c_1242_http_status_errno(404), libc::ENOENT);
            assert_eq!(hfile_s3_c_1242_http_status_errno(302), libc::EIO);
        }
    }

    #[test]
    fn s3_recv_callback_appends_payload_to_read_buffer() {
        unsafe {
            let mut fp = zeroed_s3();
            let mut first = *b"abc";
            let mut second = *b"defg";

            assert_eq!(
                hfile_s3_c_1854_recv_callback(
                    first.as_mut_ptr().cast(),
                    1,
                    first.len(),
                    (&mut fp as *mut hFILE_s3).cast(),
                ),
                first.len()
            );
            assert_eq!(
                hfile_s3_c_1854_recv_callback(
                    second.as_mut_ptr().cast(),
                    second.len(),
                    1,
                    (&mut fp as *mut hFILE_s3).cast(),
                ),
                second.len()
            );
            assert_eq!(fp.buffer.data.as_slice(), b"abcdefg");

            crate::htslib_rs::hts::ks_free(&mut fp.buffer);
        }
    }

    #[test]
    fn s3_response_callback_appends_payload_to_kstring() {
        unsafe {
            let mut response = kstring_t::default();
            let mut first = *b"HTTP/1.1 200 OK\r\n";
            let mut second = *b"\r\n<body/>";

            assert_eq!(
                hfile_s3_c_1293_response_callback(
                    first.as_mut_ptr().cast(),
                    1,
                    first.len(),
                    (&mut response as *mut kstring_t).cast(),
                ),
                first.len()
            );
            assert_eq!(
                hfile_s3_c_1293_response_callback(
                    second.as_mut_ptr().cast(),
                    second.len(),
                    1,
                    (&mut response as *mut kstring_t).cast(),
                ),
                second.len()
            );
            assert_eq!(
                response.data.as_slice(),
                b"HTTP/1.1 200 OK\r\n\r\n<body/>"
            );

            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_get_upload_id_extracts_multipart_id() {
        unsafe {
            let mut fp = zeroed_s3();
            let mut response = kstring_from_bytes(
                b"<?xml version=\"1.0\"?><InitiateMultipartUploadResult><UploadId>upload-123</UploadId></InitiateMultipartUploadResult>",
            );

            assert_eq!(hfile_s3_c_1837_get_upload_id(&mut fp, &mut response), 0);
            assert_eq!(fp.upload_id.data.as_slice(), b"upload-123");

            crate::htslib_rs::hts::ks_free(&mut fp.upload_id);
            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_get_upload_id_rejects_missing_id() {
        unsafe {
            let mut fp = zeroed_s3();
            let mut response = kstring_from_bytes(b"<Error><Code>NoSuchUpload</Code></Error>");

            assert_eq!(hfile_s3_c_1837_get_upload_id(&mut fp, &mut response), -1);
            assert!(fp.upload_id.data.is_empty());

            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_append_completed_upload_part_records_etag_and_part_number() {
        unsafe {
            let mut fp = zeroed_s3();
            fp.part_no = 7;
            let mut response = kstring_from_bytes(b"HTTP/1.1 200 OK\r\nEtag: \"etag-123\"\r\n\r\n");

            assert_eq!(
                hfile_s3_append_completed_upload_part(&mut fp, &mut response),
                0
            );
            assert_eq!(
                fp.completion_message.data.as_slice(),
                b"\t<Part>\n\t\t<PartNumber>7</PartNumber>\n\t\t<ETag>etag-123</ETag>\n\t</Part>\n"
            );

            crate::htslib_rs::hts::ks_free(&mut fp.completion_message);
            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_append_completed_upload_part_rejects_missing_etag() {
        unsafe {
            let mut fp = zeroed_s3();
            fp.part_no = 1;
            let mut response = kstring_from_bytes(b"HTTP/1.1 200 OK\r\n\r\n");

            assert_eq!(
                hfile_s3_append_completed_upload_part(&mut fp, &mut response),
                -1
            );
            assert!(fp.completion_message.data.is_empty());

            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_upload_callback_streams_buffer_and_advances_index() {
        unsafe {
            let mut fp = zeroed_s3();
            fp.buffer = kstring_from_bytes(b"abcdef");
            let mut out = [0u8; 4];

            assert_eq!(
                hfile_s3_c_1546_upload_callback(
                    out.as_mut_ptr().cast(),
                    1,
                    out.len(),
                    (&mut fp as *mut hFILE_s3).cast(),
                ),
                4
            );
            assert_eq!(&out, b"abcd");
            assert_eq!(fp.index, 4);

            let mut tail = [0u8; 4];
            assert_eq!(
                hfile_s3_c_1546_upload_callback(
                    tail.as_mut_ptr().cast(),
                    1,
                    tail.len(),
                    (&mut fp as *mut hFILE_s3).cast(),
                ),
                2
            );
            assert_eq!(&tail[..2], b"ef");
            assert_eq!(fp.index, 6);

            crate::htslib_rs::hts::ks_free(&mut fp.buffer);
        }
    }

    #[test]
    fn s3_write_buffers_until_part_threshold_without_network() {
        unsafe {
            let mut fp = zeroed_s3();
            fp.part_size = 10;
            fp.part_no = 1;
            fp.expand = 1;
            fp.write = 1;
            let mut hf = wrap_s3(fp);
            let payload = *b"abcdef";

            assert_eq!(
                hfile_s3_c_1625_s3_write(&mut hf, payload.as_ptr().cast(), payload.len(),),
                payload.len() as libc::ssize_t
            );
            let fp = s3_of(&mut hf);
            assert_eq!(fp.buffer.data.as_slice(), b"abcdef");
            assert_eq!(fp.part_no, 1);

            crate::htslib_rs::hts::ks_free(&mut fp.buffer);
        }
    }

    #[test]
    fn s3_write_close_aborts_unstarted_upload_without_curl() {
        unsafe {
            let mut fp = zeroed_s3();
            fp.write = 1;
            fp.part_no = 1;
            fp.buffer = kstring_from_bytes(b"");
            let mut hf = wrap_s3(fp);

            assert_eq!(hfile_s3_c_1682_s3_write_close(&mut hf), -1);
            let fp = s3_of(&mut hf);
            assert_eq!(fp.aborted, 1);
            assert!(fp.buffer.data.is_empty());
        }
    }

    #[test]
    fn s3_read_close_releases_read_state_and_auth_refcount() {
        unsafe {
            let mut url = kstring_t::default();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET@bucket-name/path/in.bam".as_ptr(),
                c"r".as_ptr(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());
            let ad_layout = ad.cast::<S3AuthDataLayout>();
            (*ad_layout).refcount = 1;

            let mut fp = zeroed_s3();
            fp.au = NonNull::new(ad_layout);
            fp.buffer = kstring_from_bytes(b"buffer");
            fp.url = kstring_from_bytes(b"https://bucket-name.s3.amazonaws.com/path/in.bam");
            fp.upload_id = kstring_from_bytes(b"upload");
            fp.completion_message = kstring_from_bytes(b"complete");
            fp.content_hash = kstring_from_bytes(b"hash");
            fp.authorisation = kstring_from_bytes(b"auth");
            fp.content = kstring_from_bytes(b"content");
            fp.date = kstring_from_bytes(b"date");
            fp.token = kstring_from_bytes(b"token");
            fp.range = kstring_from_bytes(b"range");
            let mut hf = wrap_s3(fp);

            assert_eq!(hfile_s3_c_1869_s3_read_close(&mut hf), 0);
            let fp = s3_of(&mut hf);
            assert!(fp.au.is_null());
            assert!(fp.buffer.data.is_empty());
            assert!(fp.url.data.is_empty());
            assert!(fp.upload_id.data.is_empty());
            assert!(fp.completion_message.data.is_empty());
            assert!(fp.content_hash.data.is_empty());
            assert!(fp.authorisation.data.is_empty());
            assert!(fp.content.data.is_empty());
            assert!(fp.date.data.is_empty());
            assert!(fp.token.data.is_empty());
            assert!(fp.range.data.is_empty());
            assert_eq!((*ad_layout).refcount, 0);

            hfile_s3_c_319_free_auth_data(ad);
        }
    }

    #[test]
    fn s3_seek_repositions_within_buffer_or_resets_remote_position() {
        unsafe {
            let mut fp = zeroed_s3();
            fp.buffer = kstring_from_bytes(b"abcdefghij");
            fp.last_read = 110;
            fp.last_read_buffer = 10;
            fp.file_size = 500;
            fp.keep_going = 0;
            let mut hf = wrap_s3(fp);

            assert_eq!(hfile_s3_c_2015_s3_seek(&mut hf, 105, libc::SEEK_SET), 110);
            {
                let fp = s3_of(&mut hf);
                assert_eq!(fp.last_read, 110);
                assert_eq!(fp.last_read_buffer, 5);
                assert_eq!(fp.buffer.data.len(), 10);
                assert_eq!(fp.keep_going, 1);
            }

            assert_eq!(hfile_s3_c_2015_s3_seek(&mut hf, 120, libc::SEEK_SET), 120);
            let fp = s3_of(&mut hf);
            assert_eq!(fp.last_read, 120);
            assert_eq!(fp.buffer.data.len(), 0);

            crate::htslib_rs::hts::ks_free(&mut fp.buffer);
        }
    }

    #[test]
    fn s3_seek_rejects_write_and_invalid_origins_like_upstream() {
        unsafe {
            let mut fp = zeroed_s3();
            fp.file_size = 200;
            fp.write = 1;
            let mut hf = wrap_s3(fp);

            assert_eq!(hfile_s3_c_2015_s3_seek(&mut hf, 0, libc::SEEK_SET), -1);
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::ESPIPE
            );

            s3_of(&mut hf).write = 0;
            assert_eq!(hfile_s3_c_2015_s3_seek(&mut hf, 0, libc::SEEK_CUR), -1);
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::ENOSYS
            );

            assert_eq!(hfile_s3_c_2015_s3_seek(&mut hf, 1, libc::SEEK_END), -1);
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );

            s3_of(&mut hf).file_size = -1;
            assert_eq!(hfile_s3_c_2015_s3_seek(&mut hf, 0, libc::SEEK_END), -1);
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::ESPIPE
            );
        }
    }

    #[test]
    fn s3_plugin_sets_destroy_callback_and_exit_releases_useragent() {
        unsafe {
            hfile_s3_c_2426_s3_exit();
            let mut plugin = hFILE_plugin_layout {
                api_version: 1,
                obj: None,
                name: std::ptr::null(),
                destroy: None,
            };

            assert_eq!(
                hfile_s3_c_2436_PLUGIN_GLOBAL(
                    (&mut plugin as *mut hFILE_plugin_layout).cast::<hFILE_plugin>(),
                ),
                0
            );
            assert_eq!(CStr::from_ptr(plugin.name).to_bytes(), b"Amazon S3");
            assert!(plugin.destroy.is_some());
            assert!(!(*std::ptr::addr_of!(HFILE_S3_USERAGENT.data)).is_empty());
            assert!((*std::ptr::addr_of!(HFILE_S3_USERAGENT.data)).starts_with(b"htslib/"));

            let destroy = plugin.destroy.expect("S3 destroy callback");
            destroy();
            assert!((*std::ptr::addr_of!(HFILE_S3_USERAGENT.data)).is_empty());

            hfile_s3_c_2426_s3_exit();
            assert!((*std::ptr::addr_of!(HFILE_S3_USERAGENT.data)).is_empty());
        }
    }

    #[test]
    fn s3_utc_date_formatters_match_expected_wire_formats() {
        let now: libc::time_t = 1_748_868_896;
        let mut date = [0 as c_char; 40];
        let mut date_long = [0 as c_char; 17];
        let mut date_short = [0 as c_char; 9];

        write_s3_date_header(&mut date, now);
        assert_eq!(
            unsafe { CStr::from_ptr(date.as_ptr()) }.to_bytes(),
            b"Date: Mon, 02 Jun 2025 12:54:56 GMT"
        );

        assert!(write_s3_v4_dates(&mut date_long, &mut date_short, now));
        assert_eq!(
            unsafe { CStr::from_ptr(date_long.as_ptr()) }.to_bytes(),
            b"20250602T125456Z"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(date_short.as_ptr()) }.to_bytes(),
            b"20250602"
        );
    }
}
