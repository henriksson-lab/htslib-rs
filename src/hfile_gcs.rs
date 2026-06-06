/*  hfile_gcs.c -- Google Cloud Storage backend for low-level file streams.

    Copyright (C) 2016, 2021 Genome Research Ltd.

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

use crate::htslib_rs::{
    hfile::{hFILE_plugin, hFILE_scheme_handler, hfile_add_scheme_handler},
    hts::{hFILE, hts_verbose, kputc, kputs, kputsn, kstring_t},
};
use std::ffi::{c_char, c_int, c_uint, c_void};
use std::ptr::NonNull;

type HFileOpenFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut hFILE;
type HFileIsRemoteFn = unsafe extern "C" fn(*const c_char) -> c_int;
type HFileVOpenFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE;

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
    destroy: Option<NonNull<c_void>>,
}

// Synthesize a System V AMD64 __va_list_tag from pointer-sized words so the
// recursive open can be routed through native hfile_c_1317_hopen_vargs instead
// of the C variadic hopen. Mirrors the pattern used by hfile_s3 and hts.rs.
unsafe fn hfile_gcs_hopen_vargs(
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
    crate::htslib_rs::hfile::hfile_c_1317_hopen_vargs(url, mode, &mut args)
}

type HFileLibcurlHttpHeaderCallback =
    unsafe extern "C" fn(*mut c_void, *mut *mut *mut c_char) -> c_int;

#[repr(C)]
struct GcsLibcurlCurlSlist {
    data: Option<NonNull<c_char>>,
    next: Option<NonNull<GcsLibcurlCurlSlist>>,
}

#[repr(C)]
struct GcsLibcurlHdrList {
    list: Option<NonNull<GcsLibcurlCurlSlist>>,
    num: c_uint,
    size: c_uint,
}

impl GcsLibcurlHdrList {
    fn new() -> Self {
        Self {
            list: None,
            num: 0,
            size: 0,
        }
    }

    unsafe fn append_dup(&mut self, data: *const c_char) -> c_int {
        crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_353_append_header(
            (self as *mut Self).cast(),
            data,
            1,
        )
    }

    unsafe fn free_completely(&mut self) {
        crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_372_free_headers(
            (self as *mut Self).cast(),
            1,
        );
    }
}

#[repr(C)]
struct GcsLibcurlHeaders {
    fixed: GcsLibcurlHdrList,
    extra: GcsLibcurlHdrList,
    callback: Option<HFileLibcurlHttpHeaderCallback>,
    callback_data: Option<NonNull<c_void>>,
    auth: Option<NonNull<c_void>>,
    auth_hdr_num: c_int,
    redirect: Option<NonNull<c_void>>,
    redirect_data: Option<NonNull<c_void>>,
    http_response_ptr: Option<NonNull<libc::c_long>>,
    fail_on_error: c_int,
}

impl GcsLibcurlHeaders {
    fn new_fail_on_error() -> Self {
        Self {
            fixed: GcsLibcurlHdrList::new(),
            extra: GcsLibcurlHdrList::new(),
            callback: None,
            callback_data: None,
            auth: None,
            auth_hdr_num: 0,
            redirect: None,
            redirect_data: None,
            http_response_ptr: None,
            fail_on_error: 1,
        }
    }

    unsafe fn open(&mut self, url: *const c_char, mode: *const c_char) -> *mut hFILE {
        crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1313_libcurl_open(
            url,
            mode,
            (self as *mut Self).cast(),
        )
    }
}

struct GcsKString {
    raw: kstring_t,
}

impl GcsKString {
    fn new() -> Self {
        Self {
            raw: kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
        }
    }

    fn as_mut_kstring(&mut self) -> &mut kstring_t {
        &mut self.raw
    }

    fn as_ptr(&self) -> *const c_char {
        self.raw.s
    }

    fn len(&self) -> usize {
        self.raw.l
    }
}

impl Drop for GcsKString {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.raw.s.cast());
        }
    }
}

unsafe fn hfile_gcs_c_41_build_rewrite(
    gsurl: *const c_char,
    mode: *const c_char,
    url: &mut kstring_t,
    auth_hdr: &mut kstring_t,
    requester_pays_hdr: &mut kstring_t,
) -> c_int {
    // GCS URL format is gs[+SCHEME]://BUCKET/PATH

    let mut bucket = if *gsurl.add(2) == b'+' as c_char {
        let bucket = libc::strchr(gsurl, b':' as c_int).add(1);
        if kputsn(gsurl.add(3), bucket.offset_from(gsurl.add(3)) as usize, url) < 0 {
            return -1;
        }
        bucket
    } else {
        if kputs(c"https:".as_ptr(), url) < 0 {
            return -1;
        }
        gsurl.add(3)
    };

    while *bucket == b'/' as c_char {
        if kputc(*bucket as c_int, url) < 0 {
            return -1;
        }
        bucket = bucket.add(1);
    }

    let path = bucket.add(libc::strcspn(bucket, c"/?#".as_ptr()));

    if kputsn(bucket, path.offset_from(bucket) as usize, url) < 0 {
        return -1;
    }
    if !libc::strchr(mode, b'r' as c_int).is_null() {
        if kputs(c".storage-download".as_ptr(), url) < 0 {
            return -1;
        }
    } else if !libc::strchr(mode, b'w' as c_int).is_null() {
        if kputs(c".storage-upload".as_ptr(), url) < 0 {
            return -1;
        }
    } else if kputs(c".storage".as_ptr(), url) < 0 {
        return -1;
    }
    if kputs(c".googleapis.com".as_ptr(), url) < 0 || kputs(path, url) < 0 {
        return -1;
    }

    if hts_verbose >= 8 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"[M::gcs_open] rewrote URL as %s\n".as_ptr(),
            url.s,
        );
    }

    // Preserve HTSlib's explicit GCS override.  If it is absent, translated
    // libcurl can supply refreshed bearer tokens via HTS_AUTH_LOCATION.
    let access_token = libc::getenv(c"GCS_OAUTH_TOKEN".as_ptr());

    if !access_token.is_null()
        && (kputs(c"Authorization: Bearer ".as_ptr(), auth_hdr) < 0
            || kputs(access_token, auth_hdr) < 0)
    {
        return -1;
    }

    let requester_pays_project = libc::getenv(c"GCS_REQUESTER_PAYS_PROJECT".as_ptr());

    if !requester_pays_project.is_null()
        && (kputs(c"X-Goog-User-Project: ".as_ptr(), requester_pays_hdr) < 0
            || kputs(requester_pays_project, requester_pays_hdr) < 0)
    {
        return -1;
    }

    0
}

unsafe fn hfile_gcs_c_41_open_translated_libcurl(
    url: *const c_char,
    mode: *const c_char,
    auth_hdr: *const c_char,
    requester_pays_hdr: *const c_char,
) -> *mut hFILE {
    let mut headers = GcsLibcurlHeaders::new_fail_on_error();

    if !auth_hdr.is_null() {
        if headers.fixed.append_dup(auth_hdr) < 0 {
            headers.fixed.free_completely();
            return std::ptr::null_mut();
        }
        headers.auth_hdr_num = -2;
    }

    if !requester_pays_hdr.is_null() && headers.fixed.append_dup(requester_pays_hdr) < 0 {
        headers.fixed.free_completely();
        return std::ptr::null_mut();
    }

    let fp = headers.open(url, mode);
    if fp.is_null() {
        headers.fixed.free_completely();
    }
    fp
}

// original: gcs_rewrite (htslib/hfile_gcs.c:41)
unsafe fn hfile_gcs_c_41_gcs_rewrite(
    gsurl: *const c_char,
    mut mode: *const c_char,
    mode_has_colon: c_int,
    argsp: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    let mut mode_colon = GcsKString::new();
    let mut url = GcsKString::new();
    let mut auth_hdr = GcsKString::new();
    let mut requester_pays_hdr = GcsKString::new();
    let fp: *mut hFILE;

    if hfile_gcs_c_41_build_rewrite(
        gsurl,
        mode,
        url.as_mut_kstring(),
        auth_hdr.as_mut_kstring(),
        requester_pays_hdr.as_mut_kstring(),
    ) < 0
    {
        return std::ptr::null_mut();
    }

    if !argsp.is_null() || mode_has_colon != 0 {
        if mode_has_colon == 0 {
            kputs(mode, mode_colon.as_mut_kstring());
            kputc(b':' as c_int, mode_colon.as_mut_kstring());
            mode = mode_colon.as_ptr();
        }

        if auth_hdr.len() > 0 && requester_pays_hdr.len() > 0 {
            let words: [usize; 7] = [
                c"va_list".as_ptr() as usize,
                argsp as usize,
                c"httphdr:l".as_ptr() as usize,
                auth_hdr.as_ptr() as usize,
                requester_pays_hdr.as_ptr() as usize,
                std::ptr::null::<c_char>() as usize,
                std::ptr::null::<c_char>() as usize,
            ];
            fp = hfile_gcs_hopen_vargs(url.as_ptr(), mode, &words);
        } else {
            let words: [usize; 5] = [
                c"va_list".as_ptr() as usize,
                argsp as usize,
                c"httphdr".as_ptr() as usize,
                if auth_hdr.len() > 0 {
                    auth_hdr.as_ptr() as usize
                } else {
                    std::ptr::null::<c_char>() as usize
                },
                std::ptr::null::<c_char>() as usize,
            ];
            fp = hfile_gcs_hopen_vargs(url.as_ptr(), mode, &words);
        }
    } else if auth_hdr.len() > 0 || requester_pays_hdr.len() > 0 {
        fp = hfile_gcs_c_41_open_translated_libcurl(
            url.as_ptr(),
            mode,
            if auth_hdr.len() > 0 {
                auth_hdr.as_ptr()
            } else {
                std::ptr::null()
            },
            if requester_pays_hdr.len() > 0 {
                requester_pays_hdr.as_ptr()
            } else {
                std::ptr::null()
            },
        );
    } else {
        fp = crate::htslib_rs::hfile::hopen(url.as_ptr(), mode);
    }

    fp
}

// original: gcs_open (htslib/hfile_gcs.c:125)
unsafe extern "C" fn hfile_gcs_c_125_gcs_open(
    url: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    hfile_gcs_c_41_gcs_rewrite(url, mode, 0, std::ptr::null_mut())
}

// original: gcs_vopen (htslib/hfile_gcs.c:130)
unsafe extern "C" fn hfile_gcs_c_130_gcs_vopen(
    url: *const c_char,
    mode_colon: *const c_char,
    args0: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    // Need to use va_copy() as we can only take the address of an actual
    // va_list object, not that of a parameter as its type may have decayed.
    let mut args = std::mem::MaybeUninit::<crate::htslib_rs::c_compat::__va_list_tag>::uninit();
    std::ptr::copy_nonoverlapping(args0, args.as_mut_ptr(), 1);
    hfile_gcs_c_41_gcs_rewrite(url, mode_colon, 1, args.as_mut_ptr())
}

// original: PLUGIN_GLOBAL (htslib/hfile_gcs.c:141)
pub unsafe fn hfile_gcs_c_141_PLUGIN_GLOBAL(self_: *mut hFILE_plugin) -> c_int {
    static HANDLER: hFILE_scheme_handler_layout = hFILE_scheme_handler_layout {
        open: Some(hfile_gcs_c_125_gcs_open),
        isremote: Some(crate::htslib_rs::hfile::hfile_c_1342_hfile_always_remote),
        provider: c"Google Cloud Storage".as_ptr(),
        priority: 2000 + 50,
        vopen: Some(hfile_gcs_c_130_gcs_vopen),
    };

    (*self_.cast::<hFILE_plugin_layout>()).name = c"Google Cloud Storage".as_ptr();
    hfile_add_scheme_handler(
        c"gs".as_ptr(),
        (&HANDLER as *const hFILE_scheme_handler_layout).cast::<hFILE_scheme_handler>(),
    );
    hfile_add_scheme_handler(
        c"gs+http".as_ptr(),
        (&HANDLER as *const hFILE_scheme_handler_layout).cast::<hFILE_scheme_handler>(),
    );
    hfile_add_scheme_handler(
        c"gs+https".as_ptr(),
        (&HANDLER as *const hFILE_scheme_handler_layout).cast::<hFILE_scheme_handler>(),
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    unsafe fn clear_env() {
        libc::unsetenv(c"GCS_OAUTH_TOKEN".as_ptr());
        libc::unsetenv(c"GCS_REQUESTER_PAYS_PROJECT".as_ptr());
    }

    #[test]
    fn gcs_rewrite_builds_explicit_auth_and_requester_pays_headers() {
        let _guard = env_lock();
        unsafe {
            clear_env();
            libc::setenv(c"GCS_OAUTH_TOKEN".as_ptr(), c"tok123".as_ptr(), 1);
            libc::setenv(
                c"GCS_REQUESTER_PAYS_PROJECT".as_ptr(),
                c"proj-7".as_ptr(),
                1,
            );

            let mut url = GcsKString::new();
            let mut auth = GcsKString::new();
            let mut requester = GcsKString::new();

            assert_eq!(
                hfile_gcs_c_41_build_rewrite(
                    c"gs://bucket-name/path/to.bam?generation=3".as_ptr(),
                    c"r".as_ptr(),
                    url.as_mut_kstring(),
                    auth.as_mut_kstring(),
                    requester.as_mut_kstring(),
                ),
                0
            );
            assert_eq!(
                CStr::from_ptr(url.as_ptr()).to_str().unwrap(),
                "https://bucket-name.storage-download.googleapis.com/path/to.bam?generation=3"
            );
            assert_eq!(
                CStr::from_ptr(auth.as_ptr()).to_str().unwrap(),
                "Authorization: Bearer tok123"
            );
            assert_eq!(
                CStr::from_ptr(requester.as_ptr()).to_str().unwrap(),
                "X-Goog-User-Project: proj-7"
            );

            clear_env();
        }
    }

    #[test]
    fn gcs_rewrite_leaves_auth_empty_when_gcs_token_absent() {
        let _guard = env_lock();
        unsafe {
            clear_env();
            libc::setenv(
                c"GCS_REQUESTER_PAYS_PROJECT".as_ptr(),
                c"billing-proj".as_ptr(),
                1,
            );

            let mut url = GcsKString::new();
            let mut auth = GcsKString::new();
            let mut requester = GcsKString::new();

            assert_eq!(
                hfile_gcs_c_41_build_rewrite(
                    c"gs+http://bucket/object".as_ptr(),
                    c"w".as_ptr(),
                    url.as_mut_kstring(),
                    auth.as_mut_kstring(),
                    requester.as_mut_kstring(),
                ),
                0
            );
            assert_eq!(
                CStr::from_ptr(url.as_ptr()).to_str().unwrap(),
                "http://bucket.storage-upload.googleapis.com/object"
            );
            assert!(auth.as_ptr().is_null());
            assert_eq!(
                CStr::from_ptr(requester.as_ptr()).to_str().unwrap(),
                "X-Goog-User-Project: billing-proj"
            );

            clear_env();
        }
    }
}
