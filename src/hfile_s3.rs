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
    hfile::HFileOpt,
    hts::{hFILE, hts_verbose, kstring_t},
};
use std::ffi::c_void;
use std::ptr::NonNull;

const AUTH_LIFETIME: i64 = 60;
const CREDENTIAL_LIFETIME: i64 = 60;
const DIGEST_BUFSIZ: usize = 64;
const SHA256_DIGEST_BUFSIZE: usize = 32;
const HASH_LENGTH_SHA256: usize = SHA256_DIGEST_BUFSIZE * 2 + 1;
const MINIMUM_S3_WRITE_SIZE: i32 = 5_242_880;
const EXPAND_ON: i32 = 1112;
const S3_MOVED_PERMANENTLY: i64 = 301;

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

// Inlined pure UTC date decomposition (formerly c_compat::unix_time_utc_parts):
// returns (year, month, day, hour, minute, second, weekday).
fn unix_time_utc_parts(now: i64) -> (i32, u32, u32, u32, u32, u32, usize) {
    let secs = now;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let hour = (sod / 3_600) as u32;
    let minute = ((sod % 3_600) / 60) as u32;
    let second = (sod % 60) as u32;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = (mp + if mp < 10 { 3 } else { -9 }) as u32;
    if month <= 2 {
        year += 1;
    }
    let weekday = (days + 4).rem_euclid(7) as usize;
    (year, month, day, hour, minute, second, weekday)
}

fn write_s3_date_header(buf: &mut [u8], now: i64) {
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (year, month, day, hour, minute, second, weekday) = unix_time_utc_parts(now);
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
    if !buf.is_empty() {
        let bytes = text.as_bytes();
        let n = bytes.len().min(buf.len() - 1);
        buf[..n].copy_from_slice(&bytes[..n]);
        buf[n] = 0;
    }
}

fn write_s3_v4_dates(date_long: &mut [u8], date_short: &mut [u8], now: i64) -> bool {
    let (year, month, day, hour, minute, second, _) = unix_time_utc_parts(now);
    let long = format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        year, month, day, hour, minute, second
    );
    let short = format!("{:04}{:02}{:02}", year, month, day);
    let long_n = if date_long.is_empty() {
        0
    } else {
        let bytes = long.as_bytes();
        let n = bytes.len().min(date_long.len() - 1);
        date_long[..n].copy_from_slice(&bytes[..n]);
        date_long[n] = 0;
        n
    };
    let short_n = if date_short.is_empty() {
        0
    } else {
        let bytes = short.as_bytes();
        let n = bytes.len().min(date_short.len() - 1);
        date_short[..n].copy_from_slice(&bytes[..n]);
        date_short[n] = 0;
        n
    };
    long_n == 16 && short_n == 8
}
const S3_TEMPORARY_REDIRECT: i64 = 307;
const S3_BAD_REQUEST: i64 = 400;
const HTS_LOG_INFO: i32 = 4;

const CURLE_OK: i32 = 0;
const CURLINFO_RESPONSE_CODE: i32 = 0x200000 + 2;
const CURLOPT_WRITEDATA: i32 = 10_001;
const CURLOPT_URL: i32 = 10_002;
const CURLOPT_READDATA: i32 = 10_009;
const CURLOPT_POSTFIELDS: i32 = 10_015;
const CURLOPT_WRITEFUNCTION: i32 = 20_011;
const CURLOPT_READFUNCTION: i32 = 20_012;
const CURLOPT_USERAGENT: i32 = 10_018;
const CURLOPT_HTTPHEADER: i32 = 10_023;
const CURLOPT_HEADERDATA: i32 = 10_029;
const CURLOPT_CUSTOMREQUEST: i32 = 10_036;
const CURLOPT_VERBOSE: i32 = 41;
const CURLOPT_UPLOAD: i32 = 46;
const CURLOPT_POST: i32 = 47;
const CURLOPT_POSTFIELDSIZE: i32 = 60;
const CURLOPT_HEADERFUNCTION: i32 = 20_079;
const CURLOPT_INFILESIZE_LARGE: i32 = 30_115;

type HFileOpenFn = unsafe extern "C" fn(*const u8, *const u8) -> *mut hFILE;
type HFileIsRemoteFn = unsafe extern "C" fn(*const u8) -> i32;
type HFileVOpenFn = for<'a> unsafe fn(
    *const u8,
    *const u8,
    &'a [crate::htslib_rs::hfile::HFileOpt<'a>],
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
    provider: *const u8,
    priority: i32,
    vopen: Option<HFileVOpenFn>,
}

unsafe impl Sync for hFILE_scheme_handler_layout {}

#[repr(C)]
struct hFILE_plugin_layout {
    api_version: i32,
    obj: Option<NonNull<()>>,
    name: *const u8,
    destroy: Option<HFilePluginDestroyFn>,
}

#[repr(C)]
struct HFileLibcurlCurlSlist {
    data: *mut u8,
    next: *mut HFileLibcurlCurlSlist,
}

// hFILE_s3 no longer embeds `base: HFileLayout`; it is now the payload of the
// `HFileBackend::S3(Box<hFILE_s3>)` enum variant. The owning `hFILE` carries the
// buffer/begin/end/limit/offset/flags state directly (see hts.rs).
pub struct hFILE_s3 {
    curl: Option<NonNull<c_void>>,
    ret: i32,
    au: Option<NonNull<S3AuthDataLayout>>,
    buffer: kstring_t,
    url: kstring_t,
    verbose: i64,
    write: i32,
    part_size: i32,
    content_hash: kstring_t,
    authorisation: kstring_t,
    content: kstring_t,
    date: kstring_t,
    token: kstring_t,
    range: kstring_t,
    upload_id: kstring_t,
    completion_message: kstring_t,
    part_no: i32,
    aborted: i32,
    index: usize,
    expand: i32,
    last_read: usize,
    last_read_buffer: usize,
    file_size: i64,
    keep_going: i32,
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
    fn curl_easy_perform(curl: *mut c_void) -> i32;
    fn curl_easy_setopt(curl: *mut c_void, option: i32, ...) -> i32;
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_long(curl: *mut c_void, info: i32, value: *mut i64) -> i32;
    fn curl_slist_append(
        list: *mut HFileLibcurlCurlSlist,
        string: *const u8,
    ) -> *mut HFileLibcurlCurlSlist;
    fn curl_slist_free_all(list: *mut HFileLibcurlCurlSlist);
}

#[link(name = "crypto")]
unsafe extern "C" {
    fn HMAC(
        evp_md: *const c_void,
        key: *const c_void,
        key_len: i32,
        d: *const u8,
        n: usize,
        md: *mut u8,
        md_len: *mut u32,
    ) -> *mut u8;
    fn EVP_sha1() -> *const c_void;
    fn EVP_sha256() -> *const c_void;
    fn SHA256(d: *const u8, n: usize, md: *mut u8) -> *mut u8;
}

unsafe fn cstr_bytes(ptr: *const u8) -> Vec<u8> {
    if ptr.is_null() {
        Vec::new()
    } else {
        std::ffi::CStr::from_ptr(ptr.cast()).to_bytes().to_vec()
    }
}

unsafe fn kput_cstring(s: &mut kstring_t, text: String) -> i32 {
    crate::htslib_rs::hts::kputs(text.as_bytes(), s)
}

unsafe fn kputs_literal(text: &[u8], s: &mut kstring_t) -> i32 {
    crate::htslib_rs::hts::kputsn(text, text.len(), s)
}

unsafe fn ks_release_or_free(s: &mut kstring_t) -> *mut u8 {
    if s.data.is_empty() {
        std::ptr::null_mut()
    } else {
        // Real FFI boundary: AuthHeaders stores raw NUL-terminated C strings
        // that libcurl reads, so build an owned malloc'd C string here.
        let mut bytes = crate::htslib_rs::hts::ks_release(s);
        bytes.push(0);
        let len = bytes.len();
        let buf = libc::malloc(len).cast::<u8>();
        if buf.is_null() {
            std::ptr::null_mut()
        } else {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
            buf
        }
    }
}

// original: s3_sign (htslib/hfile_s3.c:142)
pub unsafe fn hfile_s3_c_142_s3_sign(
    digest: *mut u8,
    key: *mut kstring_t,
    message: *mut kstring_t,
) -> usize {
    let mut len = 0u32;
    HMAC(
        EVP_sha1(),
        (*key).data.as_ptr().cast(),
        (*key).data.len() as i32,
        (*message).data.as_ptr().cast(),
        (*message).data.len(),
        digest,
        &mut len,
    );
    len as usize
}

// original: s3_sha256 (htslib/hfile_s3.c:152)
pub unsafe fn hfile_s3_c_152_s3_sha256(in_: *const u8, length: usize, out: *mut u8) {
    SHA256(in_, length, out);
}

// original: s3_sign_sha256 (htslib/hfile_s3.c:157)
pub unsafe fn hfile_s3_c_157_s3_sign_sha256(
    key: *const c_void,
    key_len: i32,
    d: *const u8,
    n: i32,
    md: *mut u8,
    md_len: *mut u32,
) {
    HMAC(EVP_sha256(), key, key_len, d, n as usize, md, md_len);
}

// original: urldecode_kput (htslib/hfile_s3.c:165)
pub unsafe fn hfile_s3_c_165_urldecode_kput(s: *const u8, len: i32, str_: &mut kstring_t) {
    let mut i = 0;

    while i < len {
        if *s.add(i as usize) == b'%' && i + 2 < len {
            let hi = (*s.add((i + 1) as usize) as char).to_digit(16);
            let lo = (*s.add((i + 2) as usize) as char).to_digit(16);
            let val = match (hi, lo) {
                (Some(hi), Some(lo)) => (hi * 16 + lo) as i32,
                _ => 0,
            };
            crate::htslib_rs::hts::kputc(val, str_);
            i += 3;
        } else {
            crate::htslib_rs::hts::kputc(*s.add(i as usize) as i32, str_);
            i += 1;
        }
    }
}

// original: base64_kput (htslib/hfile_s3.c:181)
pub unsafe fn hfile_s3_c_181_base64_kput(data: *const u8, len: usize, str_: &mut kstring_t) {
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
        crate::htslib_rs::hts::kputc(BASE64[((x >> bits) & 63) as usize] as i32, str_);
    }

    str_.data.truncate(str_.data.len() - pad);
    crate::htslib_rs::hts::kputsn(b"==", pad, str_);
}

// original: is_dns_compliant (htslib/hfile_s3.c:206)
pub unsafe fn hfile_s3_c_206_is_dns_compliant(
    s0: *const u8,
    slim: *const u8,
    is_https: i32,
) -> i32 {
    let mut has_nondigit = 0;
    let mut len = 0;
    let mut s = s0;
    while s < slim {
        let c = *s;
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
            if s == s0 || !(*s.sub(1)).is_ascii_alphanumeric() {
                return 0;
            }
            if s.add(1) == slim || !(*s.add(1)).is_ascii_alphanumeric() {
                return 0;
            }
        } else {
            return 0;
        }
        len += 1;
        s = s.add(1);
    }

    (has_nondigit != 0 && (3..=63).contains(&len)) as i32
}

// original: expand_tilde_open (htslib/hfile_s3.c:231)
pub unsafe fn hfile_s3_c_231_expand_tilde_open(
    fname: *const u8,
    mode: *const u8,
) -> *mut libc::FILE {
    let fname_bytes = cstr_bytes(fname);
    if fname_bytes.starts_with(b"~/") {
        let mut full_fname = kstring_t::default();
        let home = libc::getenv(c"HOME".as_ptr());
        if home.is_null() {
            return std::ptr::null_mut();
        }

        crate::htslib_rs::hts::kputs(&cstr_bytes(home.cast()), &mut full_fname);
        crate::htslib_rs::hts::kputs(&fname_bytes[1..], &mut full_fname);

        // Real syscall boundary: fopen needs a NUL-terminated path.
        full_fname.data.push(0);
        libc::fopen(full_fname.data.as_ptr().cast(), mode.cast())
    } else {
        libc::fopen(fname.cast(), mode.cast())
    }
}

unsafe fn hfile_s3_c_252_parse_ini(
    fname: *const u8,
    section: *const u8,
    pairs: &[(*const u8, *mut kstring_t)],
) {
    let mut line = kstring_t::default();
    let mut active = 1;
    let section_bytes = cstr_bytes(section);
    let fp = hfile_s3_c_231_expand_tilde_open(fname, c"r".as_ptr().cast());
    if fp.is_null() {
        return;
    }

    while {
        line.data.clear();
        crate::htslib_rs::hts::kfgetline(&mut line, fp) >= 0
    } {
        // kfgetline yields a line possibly containing interior NUL; mirror the C
        // logic by treating the bytes up to the first NUL as the working line.
        let nul = line.data.iter().position(|&b| b == 0);
        let work: &[u8] = match nul {
            Some(n) => &line.data[..n],
            None => &line.data,
        };
        if work.first() == Some(&b'[') {
            if let Some(close) = work.iter().position(|&b| b == b']') {
                active = (work[1..close] == section_bytes[..]) as i32;
            }
        } else if active != 0 {
            if let Some(sep) = work.iter().position(|&b| b == b':' || b == b'=') {
                let mut key_start = 0usize;
                while key_start < sep && (work[key_start] as char).is_whitespace() {
                    key_start += 1;
                }
                let mut key_end = sep;
                while key_end > key_start && (work[key_end - 1] as char).is_whitespace() {
                    key_end -= 1;
                }
                let key = &work[key_start..key_end];

                let mut value_start = sep + 1;
                while value_start < work.len() && (work[value_start] as char).is_whitespace() {
                    value_start += 1;
                }
                let mut value_end = work.len();
                while value_end > value_start && (work[value_end - 1] as char).is_whitespace() {
                    value_end -= 1;
                }
                let value = &work[value_start..value_end];

                for &(akey, avar) in pairs {
                    if key == &cstr_bytes(akey)[..] {
                        (*avar).data.clear();
                        crate::htslib_rs::hts::kputs(value, &mut *avar);
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
    fname: *const u8,
    id: *mut kstring_t,
    secret: *mut kstring_t,
) {
    let mut text = kstring_t::default();
    let fp = hfile_s3_c_231_expand_tilde_open(fname, c"r".as_ptr().cast());
    if fp.is_null() {
        return;
    }

    while crate::htslib_rs::hts::kfgetline(&mut text, fp) >= 0 {
        crate::htslib_rs::hts::kputc(b' ' as i32, &mut text);
    }
    libc::fclose(fp);

    // Scan the first two whitespace-delimited tokens (mirroring strcspn on the
    // NUL-terminated text), operating directly on the owned bytes.
    let work: &[u8] = match text.data.iter().position(|&b| b == 0) {
        Some(n) => &text.data[..n],
        None => &text.data,
    };
    let is_ws = |b: u8| b == b' ' || b == b'\t';

    let mut start = 0usize;
    while start < work.len() && (work[start] as char).is_whitespace() {
        start += 1;
    }
    let mut end = start;
    while end < work.len() && !is_ws(work[end]) {
        end += 1;
    }
    crate::htslib_rs::hts::kputsn(&work[start..end], end - start, &mut *id);

    let mut start = end;
    while start < work.len() && (work[start] as char).is_whitespace() {
        start += 1;
    }
    let mut end = start;
    while end < work.len() && !is_ws(work[end]) {
        end += 1;
    }
    crate::htslib_rs::hts::kputsn(&work[start..end], end - start, &mut *secret);
}

unsafe fn hfile_s3_copy_auth_headers(
    ad: &mut S3AuthDataLayout,
    hdrs: *mut *mut *mut u8,
) -> i32 {
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
        // push_strdup copies a NUL-terminated run; append a NUL so the owned
        // bytes terminate before handing the pointer to the FFI boundary.
        let mut auth_bytes = ad.auth_hdr.data.clone();
        auth_bytes.push(0);
        if ad.headers.push_strdup(auth_bytes.as_ptr()).is_err() {
            ad.headers.free_all_untransferred();
            return -1;
        }
    }

    *hdrs = ad.headers.as_raw_mut_ptr();
    0
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct NullableHeaderPtr(Option<NonNull<u8>>);

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

    unsafe fn push_strdup(&mut self, text: *const u8) -> Result<(), ()> {
        // Real FFI boundary: copy the NUL-terminated run into an owned malloc'd
        // C string that libcurl reads and free_all_untransferred later frees.
        let len = std::ffi::CStr::from_ptr(text.cast()).to_bytes_with_nul().len();
        let buf = libc::malloc(len).cast::<u8>();
        let Some(value) = NonNull::new(buf) else {
            return Err(());
        };
        std::ptr::copy_nonoverlapping(text, value.as_ptr(), len);
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

    fn as_raw_mut_ptr(&mut self) -> *mut *mut u8 {
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
    creds_expiry_time: i64,
    // NUL-terminated owned bytes (was a C `CString`); read at FFI/format sites.
    bucket: Vec<u8>,
    auth_hdr: kstring_t,
    auth_time: i64,
    date: [u8; 40],
    date_long: [u8; 17],
    date_short: [u8; 9],
    date_html: kstring_t,
    mode: u8,
    headers: AuthHeaders,
    refcount: i32,
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
            bucket: vec![0],
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
    // The owned kstring_t Vec buffers (and the bucket Vec) are released when
    // this Box drops; no manual libc::free of buffer pointers is needed.
    drop(Box::from_raw(ad));
}

// original: parse_rfc3339_date (htslib/hfile_s3.c:333)
pub unsafe fn hfile_s3_c_333_parse_rfc3339_date(datetime: *mut kstring_t) -> i64 {
    let mut offset = 0;
    let mut should_be_t = 0u8;
    let mut timezone = [0u8; 10];
    let mut year = 0u32;
    let mut mon = 0u32;
    let mut day = 0u32;
    let mut hour = 0u32;
    let mut min = 0u32;
    let mut sec = 0u32;

    if (*datetime).data.is_empty() {
        return 0;
    }

    // sscanf needs a NUL-terminated C string; build one from the owned bytes.
    let mut datetime_c = (*datetime).data.clone();
    datetime_c.push(0);
    let num = libc::sscanf(
        datetime_c.as_ptr().cast(),
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
    if should_be_t != b'T' && should_be_t != b't' && should_be_t != b' ' {
        return 0;
    }

    let mut parsed = libc::tm {
        tm_sec: sec as i32,
        tm_min: min as i32,
        tm_hour: hour as i32,
        tm_mday: day as i32,
        tm_mon: mon as i32 - 1,
        tm_year: year as i32 - 1900,
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

    match timezone[0] {
        b'Z' | b'z' | 0 => {}
        b'+' | b'-' => {
            let mut hr_off = 0u32;
            let mut min_off = 0u32;
            if libc::sscanf(
                timezone.as_ptr().add(1).cast(),
                c"%2u:%2u".as_ptr(),
                &mut hr_off,
                &mut min_off,
            ) != 0
                && hr_off < 24
                && min_off <= 60
            {
                offset = ((hr_off * 60 + min_off) as i32)
                    * if timezone[0] == b'+' { -60 } else { 60 };
            }
        }
        _ => return 0,
    }

    let when = crate::htslib_rs::hts::hts_time_gm(&mut parsed);
    if when >= 0 {
        when + offset as i64
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
    let mut profile_c = (*ad).profile.data.clone();
    profile_c.push(0);
    hfile_s3_c_252_parse_ini(
        if v.is_null() {
            c"~/.aws/credentials".as_ptr().cast()
        } else {
            v.cast()
        },
        profile_c.as_ptr(),
        &[
            (c"aws_access_key_id".as_ptr().cast(), &mut (*ad).id),
            (c"aws_secret_access_key".as_ptr().cast(), &mut (*ad).secret),
            (c"aws_session_token".as_ptr().cast(), &mut (*ad).token),
            (c"expiry_time".as_ptr().cast(), &mut expiry_time),
        ],
    );
    if expiry_time.data.len() != 0 {
        (*ad).creds_expiry_time = hfile_s3_c_333_parse_rfc3339_date(&mut expiry_time);
    }
    crate::htslib_rs::hts::ks_free(&mut expiry_time);
}

// original: escape_query (htslib/hfile_s3.c:396)
unsafe fn hfile_s3_escape_query_owned(qs: &[u8]) -> Vec<u8> {
    let mut escaped = Vec::with_capacity(qs.len() * 3);
    for &c in qs {
        if c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'~' | b'.' | b'/' | b'=' | b'&')
        {
            escaped.push(c);
        } else {
            escaped.extend_from_slice(format!("%{c:02X}").as_bytes());
        }
    }
    escaped
}

// original: escape_path (htslib/hfile_s3.c:424)
unsafe fn hfile_s3_escape_path_owned(path: &[u8]) -> Vec<u8> {
    let bytes = path;
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
    escaped
}

// original: is_escaped (htslib/hfile_s3.c:460)
pub unsafe fn hfile_s3_c_460_is_escaped(str_: *const u8) -> i32 {
    let mut c = str_;
    let mut escaped = 0;
    let mut needs_escape = 0;

    while *c != 0 {
        let ch = *c;
        if ch == b'%' && *c.add(1) != 0 && *c.add(2) != 0 {
            if (*c.add(1)).is_ascii_hexdigit() && (*c.add(2)).is_ascii_hexdigit() {
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

    (escaped != 0 || needs_escape == 0) as i32
}

// original: redirect_endpoint (htslib/hfile_s3.c:488)
pub unsafe extern "C" fn hfile_s3_c_488_redirect_endpoint(
    auth: *mut c_void,
    response: i64,
    header: *mut kstring_t,
    url: *mut kstring_t,
) -> i32 {
    let ad = auth.cast::<S3AuthDataLayout>();
    let mut ret = -1;
    if header.is_null() || (*header).data.is_empty() {
        return ret;
    }

    const TAG: &[u8] = b"x-amz-bucket-region: ";
    let hdr = &(*header).data;
    if let Some(pos) = hdr.windows(TAG.len()).position(|w| w == TAG) {
        let region_start = pos + TAG.len();
        let mut end = region_start;
        while end < hdr.len()
            && (hdr[end].is_ascii_alphanumeric() || hdr[end].is_ascii_punctuation())
        {
            end += 1;
        }
        let new_region_bytes = hdr[region_start..end].to_vec();

        // Bucket region host must be an amazonaws.com endpoint.
        let host_has_aws = (*ad)
            .host
            .data
            .windows(b"amazonaws.com".len())
            .any(|w| w == b"amazonaws.com");
        if !host_has_aws {
            return ret;
        }
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
            // bucket is NUL-terminated owned bytes; append up to the NUL.
            let bucket_len = (*ad).bucket.iter().position(|&b| b == 0).unwrap_or((*ad).bucket.len());
            let bucket = &(*ad).bucket;
            crate::htslib_rs::hts::kputsn(&bucket[..bucket_len], bucket_len, &mut *url);
            if (*ad).user_query_string.data.len() != 0 {
                crate::htslib_rs::hts::kputc(b'?' as i32, &mut *url);
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
    s3url: *const u8,
    mode: *const u8,
    sigver: i32,
    url: *mut kstring_t,
) -> *mut s3_auth_data {
    // Scan a NUL-terminated run from `p` until any byte in `set` (or NUL),
    // returning the count (mirrors strcspn).
    let cspn = |mut p: *const u8, set: &[u8]| -> usize {
        let mut n = 0usize;
        while *p != 0 && !set.contains(&*p) {
            p = p.add(1);
            n += 1;
        }
        n
    };
    // Find the first byte in `set` at or after `p` (mirrors strpbrk), or null.
    let pbrk = |mut p: *const u8, set: &[u8]| -> *const u8 {
        while *p != 0 && !set.contains(&*p) {
            p = p.add(1);
        }
        if *p == 0 {
            std::ptr::null()
        } else {
            p
        }
    };

    let ad = Box::into_raw(Box::<S3AuthDataLayout>::default());
    (*ad).mode = if cstr_bytes(mode).contains(&b'r') {
        b'r'
    } else {
        b'w'
    };

    let mut is_https = 1;
    let mut address_style = 0;
    let mut bucket: *const u8;
    if *s3url.add(2) == b'+' {
        let colon = pbrk(s3url, b":");
        if colon.is_null() {
            drop(Box::from_raw(ad));
            return std::ptr::null_mut();
        }
        bucket = colon.add(1);
        let prefix_len = bucket.offset_from(s3url.add(3)) as usize;
        crate::htslib_rs::hts::kputsn(
            std::slice::from_raw_parts(s3url.add(3), prefix_len),
            prefix_len,
            &mut *url,
        );
        is_https = {
            let ud = &(*url).data;
            (ud.len() >= 6 && &ud[..6] == b"https:") as i32
        };
    } else {
        crate::htslib_rs::hts::kputs(b"https:", &mut *url);
        bucket = s3url.add(3);
    }
    while *bucket == b'/' {
        crate::htslib_rs::hts::kputc(*bucket as i32, &mut *url);
        bucket = bucket.add(1);
    }

    let mut path = bucket.add(cspn(bucket, b"/?#@"));
    if *path == b'@' {
        let colon = pbrk(bucket, b":@");
        if *colon != b':' {
            hfile_s3_c_165_urldecode_kput(
                bucket,
                colon.offset_from(bucket) as i32,
                &mut (*ad).profile,
            );
        } else {
            let colon2 = pbrk(colon.add(1), b":@");
            hfile_s3_c_165_urldecode_kput(
                bucket,
                colon.offset_from(bucket) as i32,
                &mut (*ad).id,
            );
            hfile_s3_c_165_urldecode_kput(
                colon.add(1),
                colon2.offset_from(colon.add(1)) as i32,
                &mut (*ad).secret,
            );
            if *colon2 == b':' {
                hfile_s3_c_165_urldecode_kput(
                    colon2.add(1),
                    path.offset_from(colon2.add(1)) as i32,
                    &mut (*ad).token,
                );
            }
        }
        bucket = path.add(1);
        path = bucket.add(cspn(bucket, b"/?#"));
    } else {
        let mut v = libc::getenv(c"AWS_ACCESS_KEY_ID".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v.cast()), &mut (*ad).id);
        }
        v = libc::getenv(c"AWS_SECRET_ACCESS_KEY".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v.cast()), &mut (*ad).secret);
        }
        v = libc::getenv(c"AWS_SESSION_TOKEN".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v.cast()), &mut (*ad).token);
        }
        v = libc::getenv(c"AWS_DEFAULT_REGION".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v.cast()), &mut (*ad).region);
        }
        v = libc::getenv(c"HTS_S3_HOST".as_ptr());
        if !v.is_null() {
            crate::htslib_rs::hts::kputs(&cstr_bytes(v.cast()), &mut (*ad).host);
        }
        v = libc::getenv(c"AWS_DEFAULT_PROFILE".as_ptr());
        if v.is_null() {
            v = libc::getenv(c"AWS_PROFILE".as_ptr());
        }
        if v.is_null() {
            v = c"default".as_ptr().cast_mut();
        }
        crate::htslib_rs::hts::kputs(&cstr_bytes(v.cast()), &mut (*ad).profile);
        v = libc::getenv(c"HTS_S3_ADDRESS_STYLE".as_ptr());
        if !v.is_null() {
            let style = cstr_bytes(v.cast());
            if style.eq_ignore_ascii_case(b"virtual") {
                address_style = 1;
            } else if style.eq_ignore_ascii_case(b"path") {
                address_style = 2;
            }
        }
    }

    if (*ad).id.data.len() == 0 {
        let mut url_style = kstring_t::default();
        let mut expiry_time = kstring_t::default();
        let v = libc::getenv(c"AWS_SHARED_CREDENTIALS_FILE".as_ptr());
        let mut profile_c = (*ad).profile.data.clone();
        profile_c.push(0);
        hfile_s3_c_252_parse_ini(
            if v.is_null() {
                c"~/.aws/credentials".as_ptr().cast()
            } else {
                v.cast()
            },
            profile_c.as_ptr(),
            &[
                (c"aws_access_key_id".as_ptr().cast(), &mut (*ad).id),
                (c"aws_secret_access_key".as_ptr().cast(), &mut (*ad).secret),
                (c"aws_session_token".as_ptr().cast(), &mut (*ad).token),
                (c"region".as_ptr().cast(), &mut (*ad).region),
                (c"addressing_style".as_ptr().cast(), &mut url_style),
                (c"expiry_time".as_ptr().cast(), &mut expiry_time),
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
        let mut profile_c = (*ad).profile.data.clone();
        profile_c.push(0);
        hfile_s3_c_252_parse_ini(
            if v.is_null() {
                c"~/.s3cfg".as_ptr().cast()
            } else {
                v.cast()
            },
            profile_c.as_ptr(),
            &[
                (c"access_key".as_ptr().cast(), &mut (*ad).id),
                (c"secret_key".as_ptr().cast(), &mut (*ad).secret),
                (c"access_token".as_ptr().cast(), &mut (*ad).token),
                (c"host_base".as_ptr().cast(), &mut (*ad).host),
                (c"bucket_location".as_ptr().cast(), &mut (*ad).region),
                (c"host_bucket".as_ptr().cast(), &mut url_style),
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
        hfile_s3_c_294_parse_simple(
            c"~/.awssecret".as_ptr().cast(),
            &mut (*ad).id,
            &mut (*ad).secret,
        );
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

    // `path` is the NUL-terminated remainder of the URL.
    let path_bytes = cstr_bytes(path);
    let mut escaped: Option<Vec<u8>> = None;
    if hfile_s3_c_460_is_escaped(path) == 0 {
        escaped = Some(hfile_s3_escape_path_owned(&path_bytes));
    }

    let bucket_len = path.offset_from(bucket) as usize;
    let url_path_pos: usize;
    if dns_compliant != 0 {
        let url_host_pos = (*url).data.len();
        crate::htslib_rs::hts::kputsn_(
            std::slice::from_raw_parts(bucket, bucket_len),
            bucket_len,
            &mut *url,
        );
        crate::htslib_rs::hts::kputc(b'.' as i32, &mut *url);
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
        crate::htslib_rs::hts::kputc(b'/' as i32, &mut *url);
        crate::htslib_rs::hts::kputsn(
            std::slice::from_raw_parts(bucket, bucket_len),
            bucket_len,
            &mut *url,
        );
    }
    let escaped_path: &[u8] = escaped.as_deref().unwrap_or(&path_bytes);
    crate::htslib_rs::hts::kputs(escaped_path, &mut *url);

    let mut bucket = if sigver == 4 || dns_compliant == 0 {
        (&(*url).data)[url_path_pos..].to_vec()
    } else {
        let mut bucket_bytes = Vec::with_capacity((*url).data.len() - url_path_pos + bucket_len + 1);
        bucket_bytes.push(b'/');
        bucket_bytes.extend_from_slice(std::slice::from_raw_parts(bucket, bucket_len));
        bucket_bytes.extend_from_slice(&(&(*url).data)[url_path_pos..]);
        bucket_bytes
    };
    if let Some(query_offset) = bucket.iter().position(|&b| b == b'?') {
        let query = bucket[query_offset + 1..].to_vec();
        crate::htslib_rs::hts::kputs(&query, &mut (*ad).user_query_string);
        bucket.truncate(query_offset);
    }
    // Store as NUL-terminated owned bytes so FFI/format reads see a C string.
    bucket.push(0);
    (*ad).bucket = bucket;
    ad.cast()
}

// original: v2_authorisation (htslib/hfile_s3.c:774)
pub unsafe extern "C" fn hfile_s3_c_774_v2_authorisation(
    ctx: *mut c_void,
    hdrs: *mut *mut *mut u8,
) -> i32 {
    let ad = ctx.cast::<S3AuthDataLayout>();
    let now = libc::time(std::ptr::null_mut()) as i64;
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
    let method = if (*ad).mode == b'r' { "GET" } else { "PUT" };
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
    // date is a NUL-terminated buffer; the C-string body starts at offset 6.
    let date_nul = (*ad).date.iter().position(|&b| b == 0).unwrap_or((*ad).date.len());
    let date = &(*ad).date;
    let date_body = String::from_utf8_lossy(&date[6..date_nul]).into_owned();
    let bucket_nul = (*ad).bucket.iter().position(|&b| b == 0).unwrap_or((*ad).bucket.len());
    let bucket = &(*ad).bucket;
    let bucket_str = String::from_utf8_lossy(&bucket[..bucket_nul]).into_owned();
    if kput_cstring(
        &mut message,
        format!(
            "{}\n\n\n{}\n{}{}{}{}",
            method, date_body, token_prefix, token, token_nl, bucket_str
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
    in_: *mut u8,
    length: usize,
    out: *mut u8,
    out_len: usize,
) {
    let mut hashed = [0u8; SHA256_DIGEST_BUFSIZE];
    hfile_s3_c_152_s3_sha256(in_.cast(), length, hashed.as_mut_ptr());
    // Write lowercase hex (mirrors snprintf("%02x")), NUL-terminating each step.
    for (i, byte) in hashed.iter().enumerate() {
        if i * 2 + 2 < out_len {
            let hex = format!("{byte:02x}");
            *out.add(i * 2) = hex.as_bytes()[0];
            *out.add(i * 2 + 1) = hex.as_bytes()[1];
            *out.add(i * 2 + 2) = 0;
        }
    }
}

// original: make_signature (htslib/hfile_s3.c:848)
pub unsafe fn hfile_s3_c_848_make_signature(
    ad: *mut s3_auth_data,
    string_to_sign: *mut kstring_t,
    signature_string: *mut u8,
    sig_string_len: usize,
) -> i32 {
    let ad = ad.cast::<S3AuthDataLayout>();
    let mut date_key = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut date_region_key = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut date_region_service_key = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut signing_key = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut signature = [0u8; SHA256_DIGEST_BUFSIZE];
    let mut secret_access_key = kstring_t::default();
    let mut len = 0u32;

    if kput_cstring(
        &mut secret_access_key,
        format!("AWS4{}", String::from_utf8_lossy(&(*ad).secret.data)),
    ) < 0
        || secret_access_key.data.len() == 0
    {
        return -1;
    }
    let date_short_len = (*ad)
        .date_short
        .iter()
        .position(|&b| b == 0)
        .unwrap_or((*ad).date_short.len());
    hfile_s3_c_157_s3_sign_sha256(
        secret_access_key.data.as_ptr().cast(),
        secret_access_key.data.len() as i32,
        (*ad).date_short.as_ptr().cast(),
        date_short_len as i32,
        date_key.as_mut_ptr(),
        &mut len,
    );
    hfile_s3_c_157_s3_sign_sha256(
        date_key.as_ptr().cast(),
        len as i32,
        (*ad).region.data.as_ptr().cast(),
        (*ad).region.data.len() as i32,
        date_region_key.as_mut_ptr(),
        &mut len,
    );
    hfile_s3_c_157_s3_sign_sha256(
        date_region_key.as_ptr().cast(),
        len as i32,
        c"s3".as_ptr().cast(),
        2,
        date_region_service_key.as_mut_ptr(),
        &mut len,
    );
    hfile_s3_c_157_s3_sign_sha256(
        date_region_service_key.as_ptr().cast(),
        len as i32,
        c"aws4_request".as_ptr().cast(),
        12,
        signing_key.as_mut_ptr(),
        &mut len,
    );
    hfile_s3_c_157_s3_sign_sha256(
        signing_key.as_ptr().cast(),
        len as i32,
        (*string_to_sign).data.as_ptr().cast(),
        (*string_to_sign).data.len() as i32,
        signature.as_mut_ptr(),
        &mut len,
    );
    for (i, byte) in signature.iter().take(len as usize).enumerate() {
        if i * 2 + 2 < sig_string_len {
            let hex = format!("{byte:02x}");
            *signature_string.add(i * 2) = hex.as_bytes()[0];
            *signature_string.add(i * 2 + 1) = hex.as_bytes()[1];
            *signature_string.add(i * 2 + 2) = 0;
        }
    }
    crate::htslib_rs::hts::ks_free(&mut secret_access_key);
    0
}

// original: make_authorisation (htslib/hfile_s3.c:884)
pub unsafe fn hfile_s3_c_884_make_authorisation(
    ad: *mut s3_auth_data,
    http_request: *mut u8,
    content: *mut u8,
    auth: *mut kstring_t,
) -> i32 {
    let ad = ad.cast::<S3AuthDataLayout>();
    let mut signed_headers = kstring_t::default();
    let mut canonical_headers = kstring_t::default();
    let mut canonical_request = kstring_t::default();
    let mut scope = kstring_t::default();
    let mut string_to_sign = kstring_t::default();
    let mut cr_hash = [0u8; HASH_LENGTH_SHA256];
    let mut signature_string = [0u8; HASH_LENGTH_SHA256];
    let mut ret = -1;

    // Render the NUL-terminated C-string inputs / fixed buffers as text.
    let content_str = String::from_utf8_lossy(&cstr_bytes(content)).into_owned();
    let http_request_str = String::from_utf8_lossy(&cstr_bytes(http_request)).into_owned();
    let date_long_str = String::from_utf8_lossy(&cstr_bytes((*ad).date_long.as_ptr())).into_owned();
    let date_short_str =
        String::from_utf8_lossy(&cstr_bytes((*ad).date_short.as_ptr())).into_owned();
    let bucket_str = String::from_utf8_lossy(&cstr_bytes((*ad).bucket.as_ptr())).into_owned();

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
                content_str,
                date_long_str
            ),
        );
    } else {
        kput_cstring(
            &mut canonical_headers,
            format!(
                "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\nx-amz-security-token:{}\n",
                String::from_utf8_lossy(&(*ad).host.data),
                content_str,
                date_long_str,
                String::from_utf8_lossy(&(*ad).token.data)
            ),
        );
    }
    if canonical_headers.data.len() != 0 {
        kput_cstring(
            &mut canonical_request,
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}",
                http_request_str,
                bucket_str,
                String::from_utf8_lossy(&(*ad).canonical_query_string.data),
                String::from_utf8_lossy(&canonical_headers.data),
                String::from_utf8_lossy(&signed_headers.data),
                content_str
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
                    date_short_str,
                    String::from_utf8_lossy(&(*ad).region.data)
                ),
            );
            if scope.data.len() != 0 {
                kput_cstring(
                    &mut string_to_sign,
                    format!(
                        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
                        date_long_str,
                        String::from_utf8_lossy(&scope.data),
                        String::from_utf8_lossy(&cstr_bytes(cr_hash.as_ptr()))
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
                            date_short_str,
                            String::from_utf8_lossy(&(*ad).region.data),
                            String::from_utf8_lossy(&signed_headers.data),
                            String::from_utf8_lossy(&cstr_bytes(signature_string.as_ptr()))
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
pub unsafe fn hfile_s3_c_968_update_time(ad: *mut s3_auth_data, now: i64) -> i32 {
    const AUTH_LIFETIME: i64 = 60;
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
pub fn hfile_s3_c_999_query_cmp(q1: &[u8], q2: &[u8]) -> std::cmp::Ordering {
    q1.cmp(q2)
}

// original: order_query_string (htslib/hfile_s3.c:1009)
pub unsafe fn hfile_s3_c_1009_order_query_string(qs: *mut kstring_t) -> i32 {
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

    if !ordered.data.is_empty() {
        let escaped = hfile_s3_escape_query_owned(&ordered.data);
        (*qs).data.clear();
        crate::htslib_rs::hts::kputs(&escaped, &mut *qs);
        ret = 0;
    }

    ret
}

// original: v4_authorisation (htslib/hfile_s3.c:1055)
pub unsafe extern "C" fn hfile_s3_c_1055_v4_authorisation(
    auth: *mut c_void,
    request: *mut u8,
    content: *mut kstring_t,
    cqs: *mut u8,
    hash: *mut kstring_t,
    auth_str: *mut kstring_t,
    date: *mut kstring_t,
    token: *mut kstring_t,
    uqs: i32,
) -> i32 {
    let ad = auth.cast::<S3AuthDataLayout>();
    let mut content_hash = [0u8; HASH_LENGTH_SHA256];
    if request.is_null() {
        hfile_s3_c_319_free_auth_data(ad.cast());
        return 0;
    }
    let now = libc::time(std::ptr::null_mut()) as i64;
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
            c"".as_ptr().cast_mut().cast(),
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
    hdrs: *mut *mut *mut u8,
) -> i32 {
    let ad = ctx.cast::<S3AuthDataLayout>();
    let mut content_hash = [0u8; HASH_LENGTH_SHA256];
    let mut content = kstring_t::default();
    let mut authorisation = kstring_t::default();
    let mut token_hdr = kstring_t::default();
    if hdrs.is_null() {
        hfile_s3_c_319_free_auth_data(ad.cast());
        return 0;
    }
    let now = libc::time(std::ptr::null_mut()) as i64;
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
        c"".as_ptr().cast_mut().cast(),
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
        c"GET".as_ptr().cast_mut().cast(),
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
            String::from_utf8_lossy(&cstr_bytes(content_hash.as_ptr()))
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
    // push_strdup copies a NUL-terminated run; append a NUL to the owned bytes.
    let mut date_html_c = (*ad).date_html.data.clone();
    date_html_c.push(0);
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

pub unsafe fn hfile_s3_c_1055_handle_400_response(fp: *mut hFILE, ad: *mut s3_auth_data) -> i32 {
    let ad = ad.cast::<S3AuthDataLayout>();
    let mut buffer = [0u8; 1024];
    let bytes = htslib_hfile_h_247_hread(fp, buffer.as_mut_ptr().cast(), buffer.len() - 1);
    if bytes < 0 {
        return -1;
    }
    let body = &buffer[..bytes as usize];
    let Some(tag_pos) = body.windows(b"<Region>".len()).position(|w| w == b"<Region>") else {
        return -1;
    };
    let mut start = tag_pos + b"<Region>".len();
    while start < body.len() && (body[start] as char).is_whitespace() {
        start += 1;
    }
    let Some(rel_end) = body[start..].iter().position(|&b| b == b'<') else {
        return -1;
    };
    let lt = start + rel_end;
    if !body[lt + 1..].starts_with(b"/Region>") {
        return -1;
    }
    let mut reg_end = lt;
    while reg_end > start && (body[reg_end - 1] as char).is_whitespace() {
        reg_end -= 1;
    }
    (*ad).region.data.clear();
    let region_slice = body[start..reg_end].to_vec();
    crate::htslib_rs::hts::kputsn(&region_slice, region_slice.len(), &mut (*ad).region);
    if (*ad).region.data.len() == 0 {
        -1
    } else {
        0
    }
}

// original: set_region (htslib/hfile_s3.c:1112)
pub unsafe fn hfile_s3_c_1112_set_region(ad: *mut s3_auth_data, region: *mut kstring_t) -> i32 {
    let ad = ad.cast::<S3AuthDataLayout>();
    (*ad).region.data.clear();
    let region_bytes = (*region).data.clone();
    (crate::htslib_rs::hts::kputsn(&region_bytes, region_bytes.len(), &mut (*ad).region) < 0) as i32
}

// original: stristr (htslib/hfile_s3.c:1176)
// Case-insensitive substring search; returns the byte offset of the match.
pub fn hfile_s3_c_1176_stristr(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|w| w.eq_ignore_ascii_case(needle))
}

// original: get_entry (htslib/hfile_s3.c:1198)
pub unsafe fn hfile_s3_c_1198_get_entry(
    in_: &[u8],
    start_tag: &[u8],
    end_tag: &[u8],
    out: *mut kstring_t,
) -> i32 {
    let Some(start_pos) = hfile_s3_c_1176_stristr(in_, start_tag) else {
        return libc::EOF;
    };
    let start = start_pos + start_tag.len();
    let Some(rel_end) = hfile_s3_c_1176_stristr(&in_[start..], end_tag) else {
        return libc::EOF;
    };
    let entry = &in_[start..start + rel_end];
    crate::htslib_rs::hts::kputsn(entry, entry.len(), &mut *out)
}

// original: report_s3_error (htslib/hfile_s3.c:1218)
pub unsafe fn hfile_s3_c_1218_report_s3_error(body: *mut kstring_t, resp_code: i64) -> i32 {
    let mut entry = kstring_t::default();
    let body_bytes = (*body).data.clone();

    if hfile_s3_c_1198_get_entry(&body_bytes, b"<Code>", b"</Code>", &mut entry) == libc::EOF {
        return -1;
    }

    eprintln!(
        "hfile_s3: S3 error {}: {}",
        resp_code,
        String::from_utf8_lossy(&entry.data)
    );

    entry.data.clear();

    if hfile_s3_c_1198_get_entry(&body_bytes, b"<Message>", b"</Message>", &mut entry) == libc::EOF
    {
        return -1;
    }

    if entry.data.len() != 0 {
        eprintln!("{}", String::from_utf8_lossy(&entry.data));
    }

    0
}

// original: http_status_errno (htslib/hfile_s3.c:1242)
pub unsafe fn hfile_s3_c_1242_http_status_errno(status: i32) -> i32 {
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
    value: *const u8,
) -> i32 {
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
    let mut auth_c = auth.data.clone();
    auth_c.push(0);
    let mut date_c = date.data.clone();
    date_c.push(0);
    let mut content_c = content.data.clone();
    content_c.push(0);
    let mut token_c = token.data.clone();
    token_c.push(0);

    err |= hfile_s3_add_header(&mut headers, c"Content-Type:".as_ptr().cast());
    err |= hfile_s3_add_header(&mut headers, c"Expect:".as_ptr().cast());
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
            let mut range_c = range.data.clone();
            range_c.push(0);
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

unsafe fn hfile_s3_response_code(fp: *mut hFILE_s3, response_code: *mut i64) -> i32 {
    let Some(curl) = (*fp).curl else {
        return -1;
    };
    curl_easy_getinfo_long(curl.as_ptr(), CURLINFO_RESPONSE_CODE, response_code)
}

unsafe fn hfile_s3_finish_uploaded_part(fp: *mut hFILE_s3, response: *mut kstring_t) -> i32 {
    let mut response_code: i64 = 0;
    if hfile_s3_response_code(fp, &mut response_code) != CURLE_OK || response_code > 200 {
        *libc::__errno_location() = hfile_s3_c_1242_http_status_errno(response_code as i32);
        -1
    } else {
        hfile_s3_append_completed_upload_part(fp, response)
    }
}

// original: get_upload_id (htslib/hfile_s3.c:1837)
unsafe fn hfile_s3_c_1837_get_upload_id(fp: *mut hFILE_s3, resp: *mut kstring_t) -> i32 {
    let resp_bytes = (*resp).data.clone();
    if hfile_s3_c_1198_get_entry(&resp_bytes, b"<UploadId>", b"</UploadId>", &mut (*fp).upload_id)
        == libc::EOF
    {
        -1
    } else {
        0
    }
}

unsafe fn hfile_s3_append_completed_upload_part(fp: *mut hFILE_s3, resp: *mut kstring_t) -> i32 {
    let mut etag = kstring_t::default();
    let resp_bytes = (*resp).data.clone();
    if hfile_s3_c_1198_get_entry(&resp_bytes, b"ETag: \"", b"\"", &mut etag) == libc::EOF {
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
pub unsafe fn hfile_s3_c_1417_abort_upload(fp: *mut hFILE_s3) -> i32 {
    let mut url = kstring_t::default();
    let mut canonical_query_string = kstring_t::default();
    let mut ret = -1;
    let save_errno = *libc::__errno_location();
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
    let mut cqs_c = canonical_query_string.data.clone();
    cqs_c.push(0);
    if hfile_s3_c_1055_v4_authorisation(
        au.as_ptr().cast(),
        c"DELETE".as_ptr().cast_mut().cast(),
        std::ptr::null_mut(),
        cqs_c.as_mut_ptr(),
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
    let mut url_c = url.data.clone();
    url_c.push(0);
    let mut useragent_c = (*std::ptr::addr_of!(HFILE_S3_USERAGENT.data)).clone();
    useragent_c.push(0);
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
    _ret: i32,
    save_errno: i32,
    url: *mut kstring_t,
    canonical_query_string: *mut kstring_t,
    headers: Option<NonNull<HFileLibcurlCurlSlist>>,
) {
    crate::htslib_rs::hts::ks_free(&mut *url);
    crate::htslib_rs::hts::ks_free(&mut *canonical_query_string);
    hfile_s3_free_headers(headers);
    (*fp).aborted = 1;
    hfile_s3_cleanup(&mut *fp);
    *libc::__errno_location() = save_errno;
}

// original: complete_upload (htslib/hfile_s3.c:1479)
pub unsafe fn hfile_s3_c_1479_complete_upload(fp: *mut hFILE_s3, resp: *mut kstring_t) -> i32 {
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
    let mut cqs_c = canonical_query_string.data.clone();
    cqs_c.push(0);
    if hfile_s3_c_1055_v4_authorisation(
        au.as_ptr().cast(),
        c"POST".as_ptr().cast_mut().cast(),
        &mut (*fp).completion_message,
        cqs_c.as_mut_ptr(),
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
    let mut url_c = url.data.clone();
    url_c.push(0);
    let mut useragent_c = (*std::ptr::addr_of!(HFILE_S3_USERAGENT.data)).clone();
    useragent_c.push(0);
    curl_easy_reset(curl.as_ptr());
    let mut err = curl_easy_setopt(curl.as_ptr(), CURLOPT_POST, 1i64);
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_POSTFIELDS,
        (*fp).completion_message.data.as_ptr(),
    );
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_POSTFIELDSIZE,
        (*fp).completion_message.data.len() as i64,
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
        // ptr is a libcurl-provided raw buffer; copy bytes into it at the boundary.
        let index = (*fp).index;
        let data = &(*fp).buffer.data;
        let src = &data[index..index + read_length];
        std::ptr::copy_nonoverlapping(src.as_ptr(), ptr.cast::<u8>(), read_length);
        (*fp).index += read_length;
    }
    read_length
}

// original: upload_part (htslib/hfile_s3.c:1563)
pub unsafe fn hfile_s3_c_1563_upload_part(fp: *mut hFILE_s3, resp: *mut kstring_t) -> i32 {
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
    let mut cqs_c = canonical_query_string.data.clone();
    cqs_c.push(0);
    if hfile_s3_c_1055_v4_authorisation(
        au.as_ptr().cast(),
        c"PUT".as_ptr().cast_mut().cast(),
        &mut (*fp).buffer,
        cqs_c.as_mut_ptr(),
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
    let mut url_c = url.data.clone();
    url_c.push(0);
    let mut useragent_c = (*std::ptr::addr_of!(HFILE_S3_USERAGENT.data)).clone();
    useragent_c.push(0);
    (*fp).index = 0;
    curl_easy_reset(curl.as_ptr());
    let mut err = curl_easy_setopt(curl.as_ptr(), CURLOPT_UPLOAD, 1i64);
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_READFUNCTION,
        hfile_s3_c_1546_upload_callback as usize,
    );
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_READDATA, fp.cast::<c_void>());
    err |= curl_easy_setopt(
        curl.as_ptr(),
        CURLOPT_INFILESIZE_LARGE,
        (*fp).buffer.data.len() as i64,
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
    ptr: *mut u8,
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
            std::slice::from_raw_parts(ptr, n),
            n,
            &mut (*fp).buffer,
        ) == libc::EOF
        {
            eprintln!("hfile_s3: error: unable to allocate memory to read data.");
            return 0;
        }
    }
    n
}

// original: s3_read_close (htslib/hfile_s3.c:1869)
pub unsafe fn hfile_s3_c_1869_s3_read_close(fp: &mut hFILE) -> i32 {
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
) -> isize {
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

    nbytes as isize
}

// original: s3_write_close (htslib/hfile_s3.c:1682)
pub unsafe fn hfile_s3_c_1682_s3_write_close(fp: &mut hFILE) -> i32 {
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
            const MARKER: &[u8] = b"CompleteMultipartUploadResult";
            if ret == 0
                && (response.data.is_empty()
                    || !response
                        .data
                        .windows(MARKER.len())
                        .any(|w| w == MARKER))
            {
                ret = -1;
                let mut response_code: i64 = 0;
                if hfile_s3_response_code(fp, &mut response_code) == CURLE_OK {
                    if hts_verbose >= HTS_LOG_INFO
                        && hfile_s3_c_1218_report_s3_error(&mut response, response_code) != 0
                    {
                        eprintln!(
                            "hfile_s3: warning, unable to report full S3 error status."
                        );
                    }
                    *libc::__errno_location() =
                        hfile_s3_c_1242_http_status_errno(response_code as i32);
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

unsafe fn hfile_s3_handle_bad_request(fp: *mut hFILE_s3, resp: *mut kstring_t) -> i32 {
    let mut region = kstring_t::default();
    let resp_bytes = (*resp).data.clone();
    if hfile_s3_c_1198_get_entry(&resp_bytes, b"<Region>", b"</Region>", &mut region) == libc::EOF {
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
    user_query: i32,
) -> i32 {
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
        c"POST".as_ptr().cast_mut().cast(),
        std::ptr::null_mut(),
        c"uploads=".as_ptr().cast_mut().cast(),
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
    let mut url_c = url.data.clone();
    url_c.push(0);
    let mut useragent_c = (*std::ptr::addr_of!(HFILE_S3_USERAGENT.data)).clone();
    useragent_c.push(0);
    let mut err = curl_easy_setopt(curl.as_ptr(), CURLOPT_URL, url_c.as_ptr());
    err |= curl_easy_setopt(curl.as_ptr(), CURLOPT_POST, 1i64);
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

pub unsafe fn hfile_s3_c_2072_s3_close(fp: &mut hFILE) -> i32 {
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
pub unsafe fn hfile_s3_c_2015_s3_seek(fp: &mut hFILE, offset: i64, whence: i32) -> i64 {
    let HFileBackend::S3(s3) = &mut fp.backend else {
        return -1;
    };
    let fp = &mut **s3 as *mut hFILE_s3;

    if (*fp).write != 0 {
        *libc::__errno_location() = libc::ESPIPE;
        return -1;
    }

    let origin = match whence {
        libc::SEEK_SET => 0i64,
        libc::SEEK_CUR => {
            *libc::__errno_location() = libc::ENOSYS;
            return -1;
        }
        libc::SEEK_END => {
            if (*fp).file_size < 0 {
                *libc::__errno_location() = libc::ESPIPE;
                return -1;
            }
            (*fp).file_size
        }
        _ => {
            *libc::__errno_location() = libc::EINVAL;
            return -1;
        }
    };

    let Some(pos_i64) = origin.checked_add(offset) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    if pos_i64 < 0 || ((*fp).file_size >= 0 && pos_i64 > (*fp).file_size) {
        *libc::__errno_location() = libc::EINVAL;
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

    (*fp).last_read as i64
}

// original: s3_write_open (htslib/hfile_s3.c:2102)
pub unsafe fn hfile_s3_c_2102_s3_write_open(
    url: *const u8,
    auth: *mut s3_auth_data,
) -> *mut hFILE {
    // The S3 subclass state is now an owned `Box<hFILE_s3>` (the payload of the
    // HFileBackend::S3 variant). Build it raw so the existing `(*fp).field`
    // accesses and the helper calls that take `*mut hFILE_s3` keep working; on
    // error we reclaim it with `Box::from_raw` (its Drop frees the kstrings).
    let fp = Box::into_raw(Box::new(hFILE_s3::default()));
    let Some(curl) = NonNull::new(curl_easy_init()) else {
        *libc::__errno_location() = libc::ENOMEM;
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
        // Parse leading integer from the env value (mirrors atoi).
        let env_bytes = cstr_bytes(env.as_ptr().cast());
        let env_str = String::from_utf8_lossy(&env_bytes);
        let trimmed = env_str.trim_start();
        let mut end = 0usize;
        for (i, c) in trimmed.char_indices() {
            if (i == 0 && (c == '-' || c == '+')) || c.is_ascii_digit() {
                end = i + c.len_utf8();
            } else {
                break;
            }
        }
        let parsed: i32 = trimmed[..end].parse().unwrap_or(0);
        let part_size = parsed * 1024 * 1024;
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
    let has_user_query = query_start.is_some() as i32;
    let mut response = kstring_t::default();
    let mut header = kstring_t::default();

    if hfile_s3_c_1779_initialise_upload(fp, &mut header, &mut response, has_user_query) != 0 {
        goto_write_open_error(fp, &mut response, &mut header);
        return std::ptr::null_mut();
    }

    let mut response_code: i64 = 0;
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
                eprintln!("hfile_s3: warning, unable to report full S3 error status.");
            }
            *libc::__errno_location() =
                hfile_s3_c_1242_http_status_errno(response_code as i32);
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

// Route the rewritten S3 open through the native typed-options hopen_vargs.
// `parent_opts` flattens any options forwarded from the caller's `vopen` (the
// old nested `va_list`); `opts` carries this backend's auth/redirect options.
unsafe fn hfile_s3_hopen_opts(
    url: *const u8,
    mode: *const u8,
    parent_opts: &[HFileOpt],
    opts: &[HFileOpt],
) -> *mut hFILE {
    let mut all = Vec::with_capacity(parent_opts.len() + opts.len());
    all.extend_from_slice(parent_opts);
    all.extend_from_slice(opts);

    if !cstr_bytes(mode).contains(&b':') {
        let mut mode_colon = kstring_t::default();
        if crate::htslib_rs::hts::kputs(&cstr_bytes(mode), &mut mode_colon) < 0
            || crate::htslib_rs::hts::kputc(b':' as i32, &mut mode_colon) < 0
        {
            return std::ptr::null_mut();
        }
        // hopen_vargs needs a NUL-terminated mode string; build one here.
        let mut mode_colon_c = mode_colon.data.clone();
        mode_colon_c.push(0);
        hfile_c_1317_hopen_vargs(url, mode_colon_c.as_ptr(), &all)
    } else {
        hfile_c_1317_hopen_vargs(url, mode, &all)
    }
}

unsafe fn hfile_s3_c_2348_hopen_v4_read(
    url: *const u8,
    mode: *const u8,
    parent_opts: &[HFileOpt],
    ad: *mut S3AuthDataLayout,
    http_response: *mut i64,
    fail_on_error: i32,
) -> *mut hFILE {
    let opts = [
        HFileOpt::HttpHeaderCallback(hfile_s3_c_1055_v4_auth_header_callback as usize),
        HFileOpt::HttpHeaderCallbackData(ad.cast::<()>()),
        HFileOpt::RedirectCallback(hfile_s3_c_488_redirect_endpoint as usize),
        HFileOpt::RedirectCallbackData(ad.cast::<()>()),
        HFileOpt::HttpResponsePtr(http_response),
        HFileOpt::FailOnError(fail_on_error),
    ];
    hfile_s3_hopen_opts(url, mode, parent_opts, &opts)
}

unsafe fn hfile_s3_c_774_hopen_v2_read(
    url: *const u8,
    mode: *const u8,
    parent_opts: &[HFileOpt],
    ad: *mut s3_auth_data,
) -> *mut hFILE {
    let opts = [
        HFileOpt::HttpHeaderCallback(hfile_s3_c_774_v2_authorisation as usize),
        HFileOpt::HttpHeaderCallbackData(ad.cast::<()>()),
        HFileOpt::RedirectCallback(hfile_s3_c_488_redirect_endpoint as usize),
        HFileOpt::RedirectCallbackData(ad.cast::<()>()),
    ];
    hfile_s3_hopen_opts(url, mode, parent_opts, &opts)
}

unsafe fn hfile_s3_c_2348_hopen_v4_write(
    url: *const u8,
    mode: *const u8,
    parent_opts: &[HFileOpt],
    ad: *mut S3AuthDataLayout,
) -> *mut hFILE {
    let _ = (mode, parent_opts);
    hfile_s3_c_2102_s3_write_open(url, ad.cast())
}

unsafe fn hfile_s3_c_774_s3_rewrite(
    s3url: *const u8,
    mode: *const u8,
    parent_opts: &[HFileOpt],
) -> *mut hFILE {
    let mut url = kstring_t::default();
    let Some(ad) = NonNull::new(hfile_s3_c_545_setup_auth_data(s3url, mode, 2, &mut url)) else {
        return std::ptr::null_mut();
    };
    // hopen_v2_read needs a NUL-terminated url; build one from the owned bytes.
    let mut url_c = url.data.clone();
    url_c.push(0);
    let fp = NonNull::new(hfile_s3_c_774_hopen_v2_read(
        url_c.as_ptr(),
        mode,
        parent_opts,
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
    s3url: *const u8,
    mode: *const u8,
    parent_opts: &[HFileOpt],
) -> *mut hFILE {
    let mut url = kstring_t::default();
    let Some(ad_nn) = NonNull::new(
        hfile_s3_c_545_setup_auth_data(s3url, mode, 4, &mut url).cast::<S3AuthDataLayout>(),
    ) else {
        return std::ptr::null_mut();
    };
    let ad = ad_nn.as_ptr();
    // The hopen_v4 helpers need a NUL-terminated url; build one from owned bytes.
    let mut url_c = url.data.clone();
    url_c.push(0);
    let fp: Option<NonNull<hFILE>>;
    if (*ad).mode == b'r' {
        let mut http_response: i64 = 0;
        let Some(first_fp) = NonNull::new(hfile_s3_c_2348_hopen_v4_read(
            url_c.as_ptr(),
            mode,
            parent_opts,
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
                parent_opts,
                ad,
                std::ptr::null_mut(),
                1,
            ));
        } else if http_response > 400 {
            (*ad).refcount = 1;
            *libc::__errno_location() =
                hfile_s3_c_1242_http_status_errno(http_response as i32);
            hclose_abruptly(first_fp.as_ptr());
            hfile_s3_c_319_free_auth_data(ad.cast());
            return std::ptr::null_mut();
        } else {
            fp = Some(first_fp);
        }
    } else {
        fp = NonNull::new(hfile_s3_c_2348_hopen_v4_write(url_c.as_ptr(), mode, parent_opts, ad));
    }

    let Some(fp) = fp else {
        hfile_s3_c_319_free_auth_data(ad.cast());
        return std::ptr::null_mut();
    };
    fp.as_ptr()
}

// original: s3_open_v2 (htslib/hfile_s3.c:2374)
pub unsafe fn hfile_s3_c_2374_s3_open_v2(
    s3url: *const u8,
    mode: *const u8,
    parent_opts: &[HFileOpt],
) -> *mut hFILE {
    hfile_s3_c_774_s3_rewrite(s3url, mode, parent_opts)
}

// original: hopen_s3 (htslib/hfile_s3.c:2400)
unsafe extern "C" fn hfile_s3_c_2400_hopen_s3(url: *const u8, mode: *const u8) -> *mut hFILE {
    if libc::getenv(c"HTS_S3_V2".as_ptr()).is_null() {
        hfile_s3_c_2348_s3_open_v4(url, mode, &[])
    } else {
        hfile_s3_c_2374_s3_open_v2(url, mode, &[])
    }
}

// original: vhopen_s3 (htslib/hfile_s3.c:2414)
unsafe fn hfile_s3_c_2414_vhopen_s3(
    url: *const u8,
    mode_colon: *const u8,
    opts: &[HFileOpt],
) -> *mut hFILE {
    if libc::getenv(c"HTS_S3_V2".as_ptr()).is_null() {
        hfile_s3_c_2348_s3_open_v4(url, mode_colon, opts)
    } else {
        hfile_s3_c_2374_s3_open_v2(url, mode_colon, opts)
    }
}

// original: s3_exit (htslib/hfile_s3.c:2426)
pub unsafe extern "C" fn hfile_s3_c_2426_s3_exit() {
    HFILE_S3_USERAGENT.data = Vec::new();
}

// original: PLUGIN_GLOBAL (htslib/hfile_s3.c:2436)
pub unsafe fn hfile_s3_c_2436_PLUGIN_GLOBAL(self_: *mut hFILE_plugin) -> i32 {
    static HANDLER: hFILE_scheme_handler_layout = hFILE_scheme_handler_layout {
        open: Some(hfile_s3_c_2400_hopen_s3),
        isremote: Some(crate::htslib_rs::hfile::hfile_c_1342_hfile_always_remote),
        provider: c"Amazon S3".as_ptr().cast(),
        priority: 2000 + 50,
        vopen: Some(hfile_s3_c_2414_vhopen_s3),
    };

    (*self_.cast::<hFILE_plugin_layout>()).name = c"Amazon S3".as_ptr().cast();
    (*self_.cast::<hFILE_plugin_layout>()).destroy = Some(hfile_s3_c_2426_s3_exit);
    hfile_s3_c_2426_s3_exit();
    crate::htslib_rs::kstring::ksprintf(
        &mut *std::ptr::addr_of_mut!(HFILE_S3_USERAGENT),
        b"htslib/%s",
        &[crate::htslib_rs::kstring::KsPrintfArg::Str(
            std::ffi::CStr::from_ptr(crate::htslib_rs::hts::hts_version().cast())
                .to_bytes(),
        )],
    );
    hfile_add_scheme_handler(
        c"s3".as_ptr().cast(),
        (&HANDLER as *const hFILE_scheme_handler_layout).cast::<hFILE_scheme_handler>(),
    );
    hfile_add_scheme_handler(
        c"s3+http".as_ptr().cast(),
        (&HANDLER as *const hFILE_scheme_handler_layout).cast::<hFILE_scheme_handler>(),
    );
    hfile_add_scheme_handler(
        c"s3+https".as_ptr().cast(),
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
                key.data.len() as i32,
                message.data.as_ptr().cast(),
                message.data.len() as i32,
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
                c"s3://AKID:SECRET:TOKEN@bucket-name/path/to file.bam?b=2&a=1".as_ptr().cast(),
                c"r".as_ptr().cast(),
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
                CStr::from_ptr((*ad_layout).bucket.as_ptr().cast()).to_bytes(),
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
                c"s3://AKID:SECRET:TOKEN@bucket-name/path/out.bam?z=9&a=1".as_ptr().cast(),
                c"w".as_ptr().cast(),
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
                    c"PUT".as_ptr().cast_mut().cast(),
                    &mut content,
                    c"partNumber=1&uploadId=upload-1".as_ptr().cast_mut().cast(),
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
                c"s3://AKID:SECRET:TOKEN@bucket-name/path/in.bam?z=9&a=1".as_ptr().cast(),
                c"r".as_ptr().cast(),
                4,
                &mut url,
            );
            assert!(!ad.is_null());

            let mut hdrv: *mut *mut u8 = std::ptr::null_mut();
            assert_eq!(
                hfile_s3_c_1055_v4_auth_header_callback(ad.cast(), &mut hdrv),
                0
            );
            assert!(!hdrv.is_null());

            let auth_text = CStr::from_ptr((*hdrv).cast()).to_string_lossy();
            assert!(auth_text.starts_with("Authorization: AWS4-HMAC-SHA256 Credential=AKID/"));
            assert!(auth_text.contains("/us-east-1/s3/aws4_request"));
            assert!(auth_text.contains(
                "SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-security-token"
            ));
            assert!(auth_text.contains("Signature="));
            assert!(CStr::from_ptr((*hdrv.add(1)).cast())
                .to_bytes()
                .starts_with(b"x-amz-date: "));
            assert_eq!(
                CStr::from_ptr((*hdrv.add(2)).cast()).to_bytes(),
                b"x-amz-content-sha256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
            assert_eq!(
                CStr::from_ptr((*hdrv.add(3)).cast()).to_bytes(),
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
                c"s3://AKID:SECRET@bucket-name/path/out.bam".as_ptr().cast(),
                c"w".as_ptr().cast(),
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
                c"s3://AKID:SECRET@bucket-name/path/in.bam?z=9&a=1".as_ptr().cast(),
                c"r".as_ptr().cast(),
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
                payload.len() as isize
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
                c"s3://AKID:SECRET@bucket-name/path/in.bam".as_ptr().cast(),
                c"r".as_ptr().cast(),
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
                *libc::__errno_location(),
                libc::ESPIPE
            );

            s3_of(&mut hf).write = 0;
            assert_eq!(hfile_s3_c_2015_s3_seek(&mut hf, 0, libc::SEEK_CUR), -1);
            assert_eq!(
                *libc::__errno_location(),
                libc::ENOSYS
            );

            assert_eq!(hfile_s3_c_2015_s3_seek(&mut hf, 1, libc::SEEK_END), -1);
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL
            );

            s3_of(&mut hf).file_size = -1;
            assert_eq!(hfile_s3_c_2015_s3_seek(&mut hf, 0, libc::SEEK_END), -1);
            assert_eq!(
                *libc::__errno_location(),
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
            assert_eq!(CStr::from_ptr(plugin.name.cast()).to_bytes(), b"Amazon S3");
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
        let now: i64 = 1_748_868_896;
        let mut date = [0u8; 40];
        let mut date_long = [0u8; 17];
        let mut date_short = [0u8; 9];

        write_s3_date_header(&mut date, now);
        assert_eq!(
            unsafe { CStr::from_ptr(date.as_ptr().cast()) }.to_bytes(),
            b"Date: Mon, 02 Jun 2025 12:54:56 GMT"
        );

        assert!(write_s3_v4_dates(&mut date_long, &mut date_short, now));
        assert_eq!(
            unsafe { CStr::from_ptr(date_long.as_ptr().cast()) }.to_bytes(),
            b"20250602T125456Z"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(date_short.as_ptr().cast()) }.to_bytes(),
            b"20250602"
        );
    }
}
