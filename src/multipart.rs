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
    hts::{hFILE, hts_json_token, ks_free, kstring_t},
    textutils::{hts_json_fnext, hts_json_fskip_value},
};
use std::ptr::NonNull;

// hFILE flag bits (mirrors hfile.rs). Multipart is a mobile (streaming),
// read-only backend.
const HFILE_MOBILE: u32 = 1 << 1;
const HFILE_READONLY: u32 = 1 << 2;

// Synthesize a System V AMD64 __va_list_tag from pointer-sized words so the
// recursive open can be routed through native hfile_c_1317_hopen_vargs instead
// of the C variadic hopen. Mirrors the pattern used by hfile_s3 and hts.rs.
// `url`/`mode` are NUL-terminated byte slices because the variadic hopen
// boundary still expects C strings.
unsafe fn multipart_hopen_vargs(url: &[u8], mode: &[u8], words: &[usize]) -> *mut hFILE {
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
    hfile::hfile_c_1317_hopen_vargs(url.as_ptr().cast(), mode.as_ptr().cast(), &mut args)
}

// original: hfile_part (htslib/multipart.c:41)
//
// URL and header strings are owned as NUL-terminated `Vec<u8>` because they are
// ultimately handed to the variadic hopen boundary as C strings. The trailing
// NUL is kept in the buffer; `as_bytes()` returns the payload without it.
pub struct KStringCString {
    value: Vec<u8>,
}

impl KStringCString {
    fn from_raw(mut bytes: Vec<u8>) -> Option<Self> {
        // The owned kstring_t carries no trailing NUL. An interior NUL would
        // truncate the C view at the variadic boundary; reject such malformed
        // input to match the old "stop at NUL" behaviour.
        if bytes.contains(&0) {
            return None;
        }
        bytes.push(0);
        Some(Self { value: bytes })
    }

    fn as_ptr(&self) -> *const u8 {
        self.value.as_ptr()
    }

    fn as_bytes(&self) -> &[u8] {
        &self.value[..self.value.len() - 1]
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
pub unsafe fn multipart_read(fp: &mut hFILE_multipart, buffer: &mut [u8]) -> isize {
    loop {
        if fp.currentfp.is_none() {
            if fp.current < fp.parts.len() {
                let current = fp.current;
                let nparts = fp.parts.len();
                let part = &fp.parts[current];
                // NUL-terminated URL buffer (the trailing NUL is the last byte);
                // the variadic hopen boundary needs the C-string pointer.
                let url = part.url.as_ref().expect("multipart part URL");
                let url_bytes = url.as_bytes();
                let truncate = url_bytes.len() > 120;
                let shown =
                    std::str::from_utf8(if truncate { &url_bytes[..120] } else { url_bytes })
                        .unwrap_or("");
                let msg = format!(
                    "Opening part #{} of {}: \"{}{}\"",
                    current + 1,
                    nparts,
                    shown,
                    if truncate { "..." } else { "" },
                );
                crate::htslib_rs::hts::hts_log_cstr(
                    crate::htslib_rs::hts::HTS_LOG_DEBUG,
                    b"multipart",
                    msg.as_bytes(),
                );

                let url_buf = &url.value;
                let new_currentfp = if !part.headers.is_empty() {
                    let mut header_ptrs: Vec<*const u8> =
                        part.headers.iter().map(KStringCString::as_ptr).collect();
                    header_ptrs.push(std::ptr::null());
                    let words: [usize; 5] = [
                        c"httphdr:v".as_ptr() as usize,
                        header_ptrs.as_mut_ptr() as usize,
                        c"auth_token_enabled".as_ptr() as usize,
                        c"false".as_ptr() as usize,
                        std::ptr::null::<()>() as usize,
                    ];
                    hfile::OwnedHFile::from_raw(multipart_hopen_vargs(url_buf, b"r:\0", &words))
                } else {
                    let words: [usize; 3] = [
                        c"auth_token_enabled".as_ptr() as usize,
                        c"false".as_ptr() as usize,
                        std::ptr::null::<()>() as usize,
                    ];
                    hfile::OwnedHFile::from_raw(multipart_hopen_vargs(url_buf, b"r:\0", &words))
                };
                fp.currentfp = new_currentfp;

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
                buffer.len(),
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
    _buffer: *const (),
    _nbytes: usize,
) -> isize {
    *crate::htslib_rs::c_compat::__errno_location() = libc::EROFS;
    -1
}

// original: multipart_seek (htslib/multipart.c:120)
pub unsafe fn multipart_c_120_multipart_seek(
    _fp: &mut hFILE,
    _offset: i64,
    _whence: i32,
) -> i64 {
    *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
    -1
}

// original: multipart_close (htslib/multipart.c:126)
pub unsafe fn multipart_c_126_multipart_close(fp: &mut hFILE) -> i32 {
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

unsafe fn multipart_reserve_parts(fp: &mut hFILE_multipart) -> i32 {
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
) -> i8 {
    let mut t = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };

    if hts_json_fnext(json, &mut t, b) != b'{' as i8 {
        return t.type_;
    }
    while hts_json_fnext(json, &mut t, b) != b'}' as i8 {
        if t.type_ != b's' as i8 {
            return b'?' as i8;
        }

        // `t.str_` is a NUL-terminated text buffer owned by the JSON tokeniser;
        // view it as a byte slice for the literal comparisons below.
        let key = {
            let mut len = 0usize;
            while *t.str_.add(len) != 0 {
                len += 1;
            }
            std::slice::from_raw_parts(t.str_.cast::<u8>(), len)
        };

        if key == b"urls" {
            if hts_json_fnext(json, &mut t, b) != b'[' as i8 {
                return t.type_;
            }

            while hts_json_fnext(json, &mut t, b) != b']' as i8 {
                if multipart_reserve_parts(fp) != 0 {
                    return b'?' as i8;
                }
                fp.parts.push(hfile_part {
                    url: None,
                    headers: Vec::new(),
                });
                let part_index = fp.parts.len() - 1;

                if t.type_ != b'{' as i8 {
                    return t.type_;
                }
                while hts_json_fnext(json, &mut t, b) != b'}' as i8 {
                    if t.type_ != b's' as i8 {
                        return b'?' as i8;
                    }

                    let inner_key = {
                        let mut len = 0usize;
                        while *t.str_.add(len) != 0 {
                            len += 1;
                        }
                        std::slice::from_raw_parts(t.str_.cast::<u8>(), len)
                    };

                    if inner_key == b"url" {
                        if hts_json_fnext(json, &mut t, b) != b's' as i8 {
                            return t.type_;
                        }
                        fp.part_mut(part_index).url =
                            KStringCString::from_raw(crate::htslib_rs::hts::ks_release(b));
                    } else if inner_key == b"headers" {
                        if hts_json_fnext(json, &mut t, b) != b'{' as i8 {
                            return t.type_;
                        }

                        while hts_json_fnext(json, &mut t, header) != b'}' as i8 {
                            if t.type_ != b's' as i8 {
                                return b'?' as i8;
                            }

                            if hts_json_fnext(json, &mut t, b) != b's' as i8 {
                                return t.type_;
                            }

                            let value = {
                                let mut len = 0usize;
                                while *t.str_.add(len) != 0 {
                                    len += 1;
                                }
                                std::slice::from_raw_parts(t.str_.cast::<u8>(), len)
                            };
                            crate::htslib_rs::hts::kputs(b": ", header);
                            crate::htslib_rs::hts::kputs(value, header);
                            let part = fp.part_mut(part_index);
                            if part.headers.try_reserve(1).is_err() {
                                *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
                                return b'?' as i8;
                            }
                            if let Some(header) =
                                KStringCString::from_raw(crate::htslib_rs::hts::ks_release(header))
                            {
                                part.headers.push(header);
                            }
                        }
                    } else if hts_json_fskip_value(json, 0) != b'v' as i8 {
                        return b'?' as i8;
                    }
                }

                if fp.part_mut(part_index).url.is_none() {
                    return b'i' as i8;
                }
            }
        } else if key == b"format" {
            if hts_json_fnext(json, &mut t, b) != b's' as i8 {
                return t.type_;
            }

            let format_name = {
                let mut len = 0usize;
                while *t.str_.add(len) != 0 {
                    len += 1;
                }
                String::from_utf8_lossy(std::slice::from_raw_parts(t.str_.cast::<u8>(), len))
                    .into_owned()
            };
            let msg = format!(
                "GA4GH JSON redirection to multipart {} data",
                format_name,
            );
            crate::htslib_rs::hts::hts_log_cstr(
                crate::htslib_rs::hts::HTS_LOG_DEBUG,
                b"multipart",
                msg.as_bytes(),
            );
        } else if hts_json_fskip_value(json, 0) != b'v' as i8 {
            return b'?' as i8;
        }
    }

    b'v' as i8
}

// original: parse_ga4gh_redirect_json (htslib/multipart.c:220)
pub unsafe fn multipart_c_220_parse_ga4gh_redirect_json(
    fp: &mut hFILE_multipart,
    json: &mut hFILE,
    b: &mut kstring_t,
    header: &mut kstring_t,
) -> i8 {
    let mut t = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };

    if hts_json_fnext(json, &mut t, b) != b'{' as i8 {
        return t.type_;
    }
    while hts_json_fnext(json, &mut t, b) != b'}' as i8 {
        if t.type_ != b's' as i8 {
            return b'?' as i8;
        }

        let key = {
            let mut len = 0usize;
            while *t.str_.add(len) != 0 {
                len += 1;
            }
            std::slice::from_raw_parts(t.str_.cast::<u8>(), len)
        };

        if key == b"htsget" {
            let ret = multipart_c_149_parse_ga4gh_body_json(fp, json, b, header);
            if ret != b'v' as i8 {
                return ret;
            }
        } else {
            return b'?' as i8;
        }
    }

    if hts_json_fnext(json, &mut t, b) != 0 {
        return b'?' as i8;
    }

    b'v' as i8
}

// original: hopen_htsget_redirect (htslib/multipart.c:241)
pub unsafe fn multipart_hopen_htsget_redirect(
    mut hfile: NonNull<hFILE>,
    mode: &[u8],
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
    if ret != b'v' as i8 {
        multipart_c_66_free_all_parts(&mut m);
        *crate::htslib_rs::c_compat::__errno_location() = if ret == b'?' as i8 || ret == 0 {
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
    let readonly = hfile_mode_is_readonly(mode);
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
