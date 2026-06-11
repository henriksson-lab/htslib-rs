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
    hfile::{self, HFileBackend},
    hts::{hFILE, hts_json_token, ks_free, kstring_t, size_t},
    textutils::{hts_json_fnext, hts_json_fskip_value},
};
use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};
use std::ptr::NonNull;

// hFILE flag bits (mirrors hfile.rs). Multipart is a mobile (streaming),
// read-only backend.
const HFILE_MOBILE: c_uint = 1 << 1;
const HFILE_READONLY: c_uint = 1 << 2;

// Synthesize a System V AMD64 __va_list_tag from pointer-sized words so the
// recursive open can be routed through native hfile_c_1317_hopen_vargs instead
// of the C variadic hopen. Mirrors the pattern used by hfile_s3 and hts.rs.
unsafe fn multipart_hopen_vargs(url: &CStr, mode: &CStr, words: &[usize]) -> *mut hFILE {
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
    hfile::hfile_c_1317_hopen_vargs(url.as_ptr(), mode.as_ptr(), &mut args)
}

// original: hfile_part (htslib/multipart.c:41)
pub struct KStringCString {
    value: CString,
}

impl KStringCString {
    fn from_raw(bytes: Vec<u8>) -> Option<Self> {
        // The owned kstring_t carries no trailing NUL; CString::new appends one.
        // Interior NULs make this fail, matching the old "stop at NUL" behaviour
        // by yielding None for malformed input.
        let value = CString::new(bytes).ok()?;
        Some(Self { value })
    }

    fn as_ptr(&self) -> *const c_char {
        self.value.as_ptr()
    }

    fn as_c_str(&self) -> &CStr {
        self.value.as_c_str()
    }
}

pub struct hfile_part {
    pub url: Option<KStringCString>,
    pub headers: Vec<KStringCString>,
}

// original: hFILE_multipart (htslib/multipart.c:46)
//
// SEAM: the subclass no longer embeds `base: hFILE_layout`. It is now the
// payload of `HFileBackend::Multipart(Box<hFILE_multipart>)`, so it carries
// only the backend-specific state. The owning `hFILE` (buffer + flags + the
// enum) lives one level up and is reached as `&mut hFILE` in the dispatch
// bodies below.
pub struct hFILE_multipart {
    parts: Vec<hfile_part>,
    current: usize,
    currentfp: Option<hfile::OwnedHFile>,
}

impl hFILE_multipart {
    fn part_mut(&mut self, index: usize) -> &mut hfile_part {
        &mut self.parts[index]
    }
}

// original: free_part (htslib/multipart.c:53)
pub fn multipart_c_53_free_part(p: &mut hfile_part) {
    p.url = None;
    p.headers.clear();
}

// original: free_all_parts (htslib/multipart.c:66)
pub fn multipart_c_66_free_all_parts(fp: &mut hFILE_multipart) {
    for part in &mut fp.parts {
        multipart_c_53_free_part(part);
    }
    fp.parts.clear();
}

// original: multipart_read (htslib/multipart.c:73)
pub unsafe fn multipart_read(fp: &mut hFILE_multipart, buffer: &mut [u8]) -> libc::ssize_t {
    loop {
        if fp.currentfp.is_none() {
            if fp.current < fp.parts.len() {
                let current = fp.current;
                let nparts = fp.parts.len();
                let (url, headers) = {
                    let p = fp.part_mut(current);
                    (
                        p.url.as_ref().expect("multipart part URL").as_c_str(),
                        &p.headers,
                    )
                };
                let url_cstr = url.to_bytes();
                let truncate = url_cstr.len() > 120;
                let shown = std::str::from_utf8(if truncate { &url_cstr[..120] } else { url_cstr })
                    .unwrap_or("");
                let msg = std::ffi::CString::new(format!(
                    "Opening part #{} of {}: \"{}{}\"",
                    current + 1,
                    nparts,
                    shown,
                    if truncate { "..." } else { "" },
                ))
                .unwrap_or_default();
                crate::htslib_rs::hts::hts_log_cstr(
                    crate::htslib_rs::hts::HTS_LOG_DEBUG,
                    c"multipart".as_ptr(),
                    msg.as_ptr(),
                );

                fp.currentfp = if !headers.is_empty() {
                    let mut header_ptrs: Vec<*const c_char> =
                        headers.iter().map(KStringCString::as_ptr).collect();
                    header_ptrs.push(std::ptr::null());
                    let words: [usize; 5] = [
                        c"httphdr:v".as_ptr() as usize,
                        header_ptrs.as_mut_ptr() as usize,
                        c"auth_token_enabled".as_ptr() as usize,
                        c"false".as_ptr() as usize,
                        std::ptr::null::<c_void>() as usize,
                    ];
                    hfile::OwnedHFile::from_raw(multipart_hopen_vargs(url, c"r:", &words))
                } else {
                    let words: [usize; 3] = [
                        c"auth_token_enabled".as_ptr() as usize,
                        c"false".as_ptr() as usize,
                        std::ptr::null::<c_void>() as usize,
                    ];
                    hfile::OwnedHFile::from_raw(multipart_hopen_vargs(url, c"r:", &words))
                };

                if fp.currentfp.is_none() {
                    return -1;
                }
            } else {
                return 0;
            }
        }

        // SEAM: the sub-stream is now an owned `hFILE`. Reach it as `&mut hFILE`
        // and, when it is a mobile (streaming) backend, dispatch its read
        // directly through the `HFileBackend` enum (was the old vtable
        // `(*backend).read`); otherwise fall back to the buffered hread.
        let currentfp = fp.currentfp.as_ref().expect("multipart current hFILE");
        let sub = &mut *currentfp.as_ptr();
        let n = if (sub.flags & HFILE_MOBILE) != 0 {
            HFileBackend::read(sub, buffer)
        } else {
            hfile::htslib_hfile_h_247_hread(
                sub as *mut hFILE,
                buffer.as_mut_ptr().cast(),
                buffer.len() as size_t,
            )
        };

        if n == 0 {
            let prevfp = fp.currentfp.take().expect("multipart current hFILE");
            multipart_c_53_free_part(fp.part_mut(fp.current));
            fp.current += 1;
            if prevfp.close() < 0 {
                return -1;
            }
            continue;
        }

        return n;
    }
}

// original: multipart_write (htslib/multipart.c:114)
pub unsafe fn multipart_c_114_multipart_write(
    _fp: &mut hFILE,
    _buffer: *const c_void,
    _nbytes: size_t,
) -> libc::ssize_t {
    *crate::htslib_rs::c_compat::__errno_location() = libc::EROFS;
    -1
}

// original: multipart_seek (htslib/multipart.c:120)
pub unsafe fn multipart_c_120_multipart_seek(
    _fp: &mut hFILE,
    _offset: libc::off_t,
    _whence: c_int,
) -> libc::off_t {
    *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
    -1
}

// original: multipart_close (htslib/multipart.c:126)
pub unsafe fn multipart_c_126_multipart_close(fp: &mut hFILE) -> c_int {
    let HFileBackend::Multipart(m) = &mut fp.backend else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return -1;
    };
    let m: &mut hFILE_multipart = &mut *m;

    multipart_c_66_free_all_parts(m);
    if let Some(currentfp) = m.currentfp.take() {
        if currentfp.close() < 0 {
            return -1;
        }
    }

    0
}

unsafe fn multipart_reserve_parts(fp: &mut hFILE_multipart) -> c_int {
    if fp.parts.try_reserve(1).is_err() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
        return -1;
    }
    0
}

// original: parse_ga4gh_body_json (htslib/multipart.c:149)
pub unsafe fn multipart_c_149_parse_ga4gh_body_json(
    fp: &mut hFILE_multipart,
    json: &mut hFILE,
    b: &mut kstring_t,
    header: &mut kstring_t,
) -> c_char {
    let mut t = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };

    if hts_json_fnext(json, &mut t, b) != b'{' as c_char {
        return t.type_;
    }
    while hts_json_fnext(json, &mut t, b) != b'}' as c_char {
        if t.type_ != b's' as c_char {
            return b'?' as c_char;
        }

        if CStr::from_ptr(t.str_) == c"urls" {
            if hts_json_fnext(json, &mut t, b) != b'[' as c_char {
                return t.type_;
            }

            while hts_json_fnext(json, &mut t, b) != b']' as c_char {
                if multipart_reserve_parts(fp) != 0 {
                    return b'?' as c_char;
                }
                fp.parts.push(hfile_part {
                    url: None,
                    headers: Vec::new(),
                });
                let part_index = fp.parts.len() - 1;

                if t.type_ != b'{' as c_char {
                    return t.type_;
                }
                while hts_json_fnext(json, &mut t, b) != b'}' as c_char {
                    if t.type_ != b's' as c_char {
                        return b'?' as c_char;
                    }

                    if CStr::from_ptr(t.str_) == c"url" {
                        if hts_json_fnext(json, &mut t, b) != b's' as c_char {
                            return t.type_;
                        }
                        fp.part_mut(part_index).url =
                            KStringCString::from_raw(crate::htslib_rs::hts::ks_release(b));
                    } else if CStr::from_ptr(t.str_) == c"headers" {
                        if hts_json_fnext(json, &mut t, b) != b'{' as c_char {
                            return t.type_;
                        }

                        while hts_json_fnext(json, &mut t, header) != b'}' as c_char {
                            if t.type_ != b's' as c_char {
                                return b'?' as c_char;
                            }

                            if hts_json_fnext(json, &mut t, b) != b's' as c_char {
                                return t.type_;
                            }

                            crate::htslib_rs::hts::kputs(b": ", header);
                            crate::htslib_rs::hts::kputs(CStr::from_ptr(t.str_).to_bytes(), header);
                            let part = fp.part_mut(part_index);
                            if part.headers.try_reserve(1).is_err() {
                                *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
                                return b'?' as c_char;
                            }
                            if let Some(header) =
                                KStringCString::from_raw(crate::htslib_rs::hts::ks_release(header))
                            {
                                part.headers.push(header);
                            }
                        }
                    } else if hts_json_fskip_value(json, 0) != b'v' as c_char {
                        return b'?' as c_char;
                    }
                }

                if fp.part_mut(part_index).url.is_none() {
                    return b'i' as c_char;
                }
            }
        } else if CStr::from_ptr(t.str_) == c"format" {
            if hts_json_fnext(json, &mut t, b) != b's' as c_char {
                return t.type_;
            }

            let format_name = std::ffi::CStr::from_ptr(t.str_).to_string_lossy();
            let msg = std::ffi::CString::new(format!(
                "GA4GH JSON redirection to multipart {} data",
                format_name,
            ))
            .unwrap_or_default();
            crate::htslib_rs::hts::hts_log_cstr(
                crate::htslib_rs::hts::HTS_LOG_DEBUG,
                c"multipart".as_ptr(),
                msg.as_ptr(),
            );
        } else if hts_json_fskip_value(json, 0) != b'v' as c_char {
            return b'?' as c_char;
        }
    }

    b'v' as c_char
}

// original: parse_ga4gh_redirect_json (htslib/multipart.c:220)
pub unsafe fn multipart_c_220_parse_ga4gh_redirect_json(
    fp: &mut hFILE_multipart,
    json: &mut hFILE,
    b: &mut kstring_t,
    header: &mut kstring_t,
) -> c_char {
    let mut t = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };

    if hts_json_fnext(json, &mut t, b) != b'{' as c_char {
        return t.type_;
    }
    while hts_json_fnext(json, &mut t, b) != b'}' as c_char {
        if t.type_ != b's' as c_char {
            return b'?' as c_char;
        }

        if CStr::from_ptr(t.str_) == c"htsget" {
            let ret = multipart_c_149_parse_ga4gh_body_json(fp, json, b, header);
            if ret != b'v' as c_char {
                return ret;
            }
        } else {
            return b'?' as c_char;
        }
    }

    if hts_json_fnext(json, &mut t, b) != 0 {
        return b'?' as c_char;
    }

    b'v' as c_char
}

// original: hopen_htsget_redirect (htslib/multipart.c:241)
pub unsafe fn multipart_hopen_htsget_redirect(
    mut hfile: NonNull<hFILE>,
    mode: &CStr,
) -> *mut hFILE {
    let mut s1: kstring_t = kstring_t::default();
    let mut s2: kstring_t = kstring_t::default();

    // SEAM: build the backend payload directly (no more hfile_init of a
    // subclass that embedded `base`). Parsing populates `parts`.
    let mut m = hFILE_multipart {
        parts: Vec::new(),
        current: 0,
        currentfp: None,
    };

    let ret = multipart_c_220_parse_ga4gh_redirect_json(&mut m, hfile.as_mut(), &mut s1, &mut s2);
    ks_free(&mut s1);
    ks_free(&mut s2);
    if ret != b'v' as c_char {
        multipart_c_66_free_all_parts(&mut m);
        *crate::htslib_rs::c_compat::__errno_location() = if ret == b'?' as c_char || ret == 0 {
            libc::EPROTO
        } else {
            libc::EINVAL
        };
        return std::ptr::null_mut();
    }

    m.current = 0;
    m.currentfp = None;

    // SEAM: construct and OWN the root hFILE. The buffer is a Vec<u8> sized to
    // the default read capacity; begin/end/limit are byte indices. The backend
    // is the enum variant carrying the multipart payload inline. Ownership is
    // handed to the caller as a raw pointer (Box::into_raw); hclose reclaims it.
    let readonly = hfile_mode_is_readonly(mode.to_bytes());
    let mut flags = HFILE_MOBILE;
    if readonly {
        flags |= HFILE_READONLY;
    }
    let capacity = 128 * 1024usize;
    let owner = Box::new(hFILE {
        buffer: vec![0u8; capacity],
        begin: 0,
        end: 0,
        limit: capacity,
        backend: HFileBackend::Multipart(Box::new(m)),
        offset: 0,
        flags,
        has_errno: 0,
    });
    Box::into_raw(owner)
}

fn hfile_mode_is_readonly(mode: &[u8]) -> bool {
    mode.contains(&b'r') && !mode.contains(&b'+')
}
