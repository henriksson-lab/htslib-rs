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
        hfile_c_1317_hopen_vargs, hfile_destroy, hfile_init, htslib_hfile_h_247_hread,
    },
    hts::{hFILE, hts_verbose, kstring_t},
};
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void, CStr, CString};

const AUTH_LIFETIME: libc::time_t = 60;
const CREDENTIAL_LIFETIME: libc::time_t = 60;
const DIGEST_BUFSIZ: usize = 64;
const SHA256_DIGEST_BUFSIZE: usize = 32;
const HASH_LENGTH_SHA256: usize = SHA256_DIGEST_BUFSIZE * 2 + 1;
const MINIMUM_S3_WRITE_SIZE: c_int = 5_242_880;
const EXPAND_ON: c_int = 1112;
const S3_MOVED_PERMANENTLY: libc::c_long = 301;

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

type HFileReadFn = unsafe extern "C" fn(*mut hFILE, *mut c_void, usize) -> libc::ssize_t;
type HFileWriteFn = unsafe extern "C" fn(*mut hFILE, *const c_void, usize) -> libc::ssize_t;
type HFileSeekFn = unsafe extern "C" fn(*mut hFILE, libc::off_t, c_int) -> libc::off_t;
type HFileFlushFn = unsafe extern "C" fn(*mut hFILE) -> c_int;
type HFileCloseFn = unsafe extern "C" fn(*mut hFILE) -> c_int;
type HFileOpenFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut hFILE;
type HFileIsRemoteFn = unsafe extern "C" fn(*const c_char) -> c_int;
type HFileVOpenFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE;
type HFilePluginDestroyFn = unsafe extern "C" fn();

unsafe fn hfile_plugin_destroy_fn(ptr: *const c_void) -> HFilePluginDestroyFn {
    debug_assert!(!ptr.is_null());
    std::mem::transmute_copy(&ptr)
}

#[repr(C)]
struct hFILE_backend {
    read: Option<HFileReadFn>,
    write: Option<HFileWriteFn>,
    seek: Option<HFileSeekFn>,
    flush: Option<HFileFlushFn>,
    close: Option<HFileCloseFn>,
}

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
    obj: *mut c_void,
    name: *const c_char,
    destroy: *const c_void,
}

#[repr(C)]
struct HFileLayout {
    buffer: *mut c_char,
    begin: *mut c_char,
    end: *mut c_char,
    limit: *mut c_char,
    backend: *const hFILE_backend,
    offset: libc::off_t,
    flags: c_uint,
    has_errno: c_int,
}

#[repr(C)]
struct HFileLibcurlCurlSlist {
    data: *mut c_char,
    next: *mut HFileLibcurlCurlSlist,
}

#[repr(C)]
pub struct hFILE_s3 {
    base: HFileLayout,
    curl: *mut c_void,
    ret: c_int,
    au: *mut S3AuthDataLayout,
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

// Concurrency note (audit 2026-05):
//
// `HFILE_S3_USERAGENT` is a `static mut kstring_t` that mirrors the file-scope
// `useragent` in `htslib/hfile_s3.c`. It is initialized exactly once inside
// `hfile_s3_c_2436_PLUGIN_GLOBAL`, which is dispatched from
// `hfile_c_1111_load_hfile_plugins` under the `hfile_plugin_state` mutex
// (see `src/hfile.rs`). After init the `s`/`l`/`m` fields are not mutated
// (only the immutable `.s` pointer is read via `CURLOPT_USERAGENT`) until
// `hfile_s3_c_2426_s3_exit` runs as the plugin destroy callback at process
// shutdown.
//
// SAFETY: init-once-then-read, protected by the plugin-load mutex. Read
// sites only consume `HFILE_S3_USERAGENT.s` (a `*const c_char`) by value and
// hand it to libcurl, which performs its own internal synchronization for
// per-easy-handle option storage.
static mut HFILE_S3_USERAGENT: kstring_t = kstring_t {
    l: 0,
    m: 0,
    s: std::ptr::null_mut(),
};

// original: s3_auth_data (htslib/hfile_s3.c:48)
#[repr(C)]
pub struct s3_auth_data {
    id: kstring_t,
    token: kstring_t,
    secret: kstring_t,
    region: kstring_t,
    canonical_query_string: kstring_t,
    user_query_string: kstring_t,
    host: kstring_t,
    profile: kstring_t,
    creds_expiry_time: libc::time_t,
    bucket: *mut c_char,
    auth_hdr: kstring_t,
    auth_time: libc::time_t,
    date: [c_char; 40],
    date_long: [c_char; 17],
    date_short: [c_char; 9],
    date_html: kstring_t,
    mode: c_char,
    headers: [*mut c_char; 5],
    refcount: c_int,
}

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
        Ok(cstr) => crate::htslib_rs::hts::kputs(cstr.as_ptr(), s),
        Err(_) => -1,
    }
}

unsafe fn kputs_literal(text: &[u8], s: *mut kstring_t) -> c_int {
    crate::htslib_rs::hts::kputsn(text.as_ptr().cast(), text.len(), s)
}

unsafe fn ks_release_or_free(s: &mut kstring_t) -> *mut c_char {
    if s.s.is_null() {
        std::ptr::null_mut()
    } else {
        crate::htslib_rs::hts::ks_release(s)
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
        (*key).s.cast(),
        (*key).l as c_int,
        (*message).s.cast(),
        (*message).l,
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
pub unsafe fn hfile_s3_c_165_urldecode_kput(s: *const c_char, len: c_int, str_: *mut kstring_t) {
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
pub unsafe fn hfile_s3_c_181_base64_kput(data: *const c_uchar, len: usize, str_: *mut kstring_t) {
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

    (*str_).l -= pad;
    crate::htslib_rs::hts::kputsn(c"==".as_ptr(), pad, str_);
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
        let mut full_fname: kstring_t = std::mem::zeroed();
        let home = libc::getenv(c"HOME".as_ptr());
        if home.is_null() {
            return std::ptr::null_mut();
        }

        crate::htslib_rs::hts::kputs(home, &mut full_fname);
        crate::htslib_rs::hts::kputs(fname.add(1), &mut full_fname);

        let fp = libc::fopen(full_fname.s, mode);
        libc::free(full_fname.s.cast());
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
    let mut line: kstring_t = std::mem::zeroed();
    let mut active = 1;
    let fp = hfile_s3_c_231_expand_tilde_open(fname, c"r".as_ptr());
    if fp.is_null() {
        return;
    }

    while {
        line.l = 0;
        crate::htslib_rs::hts::kfgetline(&mut line, fp) >= 0
    } {
        if !line.s.is_null() && *line.s == b'[' as c_char {
            let s = libc::strchr(line.s, b']' as c_int);
            if !s.is_null() {
                *s = 0;
                active = (libc::strcmp(line.s.add(1), section) == 0) as c_int;
            }
        } else if active != 0 {
            let s = libc::strpbrk(line.s, c":=".as_ptr());
            if !s.is_null() {
                let mut key = line.s;
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
                while line.l > 0 && libc::isspace(*line.s.add(line.l - 1) as c_uchar as c_int) != 0
                {
                    line.l -= 1;
                    *line.s.add(line.l) = 0;
                }

                for &(akey, avar) in pairs {
                    if libc::strcmp(key, akey) == 0 {
                        (*avar).l = 0;
                        crate::htslib_rs::hts::kputs(value, avar);
                        break;
                    }
                }
            }
        }
    }

    libc::fclose(fp);
    libc::free(line.s.cast());
}

// original: parse_simple (htslib/hfile_s3.c:294)
pub unsafe fn hfile_s3_c_294_parse_simple(
    fname: *const c_char,
    id: *mut kstring_t,
    secret: *mut kstring_t,
) {
    let mut text: kstring_t = std::mem::zeroed();
    let fp = hfile_s3_c_231_expand_tilde_open(fname, c"r".as_ptr());
    if fp.is_null() {
        return;
    }

    while crate::htslib_rs::hts::kfgetline(&mut text, fp) >= 0 {
        crate::htslib_rs::hts::kputc(b' ' as c_int, &mut text);
    }
    libc::fclose(fp);

    let mut s = text.s;
    while libc::isspace(*s as c_uchar as c_int) != 0 {
        s = s.add(1);
    }
    let len = libc::strcspn(s, c" \t".as_ptr());
    crate::htslib_rs::hts::kputsn(s, len, id);

    s = s.add(len);
    while libc::isspace(*s as c_uchar as c_int) != 0 {
        s = s.add(1);
    }
    crate::htslib_rs::hts::kputsn(s, libc::strcspn(s, c" \t".as_ptr()), secret);

    libc::free(text.s.cast());
}

unsafe fn hfile_s3_copy_auth_headers(
    ad: *mut S3AuthDataLayout,
    hdrs: *mut *mut *mut c_char,
) -> c_int {
    let mut idx = 0usize;
    *hdrs = (*ad).headers.as_mut_ptr();

    (*ad).headers[idx] = libc::strdup((*ad).date.as_ptr());
    if (*ad).headers[idx].is_null() {
        return -1;
    }
    idx += 1;

    if (*ad).token.l != 0 {
        let mut token_hdr: kstring_t = std::mem::zeroed();
        crate::htslib_rs::hts::kputs(c"X-Amz-Security-Token: ".as_ptr(), &mut token_hdr);
        crate::htslib_rs::hts::kputs((*ad).token.s, &mut token_hdr);
        if token_hdr.s.is_null() {
            while idx > 0 {
                idx -= 1;
                libc::free((*ad).headers[idx].cast());
            }
            return -1;
        }
        (*ad).headers[idx] = token_hdr.s;
        idx += 1;
    }

    if (*ad).auth_hdr.l != 0 {
        (*ad).headers[idx] = libc::strdup((*ad).auth_hdr.s);
        if (*ad).headers[idx].is_null() {
            while idx > 0 {
                idx -= 1;
                libc::free((*ad).headers[idx].cast());
            }
            return -1;
        }
        idx += 1;
    }

    (*ad).headers[idx] = std::ptr::null_mut();
    0
}

#[repr(C)]
struct S3AuthDataLayout {
    id: kstring_t,
    token: kstring_t,
    secret: kstring_t,
    region: kstring_t,
    canonical_query_string: kstring_t,
    user_query_string: kstring_t,
    host: kstring_t,
    profile: kstring_t,
    creds_expiry_time: libc::time_t,
    bucket: *mut c_char,
    auth_hdr: kstring_t,
    auth_time: libc::time_t,
    date: [c_char; 40],
    date_long: [c_char; 17],
    date_short: [c_char; 9],
    date_html: kstring_t,
    mode: c_char,
    headers: [*mut c_char; 5],
    refcount: c_int,
}

// original: free_auth_data (htslib/hfile_s3.c:319)
pub unsafe fn hfile_s3_c_319_free_auth_data(ad: *mut s3_auth_data) {
    let ad = ad.cast::<S3AuthDataLayout>();
    if (*ad).refcount > 0 {
        (*ad).refcount -= 1;
        return;
    }
    libc::free((*ad).profile.s.cast());
    libc::free((*ad).id.s.cast());
    libc::free((*ad).token.s.cast());
    libc::free((*ad).secret.s.cast());
    libc::free((*ad).region.s.cast());
    libc::free((*ad).canonical_query_string.s.cast());
    libc::free((*ad).user_query_string.s.cast());
    libc::free((*ad).host.s.cast());
    libc::free((*ad).bucket.cast());
    libc::free((*ad).auth_hdr.s.cast());
    libc::free((*ad).date_html.s.cast());
    libc::free(ad.cast());
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

    if (*datetime).s.is_null() {
        return 0;
    }

    let num = libc::sscanf(
        (*datetime).s,
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
    let mut expiry_time: kstring_t = std::mem::zeroed();
    hfile_s3_c_252_parse_ini(
        if v.is_null() {
            c"~/.aws/credentials".as_ptr()
        } else {
            v
        },
        (*ad).profile.s,
        &[
            (c"aws_access_key_id".as_ptr(), &mut (*ad).id),
            (c"aws_secret_access_key".as_ptr(), &mut (*ad).secret),
            (c"aws_session_token".as_ptr(), &mut (*ad).token),
            (c"expiry_time".as_ptr(), &mut expiry_time),
        ],
    );
    if expiry_time.l != 0 {
        (*ad).creds_expiry_time = hfile_s3_c_333_parse_rfc3339_date(&mut expiry_time);
    }
    crate::htslib_rs::hts::ks_free(&mut expiry_time);
}

// original: escape_query (htslib/hfile_s3.c:396)
pub unsafe fn hfile_s3_c_396_escape_query(qs: *const c_char) -> *mut c_char {
    let length = libc::strlen(qs);
    let alloced = length * 3 + 1;
    let escaped = libc::malloc(alloced).cast::<c_char>();
    if escaped.is_null() {
        return std::ptr::null_mut();
    }

    let mut j = 0usize;
    for i in 0..length {
        let c = *qs.add(i) as u8;
        if c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'~' | b'.' | b'/' | b'=' | b'&')
        {
            *escaped.add(j) = c as c_char;
            j += 1;
        } else {
            libc::snprintf(
                escaped.add(j),
                alloced - j,
                c"%%%02X".as_ptr(),
                *qs.add(i) as c_int,
            );
            j += 3;
        }
    }

    *escaped.add(j) = 0;
    escaped
}

// original: escape_path (htslib/hfile_s3.c:424)
pub unsafe fn hfile_s3_c_424_escape_path(path: *const c_char) -> *mut c_char {
    let length = libc::strlen(path);
    let alloced = length * 3 + 1;
    let escaped = libc::malloc(alloced).cast::<c_char>();
    if escaped.is_null() {
        return std::ptr::null_mut();
    }

    let mut j = 0usize;
    let mut i = 0usize;
    while i < length {
        let c = *path.add(i) as u8;
        if c == b'?' {
            break;
        }

        if c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'~' | b'.' | b'/') {
            *escaped.add(j) = c as c_char;
            j += 1;
        } else {
            libc::snprintf(
                escaped.add(j),
                alloced - j,
                c"%%%02X".as_ptr(),
                *path.add(i) as c_int,
            );
            j += 3;
        }
        i += 1;
    }

    if i != length {
        libc::strcpy(escaped.add(j), path.add(i));
    } else {
        *escaped.add(j) = 0;
    }
    escaped
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
    if header.is_null() || (*header).s.is_null() {
        return ret;
    }

    let mut new_region = libc::strstr((*header).s, c"x-amz-bucket-region: ".as_ptr());
    if !new_region.is_null() {
        new_region = new_region.add(c"x-amz-bucket-region: ".to_bytes().len());
        let mut end = new_region;
        while libc::isalnum(*end as c_uchar as c_int) != 0
            || libc::ispunct(*end as c_uchar as c_int) != 0
        {
            end = end.add(1);
        }
        *end = 0;

        if libc::strstr((*ad).host.s, c"amazonaws.com".as_ptr()).is_null() {
            return ret;
        }
        (*ad).region.l = 0;
        crate::htslib_rs::hts::kputs(new_region, &mut (*ad).region);
        (*ad).host.l = 0;
        if kput_cstring(
            &mut (*ad).host,
            format!(
                "s3.{}.amazonaws.com",
                CStr::from_ptr(new_region).to_string_lossy()
            ),
        ) < 0
        {
            return ret;
        }

        if (*ad).region.l != 0 && (*ad).host.l != 0 {
            (*url).l = 0;
            crate::htslib_rs::hts::kputs((*ad).host.s, url);
            crate::htslib_rs::hts::kputsn((*ad).bucket, libc::strlen((*ad).bucket), url);
            if (*ad).user_query_string.l != 0 {
                crate::htslib_rs::hts::kputc(b'?' as c_int, url);
                crate::htslib_rs::hts::kputsn(
                    (*ad).user_query_string.s,
                    (*ad).user_query_string.l,
                    url,
                );
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
    let ad = libc::calloc(1, std::mem::size_of::<S3AuthDataLayout>()).cast::<S3AuthDataLayout>();
    if ad.is_null() {
        return std::ptr::null_mut();
    }
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
            libc::free(ad.cast());
            return std::ptr::null_mut();
        }
        bucket = bucket.add(1);
        crate::htslib_rs::hts::kputsn(s3url.add(3), bucket.offset_from(s3url.add(3)) as usize, url);
        is_https = (libc::strncmp((*url).s, c"https:".as_ptr(), 6) == 0) as c_int;
    } else {
        crate::htslib_rs::hts::kputs(c"https:".as_ptr(), url);
        bucket = s3url.add(3);
    }
    while *bucket == b'/' as c_char {
        crate::htslib_rs::hts::kputc(*bucket as c_int, url);
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
            crate::htslib_rs::hts::kputs(v, &mut (*ad).id);
        }
        v = libc::getenv(c"AWS_SECRET_ACCESS_KEY".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(v, &mut (*ad).secret);
        }
        v = libc::getenv(c"AWS_SESSION_TOKEN".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(v, &mut (*ad).token);
        }
        v = libc::getenv(c"AWS_DEFAULT_REGION".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(v, &mut (*ad).region);
        }
        v = libc::getenv(c"HTS_S3_HOST".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(v, &mut (*ad).host);
        }
        v = libc::getenv(c"AWS_DEFAULT_PROFILE".as_ptr());
        if v.is_null() {
            v = libc::getenv(c"AWS_PROFILE".as_ptr());
        }
        if v.is_null() {
            v = c"default".as_ptr().cast_mut();
        }
        crate::htslib_rs::hts::kputs(v, &mut (*ad).profile);
        v = libc::getenv(c"HTS_S3_ADDRESS_STYLE".as_ptr());
        if !v.is_null() {
            if libc::strcasecmp(v, c"virtual".as_ptr()) == 0 {
                address_style = 1;
            } else if libc::strcasecmp(v, c"path".as_ptr()) == 0 {
                address_style = 2;
            }
        }
    }

    if (*ad).id.l == 0 {
        let mut url_style: kstring_t = std::mem::zeroed();
        let mut expiry_time: kstring_t = std::mem::zeroed();
        let v = libc::getenv(c"AWS_SHARED_CREDENTIALS_FILE".as_ptr());
        hfile_s3_c_252_parse_ini(
            if v.is_null() {
                c"~/.aws/credentials".as_ptr()
            } else {
                v
            },
            (*ad).profile.s,
            &[
                (c"aws_access_key_id".as_ptr(), &mut (*ad).id),
                (c"aws_secret_access_key".as_ptr(), &mut (*ad).secret),
                (c"aws_session_token".as_ptr(), &mut (*ad).token),
                (c"region".as_ptr(), &mut (*ad).region),
                (c"addressing_style".as_ptr(), &mut url_style),
                (c"expiry_time".as_ptr(), &mut expiry_time),
            ],
        );
        if url_style.l != 0 {
            if libc::strcmp(url_style.s, c"virtual".as_ptr()) == 0 {
                address_style = 1;
            } else if libc::strcmp(url_style.s, c"path".as_ptr()) == 0 {
                address_style = 2;
            } else {
                address_style = 0;
            }
        }
        if expiry_time.l != 0 {
            (*ad).creds_expiry_time = hfile_s3_c_333_parse_rfc3339_date(&mut expiry_time);
        }
        crate::htslib_rs::hts::ks_free(&mut url_style);
        crate::htslib_rs::hts::ks_free(&mut expiry_time);
    }

    if (*ad).id.l == 0 {
        let mut url_style: kstring_t = std::mem::zeroed();
        let v = libc::getenv(c"HTS_S3_S3CFG".as_ptr());
        hfile_s3_c_252_parse_ini(
            if v.is_null() { c"~/.s3cfg".as_ptr() } else { v },
            (*ad).profile.s,
            &[
                (c"access_key".as_ptr(), &mut (*ad).id),
                (c"secret_key".as_ptr(), &mut (*ad).secret),
                (c"access_token".as_ptr(), &mut (*ad).token),
                (c"host_base".as_ptr(), &mut (*ad).host),
                (c"bucket_location".as_ptr(), &mut (*ad).region),
                (c"host_bucket".as_ptr(), &mut url_style),
            ],
        );
        if url_style.l != 0 {
            address_style = if libc::strstr(url_style.s, c"%(bucket)s".as_ptr()).is_null() {
                2
            } else {
                0
            };
        }
        crate::htslib_rs::hts::ks_free(&mut url_style);
    }

    if (*ad).id.l == 0 {
        hfile_s3_c_294_parse_simple(c"~/.awssecret".as_ptr(), &mut (*ad).id, &mut (*ad).secret);
    }

    let dns_compliant = match address_style {
        1 => 1,
        2 => 0,
        _ => hfile_s3_c_206_is_dns_compliant(bucket, path, is_https),
    };
    if (*ad).host.l == 0 {
        crate::htslib_rs::hts::kputs(c"s3.amazonaws.com".as_ptr(), &mut (*ad).host);
    }
    if dns_compliant == 0
        && (*ad).region.l > 0
        && libc::strcmp((*ad).host.s, c"s3.amazonaws.com".as_ptr()) == 0
    {
        (*ad).host.l = 0;
        if kput_cstring(
            &mut (*ad).host,
            format!(
                "s3.{}.amazonaws.com",
                CStr::from_ptr((*ad).region.s).to_string_lossy()
            ),
        ) < 0
        {
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        }
    }
    if (*ad).region.l == 0 {
        crate::htslib_rs::hts::kputs(c"us-east-1".as_ptr(), &mut (*ad).region);
    }

    let mut escaped: *mut c_char = std::ptr::null_mut();
    if hfile_s3_c_460_is_escaped(path) == 0 {
        escaped = hfile_s3_c_424_escape_path(path);
        if escaped.is_null() {
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        }
    }

    let bucket_len = path.offset_from(bucket) as usize;
    let url_path_pos: usize;
    if dns_compliant != 0 {
        let url_host_pos = (*url).l;
        crate::htslib_rs::hts::kputsn_(bucket.cast(), bucket_len, url);
        crate::htslib_rs::hts::kputc(b'.' as c_int, url);
        crate::htslib_rs::hts::kputsn((*ad).host.s, (*ad).host.l, url);
        url_path_pos = (*url).l;
        if sigver == 4 {
            (*ad).host.l = 0;
            crate::htslib_rs::hts::kputsn(
                (*url).s.add(url_host_pos),
                (*url).l - url_host_pos,
                &mut (*ad).host,
            );
        }
    } else {
        crate::htslib_rs::hts::kputsn((*ad).host.s, (*ad).host.l, url);
        url_path_pos = (*url).l;
        crate::htslib_rs::hts::kputc(b'/' as c_int, url);
        crate::htslib_rs::hts::kputsn(bucket, bucket_len, url);
    }
    crate::htslib_rs::hts::kputs(if escaped.is_null() { path } else { escaped }, url);

    let bucket_alloc_len = if sigver == 4 || dns_compliant == 0 {
        (*url).l - url_path_pos + 1
    } else {
        (*url).l - url_path_pos + bucket_len + 2
    };
    (*ad).bucket = libc::malloc(bucket_alloc_len).cast();
    if (*ad).bucket.is_null() {
        libc::free(escaped.cast());
        hfile_s3_c_319_free_auth_data(ad.cast());
        return std::ptr::null_mut();
    }
    if sigver == 4 || dns_compliant == 0 {
        libc::memcpy(
            (*ad).bucket.cast(),
            (*url).s.add(url_path_pos).cast(),
            (*url).l - url_path_pos + 1,
        );
    } else {
        *(*ad).bucket = b'/' as c_char;
        libc::memcpy((*ad).bucket.add(1).cast(), bucket.cast(), bucket_len);
        libc::memcpy(
            (*ad).bucket.add(bucket_len + 1).cast(),
            (*url).s.add(url_path_pos).cast(),
            (*url).l - url_path_pos + 1,
        );
    }
    let query_start = libc::strchr((*ad).bucket, b'?' as c_int);
    if !query_start.is_null() {
        crate::htslib_rs::hts::kputs(query_start.add(1), &mut (*ad).user_query_string);
        *query_start = 0;
    }
    libc::free(escaped.cast());
    ad.cast()
}

// original: v2_authorisation (htslib/hfile_s3.c:774)
pub unsafe extern "C" fn hfile_s3_c_774_v2_authorisation(
    ctx: *mut c_void,
    hdrs: *mut *mut *mut c_char,
) -> c_int {
    let ad = ctx.cast::<S3AuthDataLayout>();
    let now = libc::time(std::ptr::null_mut());
    let mut message: kstring_t = std::mem::zeroed();
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
    if (*ad).id.l == 0 || (*ad).secret.l == 0 {
        (*ad).auth_time = now;
        return hfile_s3_copy_auth_headers(ad, hdrs);
    }
    let method = if (*ad).mode == b'r' as c_char {
        "GET"
    } else {
        "PUT"
    };
    let token_prefix = if (*ad).token.l != 0 {
        "x-amz-security-token:"
    } else {
        ""
    };
    let token = if (*ad).token.l != 0 {
        CStr::from_ptr((*ad).token.s).to_string_lossy()
    } else {
        "".into()
    };
    let token_nl = if (*ad).token.l != 0 { "\n" } else { "" };
    if kput_cstring(
        &mut message,
        format!(
            "{}\n\n\n{}\n{}{}{}{}",
            method,
            CStr::from_ptr((*ad).date.as_ptr().add(6)).to_string_lossy(),
            token_prefix,
            token,
            token_nl,
            CStr::from_ptr((*ad).bucket).to_string_lossy()
        ),
    ) < 0
    {
        return -1;
    }
    let digest_len = hfile_s3_c_142_s3_sign(digest.as_mut_ptr(), &mut (*ad).secret, &mut message);
    (*ad).auth_hdr.l = 0;
    if kput_cstring(
        &mut (*ad).auth_hdr,
        format!(
            "Authorization: AWS {}:",
            CStr::from_ptr((*ad).id.s).to_string_lossy()
        ),
    ) < 0
    {
        libc::free(message.s.cast());
        return -1;
    }
    hfile_s3_c_181_base64_kput(digest.as_ptr(), digest_len, &mut (*ad).auth_hdr);
    libc::free(message.s.cast());
    (*ad).auth_time = now;
    hfile_s3_copy_auth_headers(ad, hdrs)
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
    let mut secret_access_key: kstring_t = std::mem::zeroed();
    let mut len = 0 as c_uint;

    if kput_cstring(
        &mut secret_access_key,
        format!("AWS4{}", CStr::from_ptr((*ad).secret.s).to_string_lossy()),
    ) < 0
        || secret_access_key.l == 0
    {
        return -1;
    }
    hfile_s3_c_157_s3_sign_sha256(
        secret_access_key.s.cast(),
        secret_access_key.l as c_int,
        (*ad).date_short.as_ptr().cast(),
        libc::strlen((*ad).date_short.as_ptr()) as c_int,
        date_key.as_mut_ptr(),
        &mut len,
    );
    hfile_s3_c_157_s3_sign_sha256(
        date_key.as_ptr().cast(),
        len as c_int,
        (*ad).region.s.cast(),
        (*ad).region.l as c_int,
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
        (*string_to_sign).s.cast(),
        (*string_to_sign).l as c_int,
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
    let mut signed_headers: kstring_t = std::mem::zeroed();
    let mut canonical_headers: kstring_t = std::mem::zeroed();
    let mut canonical_request: kstring_t = std::mem::zeroed();
    let mut scope: kstring_t = std::mem::zeroed();
    let mut string_to_sign: kstring_t = std::mem::zeroed();
    let mut cr_hash = [0 as c_char; HASH_LENGTH_SHA256];
    let mut signature_string = [0 as c_char; HASH_LENGTH_SHA256];
    let mut ret = -1;

    if (*ad).token.l == 0 {
        kputs_literal(b"host;x-amz-content-sha256;x-amz-date", &mut signed_headers);
    } else {
        kputs_literal(
            b"host;x-amz-content-sha256;x-amz-date;x-amz-security-token",
            &mut signed_headers,
        );
    }
    if signed_headers.l == 0 {
        return -1;
    }

    if (*ad).token.l == 0 {
        kput_cstring(
            &mut canonical_headers,
            format!(
                "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
                CStr::from_ptr((*ad).host.s).to_string_lossy(),
                CStr::from_ptr(content).to_string_lossy(),
                CStr::from_ptr((*ad).date_long.as_ptr()).to_string_lossy()
            ),
        );
    } else {
        kput_cstring(
            &mut canonical_headers,
            format!(
                "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\nx-amz-security-token:{}\n",
                CStr::from_ptr((*ad).host.s).to_string_lossy(),
                CStr::from_ptr(content).to_string_lossy(),
                CStr::from_ptr((*ad).date_long.as_ptr()).to_string_lossy(),
                CStr::from_ptr((*ad).token.s).to_string_lossy()
            ),
        );
    }
    if canonical_headers.l != 0 {
        kput_cstring(
            &mut canonical_request,
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}",
                CStr::from_ptr(http_request).to_string_lossy(),
                CStr::from_ptr((*ad).bucket).to_string_lossy(),
                CStr::from_ptr((*ad).canonical_query_string.s).to_string_lossy(),
                CStr::from_ptr(canonical_headers.s).to_string_lossy(),
                CStr::from_ptr(signed_headers.s).to_string_lossy(),
                CStr::from_ptr(content).to_string_lossy()
            ),
        );
        if canonical_request.l != 0 {
            hfile_s3_c_836_hash_string(
                canonical_request.s,
                canonical_request.l,
                cr_hash.as_mut_ptr(),
                cr_hash.len(),
            );
            kput_cstring(
                &mut scope,
                format!(
                    "{}/{}/s3/aws4_request",
                    CStr::from_ptr((*ad).date_short.as_ptr()).to_string_lossy(),
                    CStr::from_ptr((*ad).region.s).to_string_lossy()
                ),
            );
            if scope.l != 0 {
                kput_cstring(
                    &mut string_to_sign,
                    format!(
                        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
                        CStr::from_ptr((*ad).date_long.as_ptr()).to_string_lossy(),
                        CStr::from_ptr(scope.s).to_string_lossy(),
                        CStr::from_ptr(cr_hash.as_ptr()).to_string_lossy()
                    ),
                );
                if string_to_sign.l != 0
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
                            CStr::from_ptr((*ad).id.s).to_string_lossy(),
                            CStr::from_ptr((*ad).date_short.as_ptr()).to_string_lossy(),
                            CStr::from_ptr((*ad).region.s).to_string_lossy(),
                            CStr::from_ptr(signed_headers.s).to_string_lossy(),
                            CStr::from_ptr(signature_string.as_ptr()).to_string_lossy()
                        ),
                    );
                    if (*auth).l != 0 {
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

        (*ad).date_html.l = 0;
        crate::htslib_rs::hts::kputs(c"x-amz-date: ".as_ptr(), &mut (*ad).date_html);
        crate::htslib_rs::hts::kputs((*ad).date_long.as_ptr(), &mut (*ad).date_html);
    }

    if (*ad).date_html.l != 0 {
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
    let mut num_queries = 0;
    let query_offset = crate::htslib_rs::hts::ksplit(qs, b'&' as c_int, &mut num_queries);
    if query_offset.is_null() {
        return -1;
    }

    let mut queries = Vec::with_capacity(num_queries as usize);
    for i in 0..num_queries {
        queries.push((*qs).s.add(*query_offset.add(i as usize) as usize));
    }
    queries.sort_by(|a, b| unsafe { libc::strcmp(*a, *b).cmp(&0) });

    let mut ordered: kstring_t = std::mem::zeroed();
    let mut ret = -1;

    for (i, query) in queries.iter().enumerate() {
        if i != 0 {
            crate::htslib_rs::hts::kputs(c"&".as_ptr(), &mut ordered);
        }
        crate::htslib_rs::hts::kputs(*query, &mut ordered);
    }

    let escaped = hfile_s3_c_396_escape_query(ordered.s);
    if !escaped.is_null() {
        (*qs).l = 0;
        crate::htslib_rs::hts::kputs(escaped, qs);
        ret = 0;
    }

    libc::free(ordered.s.cast());
    libc::free(query_offset.cast());
    libc::free(escaped.cast());

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
            (*content).s,
            (*content).l,
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
    (*ad).canonical_query_string.l = 0;
    crate::htslib_rs::hts::kputs(cqs, &mut (*ad).canonical_query_string);
    if (*ad).canonical_query_string.l == 0 {
        return -1;
    }
    if uqs != 0 {
        crate::htslib_rs::hts::kputs(c"&".as_ptr(), &mut (*ad).canonical_query_string);
        crate::htslib_rs::hts::kputs((*ad).user_query_string.s, &mut (*ad).canonical_query_string);
        if hfile_s3_c_1009_order_query_string(&mut (*ad).canonical_query_string) != 0 {
            return -1;
        }
    }
    if hfile_s3_c_884_make_authorisation(ad.cast(), request, content_hash.as_mut_ptr(), auth_str)
        != 0
    {
        return -1;
    }
    crate::htslib_rs::hts::kputs((*ad).date_html.s, date);
    crate::htslib_rs::hts::kputsn(content_hash.as_ptr(), HASH_LENGTH_SHA256, hash);
    if (*date).l == 0 || (*hash).l == 0 {
        return -1;
    }
    if (*ad).token.l != 0 {
        kput_cstring(
            &mut *token,
            format!(
                "x-amz-security-token: {}",
                CStr::from_ptr((*ad).token.s).to_string_lossy()
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
    let mut content: kstring_t = std::mem::zeroed();
    let mut authorisation: kstring_t = std::mem::zeroed();
    let mut token_hdr: kstring_t = std::mem::zeroed();
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
    if (*ad).id.l == 0 || (*ad).secret.l == 0 {
        return hfile_s3_copy_auth_headers(ad, hdrs);
    }
    hfile_s3_c_836_hash_string(
        c"".as_ptr().cast_mut(),
        0,
        content_hash.as_mut_ptr(),
        content_hash.len(),
    );
    (*ad).canonical_query_string.l = 0;
    if (*ad).user_query_string.l > 0 {
        crate::htslib_rs::hts::kputs((*ad).user_query_string.s, &mut (*ad).canonical_query_string);
        if hfile_s3_c_1009_order_query_string(&mut (*ad).canonical_query_string) != 0 {
            return -1;
        }
    } else {
        crate::htslib_rs::hts::kputs(c"".as_ptr(), &mut (*ad).canonical_query_string);
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
    let date_html = libc::strdup((*ad).date_html.s);
    if (*ad).token.l > 0 {
        crate::htslib_rs::hts::kputs(c"X-Amz-Security-Token: ".as_ptr(), &mut token_hdr);
        crate::htslib_rs::hts::kputs((*ad).token.s, &mut token_hdr);
    }
    if content.l == 0 || date_html.is_null() {
        crate::htslib_rs::hts::ks_free(&mut authorisation);
        crate::htslib_rs::hts::ks_free(&mut content);
        crate::htslib_rs::hts::ks_free(&mut token_hdr);
        libc::free(date_html.cast());
        return -1;
    }
    *hdrs = (*ad).headers.as_mut_ptr();
    let mut idx = 0usize;
    (*ad).headers[idx] = ks_release_or_free(&mut authorisation);
    idx += 1;
    (*ad).headers[idx] = date_html;
    idx += 1;
    (*ad).headers[idx] = ks_release_or_free(&mut content);
    idx += 1;
    if !token_hdr.s.is_null() {
        (*ad).headers[idx] = ks_release_or_free(&mut token_hdr);
        idx += 1;
    }
    (*ad).headers[idx] = std::ptr::null_mut();
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
    (*ad).region.l = 0;
    crate::htslib_rs::hts::kputsn(
        region,
        reg_end.offset_from(region) as usize,
        &mut (*ad).region,
    );
    if (*ad).region.l == 0 {
        -1
    } else {
        0
    }
}

// original: set_region (htslib/hfile_s3.c:1112)
pub unsafe fn hfile_s3_c_1112_set_region(ad: *mut s3_auth_data, region: *mut kstring_t) -> c_int {
    let ad = ad.cast::<S3AuthDataLayout>();
    (*ad).region.l = 0;
    (crate::htslib_rs::hts::kputsn((*region).s, (*region).l, &mut (*ad).region) < 0) as c_int
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

    crate::htslib_rs::hts::kputsn(start, end.offset_from(start) as usize, out)
}

// original: report_s3_error (htslib/hfile_s3.c:1218)
pub unsafe fn hfile_s3_c_1218_report_s3_error(
    body: *mut kstring_t,
    resp_code: libc::c_long,
) -> c_int {
    let mut entry: kstring_t = std::mem::zeroed();

    if hfile_s3_c_1198_get_entry(
        (*body).s,
        c"<Code>".as_ptr().cast_mut(),
        c"</Code>".as_ptr().cast_mut(),
        &mut entry,
    ) == libc::EOF
    {
        return -1;
    }

    libc::fprintf(
        crate::htslib_rs::c_compat::stderr.cast(),
        c"hfile_s3: S3 error %ld: %s\n".as_ptr(),
        resp_code,
        entry.s,
    );

    entry.l = 0;
    if !entry.s.is_null() {
        *entry.s = 0;
    }

    if hfile_s3_c_1198_get_entry(
        (*body).s,
        c"<Message>".as_ptr().cast_mut(),
        c"</Message>".as_ptr().cast_mut(),
        &mut entry,
    ) == libc::EOF
    {
        libc::free(entry.s.cast());
        return -1;
    }

    if entry.l != 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"%s\n".as_ptr(),
            entry.s,
        );
    }

    libc::free(entry.s.cast());
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
unsafe fn hfile_s3_cleanup_local(fp: *mut hFILE_s3) {
    crate::htslib_rs::hts::ks_free(&mut (*fp).buffer);
    crate::htslib_rs::hts::ks_free(&mut (*fp).url);
    crate::htslib_rs::hts::ks_free(&mut (*fp).upload_id);
    crate::htslib_rs::hts::ks_free(&mut (*fp).completion_message);
    crate::htslib_rs::hts::ks_free(&mut (*fp).content_hash);
    crate::htslib_rs::hts::ks_free(&mut (*fp).authorisation);
    crate::htslib_rs::hts::ks_free(&mut (*fp).content);
    crate::htslib_rs::hts::ks_free(&mut (*fp).date);
    crate::htslib_rs::hts::ks_free(&mut (*fp).token);
    crate::htslib_rs::hts::ks_free(&mut (*fp).range);
    if !(*fp).curl.is_null() {
        curl_easy_cleanup((*fp).curl);
        (*fp).curl = std::ptr::null_mut();
    }
}

// original: cleanup (htslib/hfile_s3.c:1286)
unsafe fn hfile_s3_cleanup(fp: *mut hFILE_s3) {
    if !(*fp).au.is_null() {
        hfile_s3_c_319_free_auth_data((*fp).au.cast());
        (*fp).au = std::ptr::null_mut();
    }
    hfile_s3_cleanup_local(fp);
}

unsafe fn hfile_s3_clear_authorisation_values(fp: *mut hFILE_s3) {
    crate::htslib_rs::hts::ks_clear(&mut (*fp).content_hash);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).authorisation);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).content);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).date);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).token);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).range);
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
    if crate::htslib_rs::hts::kputsn(contents.cast(), realsize, resp) == libc::EOF {
        0
    } else {
        realsize
    }
}

unsafe fn hfile_s3_add_header(
    head: *mut *mut HFileLibcurlCurlSlist,
    value: *const c_char,
) -> c_int {
    let tmp = curl_slist_append(*head, value);
    if tmp.is_null() {
        1
    } else {
        *head = tmp;
        0
    }
}

unsafe fn hfile_s3_set_html_headers(
    fp: *mut hFILE_s3,
    auth: *mut kstring_t,
    date: *mut kstring_t,
    content: *mut kstring_t,
    token: *mut kstring_t,
    range: *mut kstring_t,
) -> *mut HFileLibcurlCurlSlist {
    let mut headers: *mut HFileLibcurlCurlSlist = std::ptr::null_mut();
    let mut err = 0;

    err |= hfile_s3_add_header(&mut headers, c"Content-Type:".as_ptr());
    err |= hfile_s3_add_header(&mut headers, c"Expect:".as_ptr());
    if err == 0 && (*auth).l != 0 {
        err |= hfile_s3_add_header(&mut headers, (*auth).s);
    }
    if err == 0 {
        err |= hfile_s3_add_header(&mut headers, (*date).s);
    }
    if err == 0 && (*content).l != 0 {
        err |= hfile_s3_add_header(&mut headers, (*content).s);
    }
    if err == 0 && !range.is_null() {
        err |= hfile_s3_add_header(&mut headers, (*range).s);
    }
    if err == 0 && (*token).l != 0 {
        err |= hfile_s3_add_header(&mut headers, (*token).s);
    }
    if err == 0 {
        err |= curl_easy_setopt((*fp).curl, CURLOPT_HTTPHEADER, headers);
    }

    if err != 0 {
        curl_slist_free_all(headers);
        std::ptr::null_mut()
    } else {
        headers
    }
}

unsafe fn hfile_s3_response_code(fp: *mut hFILE_s3, response_code: *mut libc::c_long) -> c_int {
    if (*fp).curl.is_null() {
        return -1;
    }
    curl_easy_getinfo_long((*fp).curl, CURLINFO_RESPONSE_CODE, response_code)
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
    if hfile_s3_c_1198_get_entry(
        (*resp).s,
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
    let mut etag: kstring_t = std::mem::zeroed();
    if hfile_s3_c_1198_get_entry(
        (*resp).s,
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
            CStr::from_ptr(etag.s).to_string_lossy()
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
    let mut url: kstring_t = std::mem::zeroed();
    let mut canonical_query_string: kstring_t = std::mem::zeroed();
    let mut ret = -1;
    let save_errno = *crate::htslib_rs::c_compat::__errno_location();
    let mut headers: *mut HFileLibcurlCurlSlist = std::ptr::null_mut();

    hfile_s3_clear_authorisation_values(fp);
    if (*fp).curl.is_null() {
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
        &mut canonical_query_string,
        format!(
            "uploadId={}",
            CStr::from_ptr((*fp).upload_id.s).to_string_lossy()
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
    if hfile_s3_c_1055_v4_authorisation(
        (*fp).au.cast(),
        c"DELETE".as_ptr().cast_mut(),
        std::ptr::null_mut(),
        canonical_query_string.s,
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
            CStr::from_ptr((*fp).url.s).to_string_lossy(),
            CStr::from_ptr(canonical_query_string.s).to_string_lossy()
        ),
    ) < 0
        || kput_cstring(
            &mut (*fp).content,
            format!(
                "x-amz-content-sha256: {}",
                CStr::from_ptr((*fp).content_hash.s).to_string_lossy()
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

    curl_easy_reset((*fp).curl);
    let mut err = curl_easy_setopt((*fp).curl, CURLOPT_CUSTOMREQUEST, c"DELETE".as_ptr());
    err |= curl_easy_setopt((*fp).curl, CURLOPT_USERAGENT, HFILE_S3_USERAGENT.s);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_URL, url.s);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_VERBOSE, (*fp).verbose);
    if err == CURLE_OK {
        headers = hfile_s3_set_html_headers(
            fp,
            &mut (*fp).authorisation,
            &mut (*fp).date,
            &mut (*fp).content,
            &mut (*fp).token,
            std::ptr::null_mut(),
        );
        if !headers.is_null() {
            (*fp).ret = curl_easy_perform((*fp).curl);
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
    headers: *mut HFileLibcurlCurlSlist,
) {
    crate::htslib_rs::hts::ks_free(&mut *url);
    crate::htslib_rs::hts::ks_free(&mut *canonical_query_string);
    curl_slist_free_all(headers);
    (*fp).aborted = 1;
    hfile_s3_cleanup(fp);
    *crate::htslib_rs::c_compat::__errno_location() = save_errno;
}

// original: complete_upload (htslib/hfile_s3.c:1479)
pub unsafe fn hfile_s3_c_1479_complete_upload(fp: *mut hFILE_s3, resp: *mut kstring_t) -> c_int {
    let mut url: kstring_t = std::mem::zeroed();
    let mut canonical_query_string: kstring_t = std::mem::zeroed();
    let mut ret = -1;
    let mut headers: *mut HFileLibcurlCurlSlist = std::ptr::null_mut();

    hfile_s3_clear_authorisation_values(fp);
    if (*fp).curl.is_null()
        || kput_cstring(
            &mut canonical_query_string,
            format!(
                "uploadId={}",
                CStr::from_ptr((*fp).upload_id.s).to_string_lossy()
            ),
        ) < 0
        || kputs_literal(
            b"</CompleteMultipartUpload>\n",
            &mut (*fp).completion_message,
        ) < 0
        || hfile_s3_c_1055_v4_authorisation(
            (*fp).au.cast(),
            c"POST".as_ptr().cast_mut(),
            &mut (*fp).completion_message,
            canonical_query_string.s,
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
                CStr::from_ptr((*fp).url.s).to_string_lossy(),
                CStr::from_ptr(canonical_query_string.s).to_string_lossy()
            ),
        ) < 0
        || kput_cstring(
            &mut (*fp).content,
            format!(
                "x-amz-content-sha256: {}",
                CStr::from_ptr((*fp).content_hash.s).to_string_lossy()
            ),
        ) < 0
    {
        goto_complete_out(&mut url, &mut canonical_query_string, headers);
        return -1;
    }

    curl_easy_reset((*fp).curl);
    let mut err = curl_easy_setopt((*fp).curl, CURLOPT_POST, 1 as libc::c_long);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_POSTFIELDS, (*fp).completion_message.s);
    err |= curl_easy_setopt(
        (*fp).curl,
        CURLOPT_POSTFIELDSIZE,
        (*fp).completion_message.l as libc::c_long,
    );
    err |= curl_easy_setopt(
        (*fp).curl,
        CURLOPT_WRITEFUNCTION,
        hfile_s3_c_1293_response_callback as usize,
    );
    err |= curl_easy_setopt((*fp).curl, CURLOPT_WRITEDATA, resp.cast::<c_void>());
    err |= curl_easy_setopt((*fp).curl, CURLOPT_URL, url.s);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_USERAGENT, HFILE_S3_USERAGENT.s);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_VERBOSE, (*fp).verbose);
    if err == CURLE_OK {
        headers = hfile_s3_set_html_headers(
            fp,
            &mut (*fp).authorisation,
            &mut (*fp).date,
            &mut (*fp).content,
            &mut (*fp).token,
            std::ptr::null_mut(),
        );
        if !headers.is_null() {
            (*fp).ret = curl_easy_perform((*fp).curl);
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
    headers: *mut HFileLibcurlCurlSlist,
) {
    crate::htslib_rs::hts::ks_free(&mut *url);
    crate::htslib_rs::hts::ks_free(&mut *canonical_query_string);
    curl_slist_free_all(headers);
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
    let remaining = (*fp).buffer.l.saturating_sub((*fp).index);
    let read_length = remaining.min(realsize);
    if read_length != 0 {
        libc::memcpy(ptr, (*fp).buffer.s.add((*fp).index).cast(), read_length);
        (*fp).index += read_length;
    }
    read_length
}

// original: upload_part (htslib/hfile_s3.c:1563)
pub unsafe fn hfile_s3_c_1563_upload_part(fp: *mut hFILE_s3, resp: *mut kstring_t) -> c_int {
    let mut url: kstring_t = std::mem::zeroed();
    let mut canonical_query_string: kstring_t = std::mem::zeroed();
    let mut ret = -1;
    let mut headers: *mut HFileLibcurlCurlSlist = std::ptr::null_mut();

    hfile_s3_clear_authorisation_values(fp);
    if (*fp).curl.is_null()
        || kput_cstring(
            &mut canonical_query_string,
            format!(
                "partNumber={}&uploadId={}",
                (*fp).part_no,
                CStr::from_ptr((*fp).upload_id.s).to_string_lossy()
            ),
        ) < 0
        || hfile_s3_c_1055_v4_authorisation(
            (*fp).au.cast(),
            c"PUT".as_ptr().cast_mut(),
            &mut (*fp).buffer,
            canonical_query_string.s,
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
                CStr::from_ptr((*fp).url.s).to_string_lossy(),
                CStr::from_ptr(canonical_query_string.s).to_string_lossy()
            ),
        ) < 0
        || kput_cstring(
            &mut (*fp).content,
            format!(
                "x-amz-content-sha256: {}",
                CStr::from_ptr((*fp).content_hash.s).to_string_lossy()
            ),
        ) < 0
    {
        goto_complete_out(&mut url, &mut canonical_query_string, headers);
        return -1;
    }

    (*fp).index = 0;
    curl_easy_reset((*fp).curl);
    let mut err = curl_easy_setopt((*fp).curl, CURLOPT_UPLOAD, 1 as libc::c_long);
    err |= curl_easy_setopt(
        (*fp).curl,
        CURLOPT_READFUNCTION,
        hfile_s3_c_1546_upload_callback as usize,
    );
    err |= curl_easy_setopt((*fp).curl, CURLOPT_READDATA, fp.cast::<c_void>());
    err |= curl_easy_setopt(
        (*fp).curl,
        CURLOPT_INFILESIZE_LARGE,
        (*fp).buffer.l as libc::off_t,
    );
    err |= curl_easy_setopt(
        (*fp).curl,
        CURLOPT_HEADERFUNCTION,
        hfile_s3_c_1293_response_callback as usize,
    );
    err |= curl_easy_setopt((*fp).curl, CURLOPT_HEADERDATA, resp.cast::<c_void>());
    err |= curl_easy_setopt((*fp).curl, CURLOPT_URL, url.s);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_USERAGENT, HFILE_S3_USERAGENT.s);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_VERBOSE, (*fp).verbose);
    if err == CURLE_OK {
        headers = hfile_s3_set_html_headers(
            fp,
            &mut (*fp).authorisation,
            &mut (*fp).date,
            &mut (*fp).content,
            &mut (*fp).token,
            std::ptr::null_mut(),
        );
        if !headers.is_null() {
            (*fp).ret = curl_easy_perform((*fp).curl);
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
        if crate::htslib_rs::hts::kputsn(ptr, n, &mut (*fp).buffer) == libc::EOF {
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
pub unsafe extern "C" fn hfile_s3_c_1869_s3_read_close(fpv: *mut hFILE) -> c_int {
    hfile_s3_cleanup(fpv.cast::<hFILE_s3>());
    0
}

// original: s3_write (htslib/hfile_s3.c:1625)
pub unsafe extern "C" fn hfile_s3_c_1625_s3_write(
    fpv: *mut hFILE,
    bufferv: *const c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let fp = fpv.cast::<hFILE_s3>();
    if crate::htslib_rs::hts::kputsn(bufferv.cast(), nbytes, &mut (*fp).buffer) == libc::EOF {
        return -1;
    }

    if (*fp).buffer.l > (*fp).part_size as usize {
        let mut response: kstring_t = std::mem::zeroed();
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
        (*fp).buffer.l = 0;
        if (*fp).expand != 0 && (*fp).part_no % EXPAND_ON == 0 {
            (*fp).part_size *= 2;
        }
    }

    nbytes as libc::ssize_t
}

// original: s3_write_close (htslib/hfile_s3.c:1682)
pub unsafe extern "C" fn hfile_s3_c_1682_s3_write_close(fpv: *mut hFILE) -> c_int {
    let fp = fpv.cast::<hFILE_s3>();
    let mut response: kstring_t = std::mem::zeroed();
    let mut ret = 0;

    if (*fp).aborted == 0 {
        if (*fp).buffer.l != 0 {
            ret = hfile_s3_c_1563_upload_part(fp, &mut response);
            if ret == 0 {
                ret = hfile_s3_finish_uploaded_part(fp, &mut response);
            }
            crate::htslib_rs::hts::ks_free(&mut response);
            response = std::mem::zeroed();
            if ret != 0 {
                hfile_s3_c_1417_abort_upload(fp);
                return -1;
            }
            (*fp).part_no += 1;
        }

        if (*fp).part_no > 1 {
            ret = hfile_s3_c_1479_complete_upload(fp, &mut response);
            if ret == 0
                && (response.s.is_null()
                    || libc::strstr(response.s, c"CompleteMultipartUploadResult".as_ptr())
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
            hfile_s3_cleanup(fp);
        }
    }

    crate::htslib_rs::hts::ks_free(&mut response);
    ret
}

unsafe fn hfile_s3_handle_bad_request(fp: *mut hFILE_s3, resp: *mut kstring_t) -> c_int {
    let mut region: kstring_t = std::mem::zeroed();
    if hfile_s3_c_1198_get_entry(
        (*resp).s,
        c"<Region>".as_ptr().cast_mut(),
        c"</Region>".as_ptr().cast_mut(),
        &mut region,
    ) == libc::EOF
    {
        return -1;
    }
    let ret = hfile_s3_c_1112_set_region((*fp).au.cast(), &mut region);
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
    let mut url: kstring_t = std::mem::zeroed();
    let mut ret = -1;
    let mut headers: *mut HFileLibcurlCurlSlist = std::ptr::null_mut();
    let delimiter = if user_query != 0 { '&' } else { '?' };

    hfile_s3_clear_authorisation_values(fp);
    if (*fp).curl.is_null()
        || hfile_s3_c_1055_v4_authorisation(
            (*fp).au.cast(),
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
                CStr::from_ptr((*fp).url.s).to_string_lossy(),
                delimiter
            ),
        ) < 0
        || kput_cstring(
            &mut (*fp).content,
            format!(
                "x-amz-content-sha256: {}",
                CStr::from_ptr((*fp).content_hash.s).to_string_lossy()
            ),
        ) < 0
    {
        crate::htslib_rs::hts::ks_free(&mut url);
        return -1;
    }

    let mut err = curl_easy_setopt((*fp).curl, CURLOPT_URL, url.s);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_POST, 1 as libc::c_long);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_POSTFIELDS, c"".as_ptr());
    err |= curl_easy_setopt(
        (*fp).curl,
        CURLOPT_WRITEFUNCTION,
        hfile_s3_c_1293_response_callback as usize,
    );
    err |= curl_easy_setopt((*fp).curl, CURLOPT_WRITEDATA, resp.cast::<c_void>());
    err |= curl_easy_setopt(
        (*fp).curl,
        CURLOPT_HEADERFUNCTION,
        hfile_s3_c_1293_response_callback as usize,
    );
    err |= curl_easy_setopt((*fp).curl, CURLOPT_HEADERDATA, head.cast::<c_void>());
    err |= curl_easy_setopt((*fp).curl, CURLOPT_USERAGENT, HFILE_S3_USERAGENT.s);
    err |= curl_easy_setopt((*fp).curl, CURLOPT_VERBOSE, (*fp).verbose);
    if err == CURLE_OK {
        headers = hfile_s3_set_html_headers(
            fp,
            &mut (*fp).authorisation,
            &mut (*fp).date,
            &mut (*fp).content,
            &mut (*fp).token,
            std::ptr::null_mut(),
        );
        if !headers.is_null() {
            (*fp).ret = curl_easy_perform((*fp).curl);
            if (*fp).ret == CURLE_OK {
                ret = 0;
            }
        }
    }

    curl_slist_free_all(headers);
    crate::htslib_rs::hts::ks_free(&mut url);
    ret
}

unsafe extern "C" fn hfile_s3_c_2072_s3_close(fpv: *mut hFILE) -> c_int {
    let fp = fpv.cast::<hFILE_s3>();
    if (*fp).write == 0 {
        hfile_s3_c_1869_s3_read_close(fpv)
    } else {
        hfile_s3_c_1682_s3_write_close(fpv)
    }
}

static S3_BACKEND: hFILE_backend = hFILE_backend {
    read: None,
    write: Some(hfile_s3_c_1625_s3_write),
    seek: Some(hfile_s3_c_2015_s3_seek),
    flush: None,
    close: Some(hfile_s3_c_2072_s3_close),
};

// original: s3_seek (htslib/hfile_s3.c:2015)
pub unsafe extern "C" fn hfile_s3_c_2015_s3_seek(
    fpv: *mut hFILE,
    offset: libc::off_t,
    whence: c_int,
) -> libc::off_t {
    let fp = fpv.cast::<hFILE_s3>();

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

    let buffer_start = (*fp).last_read.saturating_sub((*fp).buffer.l);
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
    let fp = hfile_init(std::mem::size_of::<hFILE_s3>(), c"w".as_ptr(), 0).cast::<hFILE_s3>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }
    (*fp).curl = curl_easy_init();
    if (*fp).curl.is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
        hfile_destroy(fp.cast());
        return std::ptr::null_mut();
    }
    (*fp).au = auth.cast();
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
    if crate::htslib_rs::hts::kputs(url, &mut (*fp).url) < 0 {
        hfile_s3_cleanup_local(fp);
        hfile_destroy(fp.cast());
        return std::ptr::null_mut();
    }

    let query_start = libc::strchr((*fp).url.s, b'?' as c_int);
    let has_user_query = (!query_start.is_null()) as c_int;
    let mut response: kstring_t = std::mem::zeroed();
    let mut header: kstring_t = std::mem::zeroed();

    if hfile_s3_c_1779_initialise_upload(fp, &mut header, &mut response, has_user_query) != 0 {
        goto_write_open_error(fp, &mut response, &mut header);
        return std::ptr::null_mut();
    }

    let mut response_code: libc::c_long = 0;
    let mut cret = hfile_s3_response_code(fp, &mut response_code);
    if cret == CURLE_OK {
        if response_code == S3_MOVED_PERMANENTLY || response_code == S3_TEMPORARY_REDIRECT {
            if hfile_s3_c_488_redirect_endpoint(
                (*fp).au.cast(),
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
    if !query_start.is_null() {
        *query_start = 0;
    }
    (*fp).base.backend = (&S3_BACKEND as *const hFILE_backend).cast();
    crate::htslib_rs::hts::ks_free(&mut response);
    crate::htslib_rs::hts::ks_free(&mut header);
    fp.cast()
}

unsafe fn goto_write_open_error(
    fp: *mut hFILE_s3,
    response: *mut kstring_t,
    header: *mut kstring_t,
) {
    crate::htslib_rs::hts::ks_free(&mut *response);
    crate::htslib_rs::hts::ks_free(&mut *header);
    hfile_s3_cleanup_local(fp);
    hfile_destroy(fp.cast());
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
        let mut mode_colon: kstring_t = std::mem::zeroed();
        if crate::htslib_rs::hts::kputs(mode, &mut mode_colon) < 0
            || crate::htslib_rs::hts::kputc(b':' as c_int, &mut mode_colon) < 0
        {
            libc::free(mode_colon.s.cast());
            return std::ptr::null_mut();
        }
        let fp = hfile_c_1317_hopen_vargs(url, mode_colon.s, &mut args);
        libc::free(mode_colon.s.cast());
        fp
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
    let mut url: kstring_t = std::mem::zeroed();
    let ad = hfile_s3_c_545_setup_auth_data(s3url, mode, 2, &mut url);
    if ad.is_null() {
        return std::ptr::null_mut();
    }
    let fp = hfile_s3_c_774_hopen_v2_read(url.s, mode, argsp, ad);
    if fp.is_null() {
        hfile_s3_c_319_free_auth_data(ad);
    }
    libc::free(url.s.cast());
    fp
}

// original: s3_open_v4 (htslib/hfile_s3.c:2348)
pub unsafe fn hfile_s3_c_2348_s3_open_v4(
    s3url: *const c_char,
    mode: *const c_char,
    argsp: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    let mut url: kstring_t = std::mem::zeroed();
    let ad = hfile_s3_c_545_setup_auth_data(s3url, mode, 4, &mut url).cast::<S3AuthDataLayout>();
    if ad.is_null() {
        return std::ptr::null_mut();
    }
    let mut fp: *mut hFILE;
    if (*ad).mode == b'r' as c_char {
        let mut http_response: libc::c_long = 0;
        fp = hfile_s3_c_2348_hopen_v4_read(url.s, mode, argsp, ad, &mut http_response, 0);
        if fp.is_null() {
            libc::free(url.s.cast());
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        }
        if http_response == 400 {
            (*ad).refcount = 1;
            if hfile_s3_c_1055_handle_400_response(fp, ad.cast()) != 0 {
                hclose_abruptly(fp);
                libc::free(url.s.cast());
                hfile_s3_c_319_free_auth_data(ad.cast());
                return std::ptr::null_mut();
            }
            hclose_abruptly(fp);
            fp = hfile_s3_c_2348_hopen_v4_read(url.s, mode, argsp, ad, std::ptr::null_mut(), 1);
        } else if http_response > 400 {
            (*ad).refcount = 1;
            *crate::htslib_rs::c_compat::__errno_location() =
                hfile_s3_c_1242_http_status_errno(http_response as c_int);
            hclose_abruptly(fp);
            libc::free(url.s.cast());
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        }
    } else {
        fp = hfile_s3_c_2348_hopen_v4_write(url.s, mode, argsp, ad);
    }

    libc::free(url.s.cast());
    if fp.is_null() {
        hfile_s3_c_319_free_auth_data(ad.cast());
    }
    fp
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
    libc::free(HFILE_S3_USERAGENT.s.cast());
    HFILE_S3_USERAGENT.l = 0;
    HFILE_S3_USERAGENT.m = 0;
    HFILE_S3_USERAGENT.s = std::ptr::null_mut();
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
    (*self_.cast::<hFILE_plugin_layout>()).destroy = hfile_s3_c_2426_s3_exit as *const c_void;
    hfile_s3_c_2426_s3_exit();
    crate::htslib_rs::kstring::kstring_c_177_ksprintf(
        std::ptr::addr_of_mut!(HFILE_S3_USERAGENT).cast(),
        c"htslib/%s".as_ptr(),
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
        let mut s: kstring_t = std::mem::zeroed();
        crate::htslib_rs::hts::kputsn(bytes.as_ptr().cast(), bytes.len(), &mut s);
        s
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
            hfile_s3_c_152_s3_sha256(message.s.cast(), message.l, sha256_digest.as_mut_ptr());
            assert_eq!(
                hex(&sha256_digest),
                "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
            );

            let mut hmac_sha256 = [0u8; SHA256_DIGEST_BUFSIZE];
            let mut hmac_len = 0;
            hfile_s3_c_157_s3_sign_sha256(
                key.s.cast(),
                key.l as c_int,
                message.s.cast(),
                message.l as c_int,
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
            let mut url: kstring_t = std::mem::zeroed();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET:TOKEN@bucket-name/path/to file.bam?b=2&a=1".as_ptr(),
                c"r".as_ptr(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());
            assert_eq!(
                CStr::from_ptr(url.s).to_bytes(),
                b"https://bucket-name.s3.amazonaws.com/path/to%20file.bam?b=2&a=1"
            );

            let ad_layout = ad.cast::<S3AuthDataLayout>();
            assert_eq!(CStr::from_ptr((*ad_layout).id.s).to_bytes(), b"AKID");
            assert_eq!(CStr::from_ptr((*ad_layout).secret.s).to_bytes(), b"SECRET");
            assert_eq!(CStr::from_ptr((*ad_layout).token.s).to_bytes(), b"TOKEN");
            assert_eq!(
                CStr::from_ptr((*ad_layout).host.s).to_bytes(),
                b"bucket-name.s3.amazonaws.com"
            );
            assert_eq!(
                CStr::from_ptr((*ad_layout).bucket).to_bytes(),
                b"/path/to%20file.bam"
            );
            assert_eq!(
                CStr::from_ptr((*ad_layout).user_query_string.s).to_bytes(),
                b"b=2&a=1"
            );

            hfile_s3_c_319_free_auth_data(ad);
            libc::free(url.s.cast());
        }
    }

    #[test]
    fn s3_write_authorisation_callback_builds_v4_upload_headers() {
        unsafe {
            let mut url: kstring_t = std::mem::zeroed();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET:TOKEN@bucket-name/path/out.bam?z=9&a=1".as_ptr(),
                c"w".as_ptr(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());

            let mut content = kstring_from_bytes(b"abc");
            let mut hash: kstring_t = std::mem::zeroed();
            let mut auth: kstring_t = std::mem::zeroed();
            let mut date: kstring_t = std::mem::zeroed();
            let mut token: kstring_t = std::mem::zeroed();

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

            assert_eq!(
                CStr::from_ptr(hash.s).to_bytes(),
                b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            );
            let auth_text = CStr::from_ptr(auth.s).to_string_lossy();
            assert!(auth_text.starts_with("Authorization: AWS4-HMAC-SHA256 Credential=AKID/"));
            assert!(auth_text.contains("/us-east-1/s3/aws4_request"));
            assert!(auth_text.contains(
                "SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-security-token"
            ));
            assert!(auth_text.contains("Signature="));
            assert!(CStr::from_ptr(date.s)
                .to_bytes()
                .starts_with(b"x-amz-date: "));
            assert_eq!(
                CStr::from_ptr(token.s).to_bytes(),
                b"x-amz-security-token: TOKEN"
            );

            crate::htslib_rs::hts::ks_free(&mut content);
            crate::htslib_rs::hts::ks_free(&mut hash);
            crate::htslib_rs::hts::ks_free(&mut auth);
            crate::htslib_rs::hts::ks_free(&mut date);
            crate::htslib_rs::hts::ks_free(&mut token);
            hfile_s3_c_319_free_auth_data(ad);
            libc::free(url.s.cast());
        }
    }

    #[test]
    fn s3_v4_read_auth_header_callback_builds_sorted_query_headers() {
        unsafe {
            let mut url: kstring_t = std::mem::zeroed();
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
                CStr::from_ptr((*ad_layout).canonical_query_string.s).to_bytes(),
                b"a=1&z=9"
            );

            let mut i = 0usize;
            while !(*hdrv.add(i)).is_null() {
                libc::free((*hdrv.add(i)).cast());
                *hdrv.add(i) = std::ptr::null_mut();
                i += 1;
            }
            hfile_s3_c_319_free_auth_data(ad);
            libc::free(url.s.cast());
        }
    }

    #[test]
    fn s3_set_region_callback_updates_auth_region_state() {
        unsafe {
            let mut url: kstring_t = std::mem::zeroed();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET@bucket-name/path/out.bam".as_ptr(),
                c"w".as_ptr(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());
            let ad_layout = ad.cast::<S3AuthDataLayout>();
            assert_eq!(
                CStr::from_ptr((*ad_layout).region.s).to_bytes(),
                b"us-east-1"
            );

            let mut region = kstring_from_bytes(b"eu-west-1");
            assert_eq!(hfile_s3_c_1112_set_region(ad, &mut region), 0);
            assert_eq!(
                CStr::from_ptr((*ad_layout).region.s).to_bytes(),
                b"eu-west-1"
            );

            crate::htslib_rs::hts::ks_free(&mut region);
            hfile_s3_c_319_free_auth_data(ad);
            libc::free(url.s.cast());
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
            let mut setup_url: kstring_t = std::mem::zeroed();
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
            assert_eq!(
                CStr::from_ptr((*ad_layout).region.s).to_bytes(),
                b"eu-west-1"
            );
            assert_eq!(
                CStr::from_ptr((*ad_layout).host.s).to_bytes(),
                b"s3.eu-west-1.amazonaws.com"
            );
            assert_eq!(
                CStr::from_ptr(redirect_url.s).to_bytes(),
                b"s3.eu-west-1.amazonaws.com/path/in.bam?z=9&a=1"
            );

            crate::htslib_rs::hts::ks_free(&mut header);
            crate::htslib_rs::hts::ks_free(&mut redirect_url);
            hfile_s3_c_319_free_auth_data(ad);
            libc::free(setup_url.s.cast());
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
            let mut fp: hFILE_s3 = std::mem::zeroed();
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
            assert_eq!(CStr::from_ptr(fp.buffer.s).to_bytes(), b"abcdefg");

            crate::htslib_rs::hts::ks_free(&mut fp.buffer);
        }
    }

    #[test]
    fn s3_response_callback_appends_payload_to_kstring() {
        unsafe {
            let mut response: kstring_t = std::mem::zeroed();
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
                CStr::from_ptr(response.s).to_bytes(),
                b"HTTP/1.1 200 OK\r\n\r\n<body/>"
            );

            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_get_upload_id_extracts_multipart_id() {
        unsafe {
            let mut fp: hFILE_s3 = std::mem::zeroed();
            let mut response = kstring_from_bytes(
                b"<?xml version=\"1.0\"?><InitiateMultipartUploadResult><UploadId>upload-123</UploadId></InitiateMultipartUploadResult>",
            );

            assert_eq!(hfile_s3_c_1837_get_upload_id(&mut fp, &mut response), 0);
            assert_eq!(CStr::from_ptr(fp.upload_id.s).to_bytes(), b"upload-123");

            crate::htslib_rs::hts::ks_free(&mut fp.upload_id);
            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_get_upload_id_rejects_missing_id() {
        unsafe {
            let mut fp: hFILE_s3 = std::mem::zeroed();
            let mut response = kstring_from_bytes(b"<Error><Code>NoSuchUpload</Code></Error>");

            assert_eq!(hfile_s3_c_1837_get_upload_id(&mut fp, &mut response), -1);
            assert!(fp.upload_id.s.is_null());

            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_append_completed_upload_part_records_etag_and_part_number() {
        unsafe {
            let mut fp: hFILE_s3 = std::mem::zeroed();
            fp.part_no = 7;
            let mut response = kstring_from_bytes(b"HTTP/1.1 200 OK\r\nEtag: \"etag-123\"\r\n\r\n");

            assert_eq!(
                hfile_s3_append_completed_upload_part(&mut fp, &mut response),
                0
            );
            assert_eq!(
                CStr::from_ptr(fp.completion_message.s).to_bytes(),
                b"\t<Part>\n\t\t<PartNumber>7</PartNumber>\n\t\t<ETag>etag-123</ETag>\n\t</Part>\n"
            );

            crate::htslib_rs::hts::ks_free(&mut fp.completion_message);
            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_append_completed_upload_part_rejects_missing_etag() {
        unsafe {
            let mut fp: hFILE_s3 = std::mem::zeroed();
            fp.part_no = 1;
            let mut response = kstring_from_bytes(b"HTTP/1.1 200 OK\r\n\r\n");

            assert_eq!(
                hfile_s3_append_completed_upload_part(&mut fp, &mut response),
                -1
            );
            assert!(fp.completion_message.s.is_null());

            crate::htslib_rs::hts::ks_free(&mut response);
        }
    }

    #[test]
    fn s3_upload_callback_streams_buffer_and_advances_index() {
        unsafe {
            let mut fp: hFILE_s3 = std::mem::zeroed();
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
            let mut fp: hFILE_s3 = std::mem::zeroed();
            fp.part_size = 10;
            fp.part_no = 1;
            fp.expand = 1;
            fp.write = 1;
            let payload = *b"abcdef";

            assert_eq!(
                hfile_s3_c_1625_s3_write(
                    (&mut fp as *mut hFILE_s3).cast(),
                    payload.as_ptr().cast(),
                    payload.len(),
                ),
                payload.len() as libc::ssize_t
            );
            assert_eq!(CStr::from_ptr(fp.buffer.s).to_bytes(), b"abcdef");
            assert_eq!(fp.part_no, 1);

            crate::htslib_rs::hts::ks_free(&mut fp.buffer);
        }
    }

    #[test]
    fn s3_write_close_aborts_unstarted_upload_without_curl() {
        unsafe {
            let mut fp: hFILE_s3 = std::mem::zeroed();
            fp.write = 1;
            fp.part_no = 1;
            fp.buffer = kstring_from_bytes(b"");

            assert_eq!(
                hfile_s3_c_1682_s3_write_close((&mut fp as *mut hFILE_s3).cast()),
                -1
            );
            assert_eq!(fp.aborted, 1);
            assert!(fp.buffer.s.is_null());
        }
    }

    #[test]
    fn s3_read_close_releases_read_state_and_auth_refcount() {
        unsafe {
            let mut url: kstring_t = std::mem::zeroed();
            let ad = hfile_s3_c_545_setup_auth_data(
                c"s3://AKID:SECRET@bucket-name/path/in.bam".as_ptr(),
                c"r".as_ptr(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());
            let ad_layout = ad.cast::<S3AuthDataLayout>();
            (*ad_layout).refcount = 1;

            let mut fp: hFILE_s3 = std::mem::zeroed();
            fp.au = ad_layout;
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

            assert_eq!(
                hfile_s3_c_1869_s3_read_close((&mut fp as *mut hFILE_s3).cast()),
                0
            );
            assert!(fp.au.is_null());
            assert!(fp.buffer.s.is_null());
            assert!(fp.url.s.is_null());
            assert!(fp.upload_id.s.is_null());
            assert!(fp.completion_message.s.is_null());
            assert!(fp.content_hash.s.is_null());
            assert!(fp.authorisation.s.is_null());
            assert!(fp.content.s.is_null());
            assert!(fp.date.s.is_null());
            assert!(fp.token.s.is_null());
            assert!(fp.range.s.is_null());
            assert_eq!((*ad_layout).refcount, 0);

            hfile_s3_c_319_free_auth_data(ad);
            libc::free(url.s.cast());
        }
    }

    #[test]
    fn s3_seek_repositions_within_buffer_or_resets_remote_position() {
        unsafe {
            let mut fp: hFILE_s3 = std::mem::zeroed();
            fp.buffer = kstring_from_bytes(b"abcdefghij");
            fp.last_read = 110;
            fp.last_read_buffer = 10;
            fp.file_size = 500;
            fp.keep_going = 0;

            assert_eq!(
                hfile_s3_c_2015_s3_seek((&mut fp as *mut hFILE_s3).cast(), 105, libc::SEEK_SET),
                110
            );
            assert_eq!(fp.last_read, 110);
            assert_eq!(fp.last_read_buffer, 5);
            assert_eq!(fp.buffer.l, 10);
            assert_eq!(fp.keep_going, 1);

            assert_eq!(
                hfile_s3_c_2015_s3_seek((&mut fp as *mut hFILE_s3).cast(), 120, libc::SEEK_SET),
                120
            );
            assert_eq!(fp.last_read, 120);
            assert_eq!(fp.buffer.l, 0);
            assert!(!fp.buffer.s.is_null());

            crate::htslib_rs::hts::ks_free(&mut fp.buffer);
        }
    }

    #[test]
    fn s3_seek_rejects_write_and_invalid_origins_like_upstream() {
        unsafe {
            let mut fp: hFILE_s3 = std::mem::zeroed();
            fp.file_size = 200;
            fp.write = 1;

            assert_eq!(
                hfile_s3_c_2015_s3_seek((&mut fp as *mut hFILE_s3).cast(), 0, libc::SEEK_SET),
                -1
            );
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::ESPIPE
            );

            fp.write = 0;
            assert_eq!(
                hfile_s3_c_2015_s3_seek((&mut fp as *mut hFILE_s3).cast(), 0, libc::SEEK_CUR),
                -1
            );
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::ENOSYS
            );

            assert_eq!(
                hfile_s3_c_2015_s3_seek((&mut fp as *mut hFILE_s3).cast(), 1, libc::SEEK_END),
                -1
            );
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );

            fp.file_size = -1;
            assert_eq!(
                hfile_s3_c_2015_s3_seek((&mut fp as *mut hFILE_s3).cast(), 0, libc::SEEK_END),
                -1
            );
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
                obj: std::ptr::null_mut(),
                name: std::ptr::null(),
                destroy: std::ptr::null(),
            };

            assert_eq!(
                hfile_s3_c_2436_PLUGIN_GLOBAL(
                    (&mut plugin as *mut hFILE_plugin_layout).cast::<hFILE_plugin>(),
                ),
                0
            );
            assert_eq!(CStr::from_ptr(plugin.name).to_bytes(), b"Amazon S3");
            assert_eq!(plugin.destroy, hfile_s3_c_2426_s3_exit as *const c_void);
            let useragent = std::ptr::addr_of!(HFILE_S3_USERAGENT.s).read();
            assert!(!useragent.is_null());
            assert!(CStr::from_ptr(useragent).to_bytes().starts_with(b"htslib/"));

            let destroy = hfile_plugin_destroy_fn(plugin.destroy);
            destroy();
            assert_eq!(std::ptr::addr_of!(HFILE_S3_USERAGENT.l).read(), 0);
            assert_eq!(std::ptr::addr_of!(HFILE_S3_USERAGENT.m).read(), 0);
            assert!(std::ptr::addr_of!(HFILE_S3_USERAGENT.s).read().is_null());

            hfile_s3_c_2426_s3_exit();
            assert!(std::ptr::addr_of!(HFILE_S3_USERAGENT.s).read().is_null());
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
