#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_variables
)]

use crate::htslib_rs::{
    hfile::{hFILE_plugin, hFILE_scheme_handler},
    hts::{hFILE, hts_verbose, hts_version, kstring_t, HTS_LOG_INFO},
};
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_ulong, c_void};

static mut HFILE_S3_SHARE_LOCK: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;

const S3_AUTO: c_int = 0;
const S3_VIRTUAL: c_int = 1;
const S3_PATH: c_int = 2;
const S3_MOVED_PERMANENTLY: libc::c_long = 301;
const S3_TEMPORARY_REDIRECT: libc::c_long = 307;
const S3_BAD_REQUEST: libc::c_long = 400;
const MINIMUM_S3_WRITE_SIZE: c_int = 5_242_880;
const EXPAND_ON: c_int = 1112;
const READ_PART_SIZE: c_int = 1_048_576;

type HFileReadFn = unsafe extern "C" fn(*mut hFILE, *mut c_void, usize) -> libc::ssize_t;
type HFileWriteFn = unsafe extern "C" fn(*mut hFILE, *const c_void, usize) -> libc::ssize_t;
type HFileSeekFn = unsafe extern "C" fn(*mut hFILE, libc::off_t, c_int) -> libc::off_t;
type HFileFlushFn = unsafe extern "C" fn(*mut hFILE) -> c_int;
type HFileCloseFn = unsafe extern "C" fn(*mut hFILE) -> c_int;

#[repr(C)]
struct hFILE_backend {
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
struct HfileS3AuthStringsPrefix {
    base: HFileLayout,
    curl: *mut c_void,
    ret: c_int,
    au: *mut s3_auth_data,
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
    url_style: c_int,
    creds_expiry_time: libc::time_t,
    bucket: *mut c_char,
    auth_time: libc::time_t,
    date: [c_char; 40],
    date_long: [c_char; 17],
    date_short: [c_char; 9],
    date_html: kstring_t,
    mode: c_char,
    is_v4: c_int,
}

// original: s3_sign (htslib/hfile_s3.c:114)
pub unsafe fn hfile_s3_c_114_s3_sign() {}

// original: s3_sha256 (htslib/hfile_s3.c:122)
pub unsafe fn hfile_s3_c_122_s3_sha256() {}

// original: s3_sign_sha256 (htslib/hfile_s3.c:127)
pub unsafe fn hfile_s3_c_127_s3_sign_sha256() {}

// original: s3_sign (htslib/hfile_s3.c:142)
pub unsafe fn hfile_s3_c_142_s3_sign() {}

// original: s3_sha256 (htslib/hfile_s3.c:152)
pub unsafe fn hfile_s3_c_152_s3_sha256() {}

// original: s3_sign_sha256 (htslib/hfile_s3.c:157)
pub unsafe fn hfile_s3_c_157_s3_sign_sha256() {}

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

// original: parse_ini (htslib/hfile_s3.c:252)
pub unsafe fn hfile_s3_c_252_parse_ini() {}

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
    url_style: c_int,
    creds_expiry_time: libc::time_t,
    bucket: *mut c_char,
    auth_time: libc::time_t,
    date: [c_char; 40],
    date_long: [c_char; 17],
    date_short: [c_char; 9],
    date_html: kstring_t,
    mode: c_char,
    is_v4: c_int,
}

// original: free_auth_data (htslib/hfile_s3.c:319)
pub unsafe fn hfile_s3_c_319_free_auth_data(ad: *mut s3_auth_data) {
    let ad = ad.cast::<S3AuthDataLayout>();
    libc::free((*ad).profile.s.cast());
    libc::free((*ad).id.s.cast());
    libc::free((*ad).token.s.cast());
    libc::free((*ad).secret.s.cast());
    libc::free((*ad).region.s.cast());
    libc::free((*ad).canonical_query_string.s.cast());
    libc::free((*ad).user_query_string.s.cast());
    libc::free((*ad).host.s.cast());
    libc::free((*ad).bucket.cast());
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
pub unsafe fn hfile_s3_c_378_refresh_auth_data() {}

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
pub unsafe fn hfile_s3_c_488_redirect_endpoint() {}

// original: setup_auth_data (htslib/hfile_s3.c:545)
pub unsafe fn hfile_s3_c_545_setup_auth_data() {}

// original: v2_authorisation (htslib/hfile_s3.c:774)
pub unsafe fn hfile_s3_c_774_v2_authorisation() {}

// original: hash_string (htslib/hfile_s3.c:836)
pub unsafe fn hfile_s3_c_836_hash_string() {}

// original: make_signature (htslib/hfile_s3.c:848)
pub unsafe fn hfile_s3_c_848_make_signature() {}

// original: make_authorisation (htslib/hfile_s3.c:884)
pub unsafe fn hfile_s3_c_884_make_authorisation() {}

// original: update_time (htslib/hfile_s3.c:968)
pub unsafe fn hfile_s3_c_968_update_time(ad: *mut s3_auth_data, now: libc::time_t) -> c_int {
    const AUTH_LIFETIME: libc::time_t = 60;
    let ad = ad.cast::<S3AuthDataLayout>();
    let mut ret = -1;
    let tm = libc::gmtime(&now);

    if now - (*ad).auth_time > AUTH_LIFETIME {
        (*ad).auth_time = now;

        if libc::strftime(
            (*ad).date_long.as_mut_ptr(),
            17,
            c"%Y%m%dT%H%M%SZ".as_ptr(),
            tm,
        ) != 16
        {
            return -1;
        }

        if libc::strftime((*ad).date_short.as_mut_ptr(), 9, c"%Y%m%d".as_ptr(), tm) != 8 {
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
pub unsafe fn hfile_s3_c_1055_v4_authorisation() {}

// original: set_region (htslib/hfile_s3.c:1112)
pub unsafe fn hfile_s3_c_1112_set_region(ad: *mut s3_auth_data, region: *mut kstring_t) -> c_int {
    let ad = ad.cast::<S3AuthDataLayout>();
    (*ad).region.l = 0;
    (crate::htslib_rs::hts::kputsn((*region).s, (*region).l, &mut (*ad).region) < 0) as c_int
}

// original: share_lock (htslib/hfile_s3.c:1133)
pub unsafe fn hfile_s3_c_1133_share_lock() {
    libc::pthread_mutex_lock(std::ptr::addr_of_mut!(HFILE_S3_SHARE_LOCK));
}

// original: share_unlock (htslib/hfile_s3.c:1138)
pub unsafe fn hfile_s3_c_1138_share_unlock() {
    libc::pthread_mutex_unlock(std::ptr::addr_of_mut!(HFILE_S3_SHARE_LOCK));
}

// original: initialise_authorisation_values (htslib/hfile_s3.c:1143)
pub unsafe fn hfile_s3_c_1143_initialise_authorisation_values(fp: *mut c_void) {
    let fp = fp.cast::<HfileS3AuthStringsPrefix>();
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).content_hash);
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).authorisation);
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).content);
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).date);
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).token);
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).range);
}

// original: clear_authorisation_values (htslib/hfile_s3.c:1153)
pub unsafe fn hfile_s3_c_1153_clear_authorisation_values(fp: *mut c_void) {
    let fp = fp.cast::<HfileS3AuthStringsPrefix>();
    crate::htslib_rs::hts::ks_clear(&mut (*fp).content_hash);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).authorisation);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).content);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).date);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).token);
    crate::htslib_rs::hts::ks_clear(&mut (*fp).range);
}

// original: free_authorisation_values (htslib/hfile_s3.c:1163)
pub unsafe fn hfile_s3_c_1163_free_authorisation_values(fp: *mut c_void) {
    let fp = fp.cast::<HfileS3AuthStringsPrefix>();
    crate::htslib_rs::hts::ks_free(&mut (*fp).content_hash);
    crate::htslib_rs::hts::ks_free(&mut (*fp).authorisation);
    crate::htslib_rs::hts::ks_free(&mut (*fp).content);
    crate::htslib_rs::hts::ks_free(&mut (*fp).date);
    crate::htslib_rs::hts::ks_free(&mut (*fp).token);
    crate::htslib_rs::hts::ks_free(&mut (*fp).range);
}

// original: stristr (htslib/hfile_s3.c:1176)
pub unsafe fn hfile_s3_c_1176_stristr(
    mut haystack: *mut c_char,
    needle: *mut c_char,
) -> *mut c_char {
    while *haystack != 0 {
        let mut h = haystack;
        let mut n = needle;

        while crate::htslib_rs::hts::toupper_c(*h) == crate::htslib_rs::hts::toupper_c(*n) {
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
pub unsafe fn hfile_s3_c_1218_report_s3_error() {}

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

// original: initialise_local (htslib/hfile_s3.c:1268)
pub unsafe fn hfile_s3_c_1268_initialise_local(fp: *mut c_void) {
    let fp = fp.cast::<HfileS3AuthStringsPrefix>();
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).buffer);
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).url);
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).upload_id);
    crate::htslib_rs::hts::ks_initialize(&mut (*fp).completion_message);
}

// original: cleanup_local (htslib/hfile_s3.c:1276)
pub unsafe fn hfile_s3_c_1276_cleanup_local(fp: *mut c_void) {
    let fp = fp.cast::<HfileS3AuthStringsPrefix>();
    crate::htslib_rs::hts::ks_free(&mut (*fp).buffer);
    crate::htslib_rs::hts::ks_free(&mut (*fp).url);
    crate::htslib_rs::hts::ks_free(&mut (*fp).upload_id);
    crate::htslib_rs::hts::ks_free(&mut (*fp).completion_message);
    curl_easy_cleanup((*fp).curl.cast());
    hfile_s3_c_1163_free_authorisation_values(fp.cast());
}

// original: cleanup (htslib/hfile_s3.c:1286)
pub unsafe fn hfile_s3_c_1286_cleanup(fp: *mut c_void) {
    let fp_s3 = fp.cast::<HfileS3AuthStringsPrefix>();
    if !(*fp_s3).au.is_null() {
        hfile_s3_c_319_free_auth_data((*fp_s3).au.cast());
    }
    (*fp_s3).au = std::ptr::null_mut();
    hfile_s3_c_1276_cleanup_local(fp);
}

// original: response_callback (htslib/hfile_s3.c:1292)
pub unsafe fn hfile_s3_c_1292_response_callback(
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

// original: add_header (htslib/hfile_s3.c:1304)
pub unsafe fn hfile_s3_c_1304_add_header(head: *mut *mut c_void, value: *mut c_char) -> c_int {
    let head = head.cast::<*mut HFileLibcurlCurlSlist>();
    let tmp = libc::calloc(1, std::mem::size_of::<HFileLibcurlCurlSlist>())
        .cast::<HFileLibcurlCurlSlist>();
    if tmp.is_null() {
        return 1;
    }
    (*tmp).data = libc::strdup(value);
    if (*tmp).data.is_null() {
        libc::free(tmp.cast());
        return 1;
    }

    if (*head).is_null() {
        *head = tmp;
    } else {
        let mut tail = *head;
        while !(*tail).next.is_null() {
            tail = (*tail).next;
        }
        (*tail).next = tmp;
    }
    0
}

unsafe fn free_curl_slist(mut head: *mut HFileLibcurlCurlSlist) {
    while !head.is_null() {
        let next = (*head).next;
        libc::free((*head).data.cast());
        libc::free(head.cast());
        head = next;
    }
}

#[link(name = "curl")]
unsafe extern "C" {
    fn curl_easy_cleanup(curl: *mut crate::htslib_rs::ref_cache::upstream::CURL);
}

// original: set_html_headers (htslib/hfile_s3.c:1318)
pub unsafe fn hfile_s3_c_1318_set_html_headers(
    _fp: *mut c_void,
    auth: *mut kstring_t,
    date: *mut kstring_t,
    content: *mut kstring_t,
    token: *mut kstring_t,
    range: *mut kstring_t,
) -> *mut c_void {
    let mut headers: *mut c_void = std::ptr::null_mut();
    let mut err = hfile_s3_c_1304_add_header(&mut headers, c"Content-Type:".as_ptr().cast_mut());
    err |= hfile_s3_c_1304_add_header(&mut headers, c"Expect:".as_ptr().cast_mut());

    if err == 0 && !auth.is_null() && (*auth).l != 0 {
        err = hfile_s3_c_1304_add_header(&mut headers, (*auth).s);
    }
    if err == 0 && !date.is_null() {
        err = hfile_s3_c_1304_add_header(&mut headers, (*date).s);
    }
    if err == 0 && !content.is_null() && (*content).l != 0 {
        err = hfile_s3_c_1304_add_header(&mut headers, (*content).s);
    }
    if err == 0 && !range.is_null() {
        err = hfile_s3_c_1304_add_header(&mut headers, (*range).s);
    }
    if err == 0 && !token.is_null() && (*token).l != 0 {
        err = hfile_s3_c_1304_add_header(&mut headers, (*token).s);
    }

    if err != 0 {
        free_curl_slist(headers.cast());
        std::ptr::null_mut()
    } else {
        headers
    }
}

// original: abort_upload (htslib/hfile_s3.c:1417)
pub unsafe fn hfile_s3_c_1417_abort_upload() {}

// original: complete_upload (htslib/hfile_s3.c:1479)
pub unsafe fn hfile_s3_c_1479_complete_upload() {}

// original: upload_callback (htslib/hfile_s3.c:1545)
pub unsafe fn hfile_s3_c_1545_upload_callback(
    ptr: *mut c_void,
    size: usize,
    nmemb: usize,
    stream: *mut c_void,
) -> usize {
    let realsize = size.saturating_mul(nmemb);
    let fp = stream.cast::<HfileS3AuthStringsPrefix>();
    let available = (*fp).buffer.l.saturating_sub((*fp).index);
    let read_length = if realsize > available {
        available
    } else {
        realsize
    };

    if read_length != 0 {
        libc::memcpy(ptr, (*fp).buffer.s.add((*fp).index).cast(), read_length);
    }
    (*fp).index += read_length;

    read_length
}

// original: upload_part (htslib/hfile_s3.c:1563)
pub unsafe fn hfile_s3_c_1563_upload_part() {}

// original: s3_write (htslib/hfile_s3.c:1625)
pub unsafe fn hfile_s3_c_1625_s3_write() {}

// original: s3_write_close (htslib/hfile_s3.c:1682)
pub unsafe fn hfile_s3_c_1682_s3_write_close() {}

// original: handle_bad_request (htslib/hfile_s3.c:1762)
pub unsafe fn hfile_s3_c_1762_handle_bad_request() {}

// original: initialise_upload (htslib/hfile_s3.c:1779)
pub unsafe fn hfile_s3_c_1779_initialise_upload() {}

// original: get_upload_id (htslib/hfile_s3.c:1837)
pub unsafe fn hfile_s3_c_1837_get_upload_id() {}

// original: recv_callback (htslib/hfile_s3.c:1854)
pub unsafe fn hfile_s3_c_1854_recv_callback(
    ptr: *mut c_char,
    size: usize,
    nmemb: usize,
    fpv: *mut c_void,
) -> usize {
    let fp = fpv.cast::<HfileS3AuthStringsPrefix>();
    let n = size.saturating_mul(nmemb);

    if n != 0 && crate::htslib_rs::hts::kputsn(ptr, n, &mut (*fp).buffer) == libc::EOF {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"hfile_s3: error: unable to allocate memory to read data.\n".as_ptr(),
        );
        return 0;
    }

    n
}

// original: s3_read_close (htslib/hfile_s3.c:1869)
pub unsafe fn hfile_s3_c_1869_s3_read_close() {}

// original: get_part (htslib/hfile_s3.c:1878)
pub unsafe fn hfile_s3_c_1878_get_part() {}

// original: s3_read (htslib/hfile_s3.c:1949)
pub unsafe fn hfile_s3_c_1949_s3_read() {}

// original: s3_seek (htslib/hfile_s3.c:2015)
pub unsafe fn hfile_s3_c_2015_s3_seek() {}

// original: initialise_download (htslib/hfile_s3.c:2074)
pub unsafe fn hfile_s3_c_2074_initialise_download() {}

// original: s3_close (htslib/hfile_s3.c:2083)
pub unsafe fn hfile_s3_c_2083_s3_close() {}

// original: s3_write_open (htslib/hfile_s3.c:2102)
pub unsafe fn hfile_s3_c_2102_s3_write_open() {}

// original: s3_read_open (htslib/hfile_s3.c:2230)
pub unsafe fn hfile_s3_c_2230_s3_read_open() {}

// original: s3_open_v4 (htslib/hfile_s3.c:2348)
pub unsafe fn hfile_s3_c_2348_s3_open_v4() {}

// original: s3_open_v2 (htslib/hfile_s3.c:2374)
pub unsafe fn hfile_s3_c_2374_s3_open_v2() {}

// original: hopen_s3 (htslib/hfile_s3.c:2400)
pub unsafe fn hfile_s3_c_2400_hopen_s3() {}

// original: vhopen_s3 (htslib/hfile_s3.c:2414)
pub unsafe fn hfile_s3_c_2414_vhopen_s3() {}

// original: s3_exit (htslib/hfile_s3.c:2426)
pub unsafe fn hfile_s3_c_2426_s3_exit() {}

// original: PLUGIN_GLOBAL (htslib/hfile_s3.c:2436)
pub unsafe fn hfile_s3_c_2436_PLUGIN_GLOBAL() {}
