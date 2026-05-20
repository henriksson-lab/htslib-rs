/// @file htslib/hfile.h
/// Buffered low-level input/output streams.
/*
    Copyright (C) 2013-2022 Genome Research Ltd.

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
use std::ffi::{c_char, c_int, c_uint, c_void};

use crate::htslib_mini_rs::{
    hfile::{hfile_set_blksize, hgetc2, hgetdelim, hputc2, hputs2, hread2, hwrite2},
    hts::{hFILE, size_t},
};

// original: hFILE (htslib/htslib/hfile.h:47)
#[repr(C)]
struct hFILE_layout {
    buffer: *mut c_char,
    begin: *mut c_char,
    end: *mut c_char,
    limit: *mut c_char,
    backend: *const c_void,
    offset: libc::off_t,
    flags: c_uint,
    has_errno: c_int,
}

const HFILE_MOBILE: c_uint = 1 << 1;

// original: herrno (htslib/htslib/hfile.h:134)
pub unsafe fn htslib_hfile_h_134_herrno(fp: *mut hFILE) -> c_int {
    (*(fp.cast::<hFILE_layout>())).has_errno
}

// original: hclearerr (htslib/htslib/hfile.h:140)
pub unsafe fn htslib_hfile_h_140_hclearerr(fp: *mut hFILE) {
    (*(fp.cast::<hFILE_layout>())).has_errno = 0;
}

// original: htell (htslib/htslib/hfile.h:155)
pub unsafe fn htslib_hfile_h_155_htell(fp: *mut hFILE) -> libc::off_t {
    let fp = fp.cast::<hFILE_layout>();
    (*fp).offset + (*fp).begin.offset_from((*fp).buffer) as libc::off_t
}

// original: hgetc (htslib/htslib/hfile.h:163)
pub unsafe fn htslib_hfile_h_163_hgetc(fp: *mut hFILE) -> c_int {
    let fp = fp.cast::<hFILE_layout>();
    if (*fp).end > (*fp).begin {
        let c = *(*fp).begin as u8;
        (*fp).begin = (*fp).begin.add(1);
        c as c_int
    } else {
        hgetc2(fp.cast())
    }
}

// original: hgetln (htslib/htslib/hfile.h:195)
pub unsafe fn htslib_hfile_h_195_hgetln(
    buffer: *mut c_char,
    size: size_t,
    fp: *mut hFILE,
) -> libc::ssize_t {
    hgetdelim(buffer, size, b'\n' as c_int, fp)
}

// original: hread (htslib/htslib/hfile.h:247)
pub unsafe fn htslib_hfile_h_247_hread(
    fp: *mut hFILE,
    buffer: *mut c_void,
    nbytes: size_t,
) -> libc::ssize_t {
    let fp_layout = fp.cast::<hFILE_layout>();
    let mut n = (*fp_layout).end.offset_from((*fp_layout).begin) as size_t;
    if n > nbytes {
        n = nbytes;
    }
    libc::memcpy(buffer, (*fp_layout).begin.cast(), n);
    (*fp_layout).begin = (*fp_layout).begin.add(n);
    if n == nbytes || ((*fp_layout).flags & HFILE_MOBILE) == 0 {
        n as libc::ssize_t
    } else {
        hread2(fp, buffer, nbytes, n)
    }
}

// original: hputc (htslib/htslib/hfile.h:263)
pub unsafe fn htslib_hfile_h_263_hputc(c: c_int, fp: *mut hFILE) -> c_int {
    let fp_layout = fp.cast::<hFILE_layout>();
    if (*fp_layout).begin < (*fp_layout).limit {
        *(*fp_layout).begin = c as c_char;
        (*fp_layout).begin = (*fp_layout).begin.add(1);
        c
    } else {
        hputc2(c, fp)
    }
}

// original: hputs (htslib/htslib/hfile.h:275)
pub unsafe fn htslib_hfile_h_275_hputs(text: *const c_char, fp: *mut hFILE) -> c_int {
    let fp_layout = fp.cast::<hFILE_layout>();
    let nbytes = libc::strlen(text);
    let mut n = (*fp_layout).limit.offset_from((*fp_layout).begin) as size_t;
    if n > nbytes {
        n = nbytes;
    }
    libc::memcpy((*fp_layout).begin.cast(), text.cast(), n);
    (*fp_layout).begin = (*fp_layout).begin.add(n);
    if n == nbytes {
        0
    } else {
        hputs2(text, nbytes, n, fp)
    }
}

// original: hwrite (htslib/htslib/hfile.h:292)
pub unsafe fn htslib_hfile_h_292_hwrite(
    fp: *mut hFILE,
    buffer: *const c_void,
    nbytes: size_t,
) -> libc::ssize_t {
    let fp_layout = fp.cast::<hFILE_layout>();
    if ((*fp_layout).flags & HFILE_MOBILE) == 0 {
        let n = (*fp_layout).limit.offset_from((*fp_layout).begin) as size_t;
        if n < nbytes {
            hfile_set_blksize(
                fp,
                (*fp_layout).limit.offset_from((*fp_layout).buffer) as size_t + nbytes,
            );
            (*fp_layout).end = (*fp_layout).limit;
        }
    }

    let mut n = (*fp_layout).limit.offset_from((*fp_layout).begin) as size_t;
    if nbytes >= n && (*fp_layout).begin == (*fp_layout).buffer {
        return hwrite2(fp, buffer, nbytes, 0);
    }

    if n > nbytes {
        n = nbytes;
    }
    libc::memcpy((*fp_layout).begin.cast(), buffer, n);
    (*fp_layout).begin = (*fp_layout).begin.add(n);
    if n == nbytes {
        n as libc::ssize_t
    } else {
        hwrite2(fp, buffer, nbytes, n)
    }
}
