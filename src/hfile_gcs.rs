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
    hts::{hFILE, hts_verbose},
};
use std::ffi::c_char;
use std::ptr::NonNull;

type HFileOpenFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut hFILE;
type HFileIsRemoteFn = unsafe extern "C" fn(*const c_char) -> i32;
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
    priority: i32,
    vopen: Option<HFileVOpenFn>,
}

unsafe impl Sync for hFILE_scheme_handler_layout {}

#[repr(C)]
struct hFILE_plugin_layout {
    api_version: i32,
    obj: Option<NonNull<()>>,
    name: *const c_char,
    destroy: Option<NonNull<()>>,
}

// Synthesize a System V AMD64 __va_list_tag from pointer-sized words so the
// recursive open can be routed through native hfile_c_1317_hopen_vargs instead
// of the C variadic hopen. Mirrors the pattern used by hfile_s3 and hts.rs.
unsafe fn hfile_gcs_hopen_vargs(url: &[u8], mode: &[u8], words: &[usize]) -> *mut hFILE {
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
    crate::htslib_rs::hfile::hfile_c_1317_hopen_vargs(
        url.as_ptr().cast(),
        mode.as_ptr().cast(),
        &mut args,
    )
}

enum GcsOpenArg<'a> {
    VaList(Option<NonNull<crate::htslib_rs::c_compat::__va_list_tag>>),
    HttpHeader(Option<&'a [u8]>),
    HttpHeaderList(&'a [u8], &'a [u8]),
}

impl GcsOpenArg<'_> {
    fn push_words(&self, words: &mut Vec<usize>) {
        match self {
            Self::VaList(argsp) => {
                words.push(c"va_list".as_ptr() as usize);
                words.push(argsp.map_or(std::ptr::null_mut(), NonNull::as_ptr) as usize);
            }
            Self::HttpHeader(header) => {
                words.push(c"httphdr".as_ptr() as usize);
                words.push(header.map_or(std::ptr::null(), <[u8]>::as_ptr) as usize);
                words.push(std::ptr::null::<u8>() as usize);
            }
            Self::HttpHeaderList(auth_hdr, requester_pays_hdr) => {
                words.push(c"httphdr:l".as_ptr() as usize);
                words.push(auth_hdr.as_ptr() as usize);
                words.push(requester_pays_hdr.as_ptr() as usize);
                words.push(std::ptr::null::<u8>() as usize);
                words.push(std::ptr::null::<u8>() as usize);
            }
        }
    }
}

unsafe fn hfile_gcs_hopen_with_args(
    url: &[u8],
    mode: &[u8],
    first: GcsOpenArg<'_>,
    second: GcsOpenArg<'_>,
) -> *mut hFILE {
    let mut words = Vec::with_capacity(7);
    first.push_words(&mut words);
    second.push_words(&mut words);
    hfile_gcs_hopen_vargs(url, mode, &words)
}

impl crate::htslib_rs::hfile_libcurl::HFileLibcurlHeaders {
    unsafe fn gcs_open(&mut self, url: &[u8], mode: &[u8]) -> *mut hFILE {
        crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1313_libcurl_open(
            url.as_ptr().cast(),
            mode.as_ptr().cast(),
            (self as *mut Self).cast(),
        )
    }
}

// Owned, NUL-terminated byte strings for the rewritten URL and headers.
struct GcsRewrite {
    url: Vec<u8>,
    auth_hdr: Option<Vec<u8>>,
    requester_pays_hdr: Option<Vec<u8>>,
}

fn gcs_auth_header_from_env() -> Option<Vec<u8>> {
    let value = unsafe { libc::getenv(c"GCS_OAUTH_TOKEN".as_ptr()) };
    NonNull::new(value).map(|value| {
        let value = unsafe { std::ffi::CStr::from_ptr(value.as_ptr()).to_bytes() };
        let mut header = Vec::with_capacity(b"Authorization: Bearer ".len() + value.len() + 1);
        header.extend_from_slice(b"Authorization: Bearer ");
        header.extend_from_slice(value);
        header.push(0);
        header
    })
}

fn gcs_requester_pays_header_from_env() -> Option<Vec<u8>> {
    let value = unsafe { libc::getenv(c"GCS_REQUESTER_PAYS_PROJECT".as_ptr()) };
    NonNull::new(value).map(|value| {
        let value = unsafe { std::ffi::CStr::from_ptr(value.as_ptr()).to_bytes() };
        let mut header = Vec::with_capacity(b"X-Goog-User-Project: ".len() + value.len() + 1);
        header.extend_from_slice(b"X-Goog-User-Project: ");
        header.extend_from_slice(value);
        header.push(0);
        header
    })
}

// `gsurl` and `mode` are NUL-terminated byte strings.
fn hfile_gcs_c_41_build_rewrite(gsurl: &[u8], mode: &[u8]) -> Option<GcsRewrite> {
    // GCS URL format is gs[+SCHEME]://BUCKET/PATH
    let gsurl_bytes = &gsurl[..gsurl.len() - 1];
    let mode_bytes = &mode[..mode.len() - 1];
    let mut url = Vec::with_capacity(gsurl_bytes.len() + 48);

    let mut bucket = if gsurl_bytes.get(2) == Some(&b'+') {
        let scheme = gsurl_bytes.get(3..)?;
        let colon = scheme.iter().position(|&ch| ch == b':')? + 3;
        url.extend_from_slice(&gsurl_bytes[3..=colon]);
        colon + 1
    } else {
        url.extend_from_slice(b"https:");
        3
    };

    while gsurl_bytes.get(bucket) == Some(&b'/') {
        url.push(b'/');
        bucket += 1;
    }

    let path = gsurl_bytes[bucket..]
        .iter()
        .position(|&ch| matches!(ch, b'/' | b'?' | b'#'))
        .map_or(gsurl_bytes.len(), |offset| bucket + offset);

    url.extend_from_slice(&gsurl_bytes[bucket..path]);
    if mode_bytes.contains(&b'r') {
        url.extend_from_slice(b".storage-download");
    } else if mode_bytes.contains(&b'w') {
        url.extend_from_slice(b".storage-upload");
    } else {
        url.extend_from_slice(b".storage");
    }
    url.extend_from_slice(b".googleapis.com");
    url.extend_from_slice(&gsurl_bytes[path..]);
    if url.contains(&0) {
        return None;
    }
    url.push(0);

    if unsafe { hts_verbose } >= 8 {
        eprintln!(
            "[M::gcs_open] rewrote URL as {}",
            String::from_utf8_lossy(&url[..url.len() - 1])
        );
    }

    Some(GcsRewrite {
        url,
        auth_hdr: gcs_auth_header_from_env(),
        requester_pays_hdr: gcs_requester_pays_header_from_env(),
    })
}

unsafe fn hfile_gcs_c_41_open_translated_libcurl(
    url: &[u8],
    mode: &[u8],
    auth_hdr: Option<&[u8]>,
    requester_pays_hdr: Option<&[u8]>,
) -> *mut hFILE {
    let mut headers = crate::htslib_rs::hfile_libcurl::HFileLibcurlHeaders::default();

    if let Some(auth_hdr) = auth_hdr {
        if crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_353_append_header(
            &mut headers.fixed,
            auth_hdr.as_ptr().cast(),
            1,
        ) < 0
        {
            crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_372_free_headers(
                &mut headers.fixed,
                1,
            );
            return std::ptr::null_mut();
        }
        headers.auth_hdr_num = -2;
    }

    if requester_pays_hdr
        .map(|hdr| {
            crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_353_append_header(
                &mut headers.fixed,
                hdr.as_ptr().cast(),
                1,
            ) < 0
        })
        .unwrap_or(false)
    {
        crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_372_free_headers(&mut headers.fixed, 1);
        return std::ptr::null_mut();
    }

    let fp = headers.gcs_open(url, mode);
    if fp.is_null() {
        crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_372_free_headers(&mut headers.fixed, 1);
    }
    fp
}

// original: gcs_rewrite (htslib/hfile_gcs.c:41)
// `gsurl` and `mode` are NUL-terminated byte strings.
unsafe fn hfile_gcs_c_41_gcs_rewrite(
    gsurl: &[u8],
    mode: &[u8],
    mode_has_colon: bool,
    argsp: Option<NonNull<crate::htslib_rs::c_compat::__va_list_tag>>,
) -> *mut hFILE {
    let Some(rewrite) = hfile_gcs_c_41_build_rewrite(gsurl, mode) else {
        return std::ptr::null_mut();
    };

    if argsp.is_some() || mode_has_colon {
        let mode_colon_storage;
        let mode_for_open: &[u8] = if mode_has_colon {
            mode
        } else {
            let mode_str = &mode[..mode.len() - 1];
            let mut mode_colon = Vec::with_capacity(mode_str.len() + 2);
            mode_colon.extend_from_slice(mode_str);
            mode_colon.push(b':');
            if mode_colon.contains(&0) {
                return std::ptr::null_mut();
            }
            mode_colon.push(0);
            mode_colon_storage = mode_colon;
            &mode_colon_storage
        };

        hfile_gcs_c_41_gcs_rewrite_with_mode_colon(&rewrite, mode_for_open, argsp)
    } else if rewrite.auth_hdr.is_some() || rewrite.requester_pays_hdr.is_some() {
        hfile_gcs_c_41_open_translated_libcurl(
            &rewrite.url,
            mode,
            rewrite.auth_hdr.as_deref(),
            rewrite.requester_pays_hdr.as_deref(),
        )
    } else {
        crate::htslib_rs::hfile::hopen(rewrite.url.as_ptr().cast(), mode.as_ptr().cast())
    }
}

// `mode_colon` is a NUL-terminated byte string.
unsafe fn hfile_gcs_c_41_gcs_rewrite_with_mode_colon(
    rewrite: &GcsRewrite,
    mode_colon: &[u8],
    argsp: Option<NonNull<crate::htslib_rs::c_compat::__va_list_tag>>,
) -> *mut hFILE {
    let auth_hdr = rewrite.auth_hdr.as_deref();
    match (auth_hdr, rewrite.requester_pays_hdr.as_deref()) {
        (Some(auth_hdr), Some(requester_pays_hdr)) => hfile_gcs_hopen_with_args(
            &rewrite.url,
            mode_colon,
            GcsOpenArg::VaList(argsp),
            GcsOpenArg::HttpHeaderList(auth_hdr, requester_pays_hdr),
        ),
        (Some(header), None) | (None, Some(header)) => hfile_gcs_hopen_with_args(
            &rewrite.url,
            mode_colon,
            GcsOpenArg::VaList(argsp),
            GcsOpenArg::HttpHeader(Some(header)),
        ),
        _ => hfile_gcs_hopen_with_args(
            &rewrite.url,
            mode_colon,
            GcsOpenArg::VaList(argsp),
            GcsOpenArg::HttpHeader(auth_hdr),
        ),
    }
}

// original: gcs_open (htslib/hfile_gcs.c:125)
unsafe extern "C" fn hfile_gcs_c_125_gcs_open(
    url: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    if url.is_null() || mode.is_null() {
        return std::ptr::null_mut();
    }

    hfile_gcs_c_41_gcs_rewrite(
        std::ffi::CStr::from_ptr(url).to_bytes_with_nul(),
        std::ffi::CStr::from_ptr(mode).to_bytes_with_nul(),
        false,
        None,
    )
}

// original: gcs_vopen (htslib/hfile_gcs.c:130)
unsafe extern "C" fn hfile_gcs_c_130_gcs_vopen(
    url: *const c_char,
    mode_colon: *const c_char,
    args0: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    if url.is_null() || mode_colon.is_null() {
        return std::ptr::null_mut();
    }

    // Need to use va_copy() as we can only take the address of an actual
    // va_list object, not that of a parameter as its type may have decayed.
    let mut args = std::mem::MaybeUninit::<crate::htslib_rs::c_compat::__va_list_tag>::uninit();
    let argsp = if args0.is_null() {
        None
    } else {
        std::ptr::copy_nonoverlapping(args0, args.as_mut_ptr(), 1);
        NonNull::new(args.as_mut_ptr())
    };
    hfile_gcs_c_41_gcs_rewrite(
        std::ffi::CStr::from_ptr(url).to_bytes_with_nul(),
        std::ffi::CStr::from_ptr(mode_colon).to_bytes_with_nul(),
        true,
        argsp,
    )
}

// original: PLUGIN_GLOBAL (htslib/hfile_gcs.c:141)
unsafe fn hfile_gcs_plugin_global(plugin: &mut hFILE_plugin_layout) -> i32 {
    static HANDLER: hFILE_scheme_handler_layout = hFILE_scheme_handler_layout {
        open: Some(hfile_gcs_c_125_gcs_open),
        isremote: Some(crate::htslib_rs::hfile::hfile_c_1342_hfile_always_remote),
        provider: c"Google Cloud Storage".as_ptr(),
        priority: 2000 + 50,
        vopen: Some(hfile_gcs_c_130_gcs_vopen),
    };

    plugin.name = c"Google Cloud Storage".as_ptr();
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

pub unsafe fn hfile_gcs_c_141_PLUGIN_GLOBAL(self_: *mut hFILE_plugin) -> i32 {
    match self_.cast::<hFILE_plugin_layout>().as_mut() {
        Some(plugin) => hfile_gcs_plugin_global(plugin),
        None => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

            let rewrite = hfile_gcs_c_41_build_rewrite(
                b"gs://bucket-name/path/to.bam?generation=3\0",
                b"r\0",
            )
            .unwrap();
            assert_eq!(
                &rewrite.url[..rewrite.url.len() - 1],
                b"https://bucket-name.storage-download.googleapis.com/path/to.bam?generation=3"
                    .as_slice()
            );
            let auth_hdr = rewrite.auth_hdr.as_ref().unwrap();
            assert_eq!(&auth_hdr[..auth_hdr.len() - 1], b"Authorization: Bearer tok123");
            let rp_hdr = rewrite.requester_pays_hdr.as_ref().unwrap();
            assert_eq!(&rp_hdr[..rp_hdr.len() - 1], b"X-Goog-User-Project: proj-7");

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

            let rewrite =
                hfile_gcs_c_41_build_rewrite(b"gs+http://bucket/object\0", b"w\0").unwrap();
            assert_eq!(
                &rewrite.url[..rewrite.url.len() - 1],
                b"http://bucket.storage-upload.googleapis.com/object".as_slice()
            );
            assert!(rewrite.auth_hdr.is_none());
            let rp_hdr = rewrite.requester_pays_hdr.as_ref().unwrap();
            assert_eq!(&rp_hdr[..rp_hdr.len() - 1], b"X-Goog-User-Project: billing-proj");

            clear_env();
        }
    }
}
