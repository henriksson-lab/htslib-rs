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

use crate::htslib_rs::hfile::{
    hfile_set_blksize_impl as hfile_set_blksize, hgetc2_impl as hgetc2,
    hgetdelim_impl as hgetdelim, hputc2_impl as hputc2, hputs2_impl as hputs2,
    hread2_impl as hread2, hwrite2_impl as hwrite2,
};
use crate::htslib_rs::hts::hFILE as hFILE_layout;

const HFILE_MOBILE: u32 = 1 << 1;

// original: herrno (htslib/htslib/hfile.h:134)
pub fn htslib_hfile_h_134_herrno(fp: &hFILE_layout) -> i32 {
    fp.has_errno
}

// original: hclearerr (htslib/htslib/hfile.h:140)
pub fn htslib_hfile_h_140_hclearerr(fp: &mut hFILE_layout) {
    fp.has_errno = 0;
}

// original: htell (htslib/htslib/hfile.h:155)
pub fn htslib_hfile_h_155_htell(fp: &hFILE_layout) -> i64 {
    fp.offset + fp.begin as i64
}

// original: hgetc (htslib/htslib/hfile.h:163)
pub fn htslib_hfile_h_163_hgetc(fp: &mut hFILE_layout) -> i32 {
    if fp.end > fp.begin {
        let c = fp.buffer[fp.begin];
        fp.begin += 1;
        c as i32
    } else {
        unsafe { hgetc2(fp) }
    }
}

// original: hgetln (htslib/htslib/hfile.h:195)
pub fn htslib_hfile_h_195_hgetln(buffer: &mut [u8], fp: &mut hFILE_layout) -> isize {
    unsafe { hgetdelim(buffer, b'\n' as i32, fp) }
}

// original: hread (htslib/htslib/hfile.h:247)
pub fn htslib_hfile_h_247_hread(fp: &mut hFILE_layout, buffer: &mut [u8]) -> isize {
    let nbytes = buffer.len();
    let mut n = fp.end - fp.begin;
    if n > nbytes {
        n = nbytes;
    }
    buffer[..n].copy_from_slice(&fp.buffer[fp.begin..fp.begin + n]);
    fp.begin += n;
    if n == nbytes || (fp.flags & HFILE_MOBILE) == 0 {
        n as isize
    } else {
        unsafe { hread2(fp, n, buffer) }
    }
}

// original: hputc (htslib/htslib/hfile.h:263)
pub fn htslib_hfile_h_263_hputc(c: i32, fp: &mut hFILE_layout) -> i32 {
    if fp.begin < fp.limit {
        fp.buffer[fp.begin] = c as u8;
        fp.begin += 1;
        c
    } else {
        unsafe { hputc2(c, fp) }
    }
}

// original: hputs (htslib/htslib/hfile.h:275)
pub fn htslib_hfile_h_275_hputs(text: &[u8], fp: &mut hFILE_layout) -> i32 {
    let nbytes = text.len();
    let mut n = fp.limit - fp.begin;
    if n > nbytes {
        n = nbytes;
    }
    fp.buffer[fp.begin..fp.begin + n].copy_from_slice(&text[..n]);
    fp.begin += n;
    if n == nbytes {
        0
    } else {
        unsafe { hputs2(text, n, fp) }
    }
}

// original: hwrite (htslib/htslib/hfile.h:292)
pub fn htslib_hfile_h_292_hwrite(fp: &mut hFILE_layout, buffer: &[u8]) -> isize {
    let nbytes = buffer.len();
    if (fp.flags & HFILE_MOBILE) == 0 {
        let n = fp.limit - fp.begin;
        if n < nbytes {
            let bufsiz = fp.limit + nbytes;
            unsafe { hfile_set_blksize(fp, bufsiz) };
            fp.end = fp.limit;
        }
    }

    let mut n = fp.limit - fp.begin;
    if nbytes >= n && fp.begin == 0 {
        return unsafe { hwrite2(fp, 0, buffer) };
    }

    if n > nbytes {
        n = nbytes;
    }
    fp.buffer[fp.begin..fp.begin + n].copy_from_slice(&buffer[..n]);
    fp.begin += n;
    if n == nbytes {
        n as isize
    } else {
        unsafe { hwrite2(fp, n, buffer) }
    }
}
