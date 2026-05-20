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
use std::ffi::{c_char, c_int, c_void};

type HFileOpenFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut hFILE;
type HFileIsRemoteFn = unsafe extern "C" fn(*const c_char) -> c_int;
type HFileVOpenFn =
    unsafe extern "C" fn(*const c_char, *const c_char, *mut hts_sys::__va_list_tag) -> *mut hFILE;

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

unsafe extern "C" {
    #[link_name = "hopen"]
    fn htslib_hopen(fname: *const c_char, mode: *const c_char, ...) -> *mut hFILE;
}

// original: gcs_rewrite (htslib/hfile_gcs.c:41)
unsafe fn hfile_gcs_c_41_gcs_rewrite(
    gsurl: *const c_char,
    mut mode: *const c_char,
    mode_has_colon: c_int,
    argsp: *mut hts_sys::__va_list_tag,
) -> *mut hFILE {
    let mut mode_colon: kstring_t = std::mem::zeroed();
    let mut url: kstring_t = std::mem::zeroed();
    let mut auth_hdr: kstring_t = std::mem::zeroed();
    let mut requester_pays_hdr: kstring_t = std::mem::zeroed();
    let mut fp: *mut hFILE = std::ptr::null_mut();

    // GCS URL format is gs[+SCHEME]://BUCKET/PATH

    let mut bucket = if *gsurl.add(2) == b'+' as c_char {
        let bucket = libc::strchr(gsurl, b':' as c_int).add(1);
        kputsn(
            gsurl.add(3),
            bucket.offset_from(gsurl.add(3)) as usize,
            &mut url,
        );
        bucket
    } else {
        kputs(c"https:".as_ptr(), &mut url);
        gsurl.add(3)
    };

    while *bucket == b'/' as c_char {
        kputc(*bucket as c_int, &mut url);
        bucket = bucket.add(1);
    }

    let path = bucket.add(libc::strcspn(bucket, c"/?#".as_ptr()));

    kputsn(bucket, path.offset_from(bucket) as usize, &mut url);
    if !libc::strchr(mode, b'r' as c_int).is_null() {
        kputs(c".storage-download".as_ptr(), &mut url);
    } else if !libc::strchr(mode, b'w' as c_int).is_null() {
        kputs(c".storage-upload".as_ptr(), &mut url);
    } else {
        kputs(c".storage".as_ptr(), &mut url);
    }
    kputs(c".googleapis.com".as_ptr(), &mut url);

    kputs(path, &mut url);

    if hts_verbose >= 8 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[M::gcs_open] rewrote URL as %s\n".as_ptr(),
            url.s,
        );
    }

    // TODO Find the access token in a more standard way
    let access_token = libc::getenv(c"GCS_OAUTH_TOKEN".as_ptr());

    if !access_token.is_null() {
        kputs(c"Authorization: Bearer ".as_ptr(), &mut auth_hdr);
        kputs(access_token, &mut auth_hdr);
    }

    let requester_pays_project = libc::getenv(c"GCS_REQUESTER_PAYS_PROJECT".as_ptr());

    if !requester_pays_project.is_null() {
        kputs(c"X-Goog-User-Project: ".as_ptr(), &mut requester_pays_hdr);
        kputs(requester_pays_project, &mut requester_pays_hdr);
    }

    if !argsp.is_null() || mode_has_colon != 0 || auth_hdr.l > 0 || requester_pays_hdr.l > 0 {
        if mode_has_colon == 0 {
            kputs(mode, &mut mode_colon);
            kputc(b':' as c_int, &mut mode_colon);
            mode = mode_colon.s;
        }

        if auth_hdr.l > 0 && requester_pays_hdr.l > 0 {
            fp = htslib_hopen(
                url.s,
                mode,
                c"va_list".as_ptr(),
                argsp,
                c"httphdr:l".as_ptr(),
                auth_hdr.s,
                requester_pays_hdr.s,
                std::ptr::null::<c_char>(),
                std::ptr::null::<c_char>(),
            );
        } else {
            fp = htslib_hopen(
                url.s,
                mode,
                c"va_list".as_ptr(),
                argsp,
                c"httphdr".as_ptr(),
                if auth_hdr.l > 0 {
                    auth_hdr.s
                } else {
                    std::ptr::null_mut()
                },
                std::ptr::null::<c_char>(),
            );
        }
    } else {
        fp = htslib_hopen(url.s, mode);
    }

    libc::free(mode_colon.s.cast());
    libc::free(url.s.cast());
    libc::free(auth_hdr.s.cast());
    libc::free(requester_pays_hdr.s.cast());
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
    args0: *mut hts_sys::__va_list_tag,
) -> *mut hFILE {
    // Need to use va_copy() as we can only take the address of an actual
    // va_list object, not that of a parameter as its type may have decayed.
    let mut args = std::mem::MaybeUninit::<hts_sys::__va_list_tag>::uninit();
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
