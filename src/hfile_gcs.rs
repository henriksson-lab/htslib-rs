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
    obj: *mut c_void,
    name: *const c_char,
    destroy: *const c_void,
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
    data: *mut c_char,
    next: *mut GcsLibcurlCurlSlist,
}

#[repr(C)]
struct GcsLibcurlHdrList {
    list: *mut GcsLibcurlCurlSlist,
    num: c_uint,
    size: c_uint,
}

#[repr(C)]
struct GcsLibcurlHeaders {
    fixed: GcsLibcurlHdrList,
    extra: GcsLibcurlHdrList,
    callback: Option<HFileLibcurlHttpHeaderCallback>,
    callback_data: *mut c_void,
    auth: *mut c_void,
    auth_hdr_num: c_int,
    redirect: *mut c_void,
    redirect_data: *mut c_void,
    http_response_ptr: *mut libc::c_long,
    fail_on_error: c_int,
}

unsafe fn hfile_gcs_c_41_build_rewrite(
    gsurl: *const c_char,
    mode: *const c_char,
    url: *mut kstring_t,
    auth_hdr: *mut kstring_t,
    requester_pays_hdr: *mut kstring_t,
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
            (*url).s,
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
    let mut headers: GcsLibcurlHeaders = std::mem::zeroed();
    headers.fail_on_error = 1;

    if !auth_hdr.is_null() {
        if crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_353_append_header(
            (&mut headers.fixed as *mut GcsLibcurlHdrList).cast(),
            auth_hdr,
            1,
        ) < 0
        {
            crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_372_free_headers(
                (&mut headers.fixed as *mut GcsLibcurlHdrList).cast(),
                1,
            );
            return std::ptr::null_mut();
        }
        headers.auth_hdr_num = -2;
    }

    if !requester_pays_hdr.is_null()
        && crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_353_append_header(
            (&mut headers.fixed as *mut GcsLibcurlHdrList).cast(),
            requester_pays_hdr,
            1,
        ) < 0
    {
        crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_372_free_headers(
            (&mut headers.fixed as *mut GcsLibcurlHdrList).cast(),
            1,
        );
        return std::ptr::null_mut();
    }

    let fp = crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1313_libcurl_open(
        url,
        mode,
        (&mut headers as *mut GcsLibcurlHeaders).cast(),
    );
    if fp.is_null() {
        crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_372_free_headers(
            (&mut headers.fixed as *mut GcsLibcurlHdrList).cast(),
            1,
        );
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
    let mut mode_colon: kstring_t = std::mem::zeroed();
    let mut url: kstring_t = std::mem::zeroed();
    let mut auth_hdr: kstring_t = std::mem::zeroed();
    let mut requester_pays_hdr: kstring_t = std::mem::zeroed();
    let fp: *mut hFILE;

    if hfile_gcs_c_41_build_rewrite(
        gsurl,
        mode,
        &mut url,
        &mut auth_hdr,
        &mut requester_pays_hdr,
    ) < 0
    {
        fp = std::ptr::null_mut();
        goto_gcs_rewrite_done(
            &mut mode_colon,
            &mut url,
            &mut auth_hdr,
            &mut requester_pays_hdr,
        );
        return fp;
    }

    if !argsp.is_null() || mode_has_colon != 0 {
        if mode_has_colon == 0 {
            kputs(mode, &mut mode_colon);
            kputc(b':' as c_int, &mut mode_colon);
            mode = mode_colon.s;
        }

        if auth_hdr.l > 0 && requester_pays_hdr.l > 0 {
            let words: [usize; 7] = [
                c"va_list".as_ptr() as usize,
                argsp as usize,
                c"httphdr:l".as_ptr() as usize,
                auth_hdr.s as usize,
                requester_pays_hdr.s as usize,
                std::ptr::null::<c_char>() as usize,
                std::ptr::null::<c_char>() as usize,
            ];
            fp = hfile_gcs_hopen_vargs(url.s, mode, &words);
        } else {
            let words: [usize; 5] = [
                c"va_list".as_ptr() as usize,
                argsp as usize,
                c"httphdr".as_ptr() as usize,
                if auth_hdr.l > 0 {
                    auth_hdr.s as usize
                } else {
                    std::ptr::null::<c_char>() as usize
                },
                std::ptr::null::<c_char>() as usize,
            ];
            fp = hfile_gcs_hopen_vargs(url.s, mode, &words);
        }
    } else if auth_hdr.l > 0 || requester_pays_hdr.l > 0 {
        fp = hfile_gcs_c_41_open_translated_libcurl(
            url.s,
            mode,
            if auth_hdr.l > 0 {
                auth_hdr.s
            } else {
                std::ptr::null()
            },
            if requester_pays_hdr.l > 0 {
                requester_pays_hdr.s
            } else {
                std::ptr::null()
            },
        );
    } else {
        fp = crate::htslib_rs::hfile::hopen(url.s, mode);
    }

    goto_gcs_rewrite_done(
        &mut mode_colon,
        &mut url,
        &mut auth_hdr,
        &mut requester_pays_hdr,
    );
    fp
}

unsafe fn goto_gcs_rewrite_done(
    mode_colon: *mut kstring_t,
    url: *mut kstring_t,
    auth_hdr: *mut kstring_t,
    requester_pays_hdr: *mut kstring_t,
) {
    libc::free((*mode_colon).s.cast());
    libc::free((*url).s.cast());
    libc::free((*auth_hdr).s.cast());
    libc::free((*requester_pays_hdr).s.cast());
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

    unsafe fn free_kstring(s: &mut kstring_t) {
        libc::free(s.s.cast());
        s.l = 0;
        s.m = 0;
        s.s = std::ptr::null_mut();
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

            let mut url: kstring_t = std::mem::zeroed();
            let mut auth: kstring_t = std::mem::zeroed();
            let mut requester: kstring_t = std::mem::zeroed();

            assert_eq!(
                hfile_gcs_c_41_build_rewrite(
                    c"gs://bucket-name/path/to.bam?generation=3".as_ptr(),
                    c"r".as_ptr(),
                    &mut url,
                    &mut auth,
                    &mut requester,
                ),
                0
            );
            assert_eq!(
                CStr::from_ptr(url.s).to_str().unwrap(),
                "https://bucket-name.storage-download.googleapis.com/path/to.bam?generation=3"
            );
            assert_eq!(
                CStr::from_ptr(auth.s).to_str().unwrap(),
                "Authorization: Bearer tok123"
            );
            assert_eq!(
                CStr::from_ptr(requester.s).to_str().unwrap(),
                "X-Goog-User-Project: proj-7"
            );

            free_kstring(&mut url);
            free_kstring(&mut auth);
            free_kstring(&mut requester);
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

            let mut url: kstring_t = std::mem::zeroed();
            let mut auth: kstring_t = std::mem::zeroed();
            let mut requester: kstring_t = std::mem::zeroed();

            assert_eq!(
                hfile_gcs_c_41_build_rewrite(
                    c"gs+http://bucket/object".as_ptr(),
                    c"w".as_ptr(),
                    &mut url,
                    &mut auth,
                    &mut requester,
                ),
                0
            );
            assert_eq!(
                CStr::from_ptr(url.s).to_str().unwrap(),
                "http://bucket.storage-upload.googleapis.com/object"
            );
            assert!(auth.s.is_null());
            assert_eq!(
                CStr::from_ptr(requester.s).to_str().unwrap(),
                "X-Goog-User-Project: billing-proj"
            );

            free_kstring(&mut url);
            free_kstring(&mut auth);
            free_kstring(&mut requester);
            clear_env();
        }
    }
}
