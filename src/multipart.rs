/*  multipart.c -- GA4GH redirection and multipart backend for file streams.

    Copyright (C) 2016-2017 Genome Research Ltd.

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
    hfile,
    hts::{hFILE, hts_json_token, kstring_t, size_t},
};
use std::ffi::{c_char, c_int, c_uint, c_void};

type HFileReadFn = unsafe extern "C" fn(*mut hFILE, *mut c_void, size_t) -> libc::ssize_t;
type HFileWriteFn = unsafe extern "C" fn(*mut hFILE, *const c_void, size_t) -> libc::ssize_t;
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
struct hFILE_layout {
    buffer: *mut c_char,
    begin: *mut c_char,
    end: *mut c_char,
    limit: *mut c_char,
    backend: *const hFILE_backend,
    offset: libc::off_t,
    flags: c_uint,
    has_errno: c_int,
}

const HFILE_MOBILE: c_uint = 1 << 1;

// Synthesize a System V AMD64 __va_list_tag from pointer-sized words so the
// recursive open can be routed through native hfile_c_1317_hopen_vargs instead
// of the C variadic hopen. Mirrors the pattern used by hfile_s3 and hts.rs.
unsafe fn multipart_hopen_vargs(
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
    hfile::hfile_c_1317_hopen_vargs(url, mode, &mut args)
}

// original: hfile_part (htslib/multipart.c:41)
#[repr(C)]
pub struct hfile_part {
    pub url: *mut c_char,
    pub headers: *mut *mut c_char,
}

#[repr(C)]
pub struct hFILE_multipart {
    base: hFILE_layout,
    parts: *mut hfile_part,
    nparts: usize,
    maxparts: usize,
    current: usize,
    currentfp: *mut hFILE,
}

// original: free_part (htslib/multipart.c:53)
pub unsafe fn multipart_c_53_free_part(p: *mut hfile_part) {
    libc::free((*p).url.cast());
    if !(*p).headers.is_null() {
        let mut hdr = (*p).headers;
        while !(*hdr).is_null() {
            libc::free((*hdr).cast());
            hdr = hdr.add(1);
        }
        libc::free((*p).headers.cast());
    }

    (*p).url = std::ptr::null_mut();
    (*p).headers = std::ptr::null_mut();
}

// original: free_all_parts (htslib/multipart.c:66)
pub unsafe fn multipart_c_66_free_all_parts(fp: *mut hFILE_multipart) {
    let mut i = 0usize;
    while i < (*fp).nparts {
        multipart_c_53_free_part((*fp).parts.add(i));
        i += 1;
    }
    libc::free((*fp).parts.cast());
}

// original: multipart_read (htslib/multipart.c:73)
pub unsafe extern "C" fn multipart_c_73_multipart_read(
    fpv: *mut hFILE,
    buffer: *mut c_void,
    nbytes: size_t,
) -> libc::ssize_t {
    let fp = fpv.cast::<hFILE_multipart>();

    loop {
        if (*fp).currentfp.is_null() {
            if (*fp).current < (*fp).nparts {
                let p = (*fp).parts.add((*fp).current);
                // TODO(P5): no native hts_log yet (variadic, libhts-only)
                hts_sys::hts_log(
                    hts_sys::htsLogLevel_HTS_LOG_DEBUG,
                    c"multipart".as_ptr(),
                    c"Opening part #%zu of %zu: \"%.120s%s\"".as_ptr(),
                    (*fp).current + 1,
                    (*fp).nparts,
                    (*p).url,
                    if libc::strlen((*p).url) > 120 {
                        c"...".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                );

                (*fp).currentfp = if !(*p).headers.is_null() {
                    let words: [usize; 5] = [
                        c"httphdr:v".as_ptr() as usize,
                        (*p).headers as usize,
                        c"auth_token_enabled".as_ptr() as usize,
                        c"false".as_ptr() as usize,
                        std::ptr::null::<c_void>() as usize,
                    ];
                    multipart_hopen_vargs((*p).url, c"r:".as_ptr(), &words)
                } else {
                    let words: [usize; 3] = [
                        c"auth_token_enabled".as_ptr() as usize,
                        c"false".as_ptr() as usize,
                        std::ptr::null::<c_void>() as usize,
                    ];
                    multipart_hopen_vargs((*p).url, c"r:".as_ptr(), &words)
                };

                if (*fp).currentfp.is_null() {
                    return -1;
                }
            } else {
                return 0;
            }
        }

        let current_layout = (*fp).currentfp.cast::<hFILE_layout>();
        let n = if ((*current_layout).flags & HFILE_MOBILE) != 0 {
            let read = (*(*current_layout).backend)
                .read
                .expect("hFILE read backend");
            read((*fp).currentfp, buffer, nbytes)
        } else {
            hfile::htslib_hfile_h_247_hread((*fp).currentfp, buffer, nbytes)
        };

        if n == 0 {
            let prevfp = (*fp).currentfp;
            multipart_c_53_free_part((*fp).parts.add((*fp).current));
            (*fp).current += 1;
            (*fp).currentfp = std::ptr::null_mut();
            if hfile::hclose(prevfp) < 0 {
                return -1;
            }
            continue;
        }

        return n;
    }
}

// original: multipart_write (htslib/multipart.c:114)
pub unsafe extern "C" fn multipart_c_114_multipart_write(
    _fpv: *mut hFILE,
    _buffer: *const c_void,
    _nbytes: size_t,
) -> libc::ssize_t {
    *crate::htslib_rs::c_compat::__errno_location() = libc::EROFS;
    -1
}

// original: multipart_seek (htslib/multipart.c:120)
pub unsafe extern "C" fn multipart_c_120_multipart_seek(
    _fpv: *mut hFILE,
    _offset: libc::off_t,
    _whence: c_int,
) -> libc::off_t {
    *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
    -1
}

// original: multipart_close (htslib/multipart.c:126)
pub unsafe extern "C" fn multipart_c_126_multipart_close(fpv: *mut hFILE) -> c_int {
    let fp = fpv.cast::<hFILE_multipart>();

    multipart_c_66_free_all_parts(fp);
    if !(*fp).currentfp.is_null() && hfile::hclose((*fp).currentfp) < 0 {
        return -1;
    }

    0
}

// original: multipart_backend (htslib/multipart.c:138)
static MULTIPART_BACKEND: hFILE_backend = hFILE_backend {
    read: Some(multipart_c_73_multipart_read),
    write: Some(multipart_c_114_multipart_write),
    seek: Some(multipart_c_120_multipart_seek),
    flush: None,
    close: Some(multipart_c_126_multipart_close),
};

// original: parse_ga4gh_body_json (htslib/multipart.c:149)
pub unsafe fn multipart_c_149_parse_ga4gh_body_json(
    fp: *mut hFILE_multipart,
    json: *mut hFILE,
    b: *mut kstring_t,
    header: *mut kstring_t,
) -> c_char {
    let mut t = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };

    if crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b'{' as c_char {
        return t.type_;
    }
    while crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b'}' as c_char {
        if t.type_ != b's' as c_char {
            return b'?' as c_char;
        }

        if libc::strcmp(t.str_, c"urls".as_ptr()) == 0 {
            if crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b'[' as c_char {
                return t.type_;
            }

            while crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b']' as c_char {
                let mut n = 0usize;
                let mut max = 0usize;

                if (*fp).nparts + 1 > (*fp).maxparts {
                    let new_max = ((*fp).nparts + 1)
                        .max((*fp).maxparts.saturating_mul(2))
                        .max(4);
                    let new_parts = libc::realloc(
                        (*fp).parts.cast(),
                        new_max * std::mem::size_of::<hfile_part>(),
                    )
                    .cast::<hfile_part>();
                    if new_parts.is_null() {
                        return b'?' as c_char;
                    }
                    (*fp).parts = new_parts;
                    (*fp).maxparts = new_max;
                }
                let part = (*fp).parts.add((*fp).nparts);
                (*fp).nparts += 1;
                (*part).url = std::ptr::null_mut();
                (*part).headers = std::ptr::null_mut();

                if t.type_ != b'{' as c_char {
                    return t.type_;
                }
                while crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b'}' as c_char {
                    if t.type_ != b's' as c_char {
                        return b'?' as c_char;
                    }

                    if libc::strcmp(t.str_, c"url".as_ptr()) == 0 {
                        if crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b's' as c_char
                        {
                            return t.type_;
                        }
                        (*part).url = crate::htslib_rs::hts::ks_release(b);
                    } else if libc::strcmp(t.str_, c"headers".as_ptr()) == 0 {
                        if crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b'{' as c_char
                        {
                            return t.type_;
                        }

                        while crate::htslib_rs::hts::hts_json_fnext(json, &mut t, header)
                            != b'}' as c_char
                        {
                            if t.type_ != b's' as c_char {
                                return b'?' as c_char;
                            }

                            if crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b)
                                != b's' as c_char
                            {
                                return t.type_;
                            }

                            crate::htslib_rs::hts::kputs(c": ".as_ptr(), header);
                            crate::htslib_rs::hts::kputs(t.str_, header);
                            n += 1;
                            if n + 1 > max {
                                let new_max = (n + 1).max(max.saturating_mul(2)).max(4);
                                let new_headers = libc::realloc(
                                    (*part).headers.cast(),
                                    new_max * std::mem::size_of::<*mut c_char>(),
                                )
                                .cast::<*mut c_char>();
                                if new_headers.is_null() {
                                    return b'?' as c_char;
                                }
                                (*part).headers = new_headers;
                                max = new_max;
                            }
                            *(*part).headers.add(n - 1) = crate::htslib_rs::hts::ks_release(header);
                            *(*part).headers.add(n) = std::ptr::null_mut();
                        }
                    } else if crate::htslib_rs::hts::hts_json_fskip_value(json, 0) != b'v' as c_char
                    {
                        return b'?' as c_char;
                    }
                }

                if (*part).url.is_null() {
                    return b'i' as c_char;
                }
            }
        } else if libc::strcmp(t.str_, c"format".as_ptr()) == 0 {
            if crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b's' as c_char {
                return t.type_;
            }

            // TODO(P5): no native hts_log yet (variadic, libhts-only)
            hts_sys::hts_log(
                hts_sys::htsLogLevel_HTS_LOG_DEBUG,
                c"multipart".as_ptr(),
                c"GA4GH JSON redirection to multipart %s data".as_ptr(),
                t.str_,
            );
        } else if crate::htslib_rs::hts::hts_json_fskip_value(json, 0) != b'v' as c_char {
            return b'?' as c_char;
        }
    }

    b'v' as c_char
}

// original: parse_ga4gh_redirect_json (htslib/multipart.c:220)
pub unsafe fn multipart_c_220_parse_ga4gh_redirect_json(
    fp: *mut hFILE_multipart,
    json: *mut hFILE,
    b: *mut kstring_t,
    header: *mut kstring_t,
) -> c_char {
    let mut t = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };

    if crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b'{' as c_char {
        return t.type_;
    }
    while crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != b'}' as c_char {
        if t.type_ != b's' as c_char {
            return b'?' as c_char;
        }

        if libc::strcmp(t.str_, c"htsget".as_ptr()) == 0 {
            let ret = multipart_c_149_parse_ga4gh_body_json(fp, json, b, header);
            if ret != b'v' as c_char {
                return ret;
            }
        } else {
            return b'?' as c_char;
        }
    }

    if crate::htslib_rs::hts::hts_json_fnext(json, &mut t, b) != 0 {
        return b'?' as c_char;
    }

    b'v' as c_char
}

// original: hopen_htsget_redirect (htslib/multipart.c:241)
pub unsafe fn multipart_c_241_hopen_htsget_redirect(
    hfile: *mut hFILE,
    mode: *const c_char,
) -> *mut hFILE {
    let mut s1: kstring_t = std::mem::zeroed();
    let mut s2: kstring_t = std::mem::zeroed();

    let fp = hfile::hfile_init(std::mem::size_of::<hFILE_multipart>(), mode, 0)
        .cast::<hFILE_multipart>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    (*fp).parts = std::ptr::null_mut();
    (*fp).nparts = 0;
    (*fp).maxparts = 0;

    let ret = multipart_c_220_parse_ga4gh_redirect_json(fp, hfile, &mut s1, &mut s2);
    libc::free(s1.s.cast());
    libc::free(s2.s.cast());
    if ret != b'v' as c_char {
        multipart_c_66_free_all_parts(fp);
        hfile::hfile_destroy(fp.cast());
        *crate::htslib_rs::c_compat::__errno_location() = if ret == b'?' as c_char || ret == 0 {
            libc::EPROTO
        } else {
            libc::EINVAL
        };
        return std::ptr::null_mut();
    }

    (*fp).current = 0;
    (*fp).currentfp = std::ptr::null_mut();
    (*fp).base.backend = &MULTIPART_BACKEND;
    &mut (*fp).base as *mut hFILE_layout as *mut hFILE
}
