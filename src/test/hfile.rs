/*  test/hfile.c -- Test cases for low-level input/output streams.

    Copyright (C) 2013-2014, 2016, 2018 Genome Research Ltd.

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

use std::ffi::{c_char, c_int};

use crate::htslib_rs::{
    hfile::{
        haddextension, hclose, hfile_c_878_hopenv_mem, hfile_mem_get_buffer, hfile_mem_steal_buffer,
        hflush, hgets, hopen,
        hpeek, hseek, htslib_hfile_h_134_herrno as herrno, htslib_hfile_h_155_htell as htell,
        htslib_hfile_h_163_hgetc as hgetc, htslib_hfile_h_247_hread as hread,
        htslib_hfile_h_263_hputc as hputc, htslib_hfile_h_275_hputs as hputs,
        htslib_hfile_h_292_hwrite as hwrite,
    },
    hts::{hFILE, ks_release, kstring_t, size_t},
};

// original: fail (htslib/test/hfile.c:38)
macro_rules! test_hfile_c_38_fail {
    ($format:expr $(, $arg:expr)* $(,)?) => {{
        let err = *crate::htslib_rs::c_compat::__errno_location();
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), $format $(, $arg)*);
        if err != 0 {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c": %s".as_ptr(),
                libc::strerror(err),
            );
        }
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"\n".as_ptr());
        libc::exit(libc::EXIT_FAILURE);
    }};
}

// original: check_offset (htslib/test/hfile.c:51)
pub unsafe fn test_hfile_c_51_check_offset(
    f: *mut hFILE,
    off: libc::off_t,
    message: *const c_char,
) {
    let ret = htell(f);
    if ret < 0 {
        test_hfile_c_38_fail!(c"htell(%s)".as_ptr(), message);
    }
    if ret == off {
        return;
    }

    libc::fprintf(
        crate::htslib_rs::c_compat::stderr.cast(),
        c"%s offset incorrect: expected %ld but got %ld\n".as_ptr(),
        message,
        off as libc::c_long,
        ret as libc::c_long,
    );
    libc::exit(libc::EXIT_FAILURE);
}

// original: slurp (htslib/test/hfile.c:62)
pub unsafe fn test_hfile_c_62_slurp(filename: *const c_char) -> *mut c_char {
    let f = libc::fopen(filename, c"rb".as_ptr());
    if f.is_null() {
        test_hfile_c_38_fail!(c"fopen(\"%s\", \"rb\")".as_ptr(), filename);
    }

    let mut sbuf: libc::stat = std::mem::zeroed();
    if libc::fstat(libc::fileno(f), &mut sbuf) != 0 {
        test_hfile_c_38_fail!(c"fstat(\"%s\")".as_ptr(), filename);
    }
    let filesize = sbuf.st_size as size_t;

    let text = libc::malloc(filesize + 1).cast::<c_char>();
    if text.is_null() {
        test_hfile_c_38_fail!(c"malloc(text)".as_ptr());
    }

    if libc::fread(text.cast(), 1, filesize, f) != filesize {
        test_hfile_c_38_fail!(c"fread".as_ptr());
    }
    libc::fclose(f);

    *text.add(filesize) = 0;
    text
}

static mut TEST_HFILE_FIN: *mut hFILE = std::ptr::null_mut();
static mut TEST_HFILE_FOUT: *mut hFILE = std::ptr::null_mut();

// original: reopen (htslib/test/hfile.c:85)
pub unsafe fn test_hfile_c_85_reopen(infname: *const c_char, outfname: *const c_char) {
    if !TEST_HFILE_FIN.is_null() && hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hclose(input)".as_ptr());
    }
    if !TEST_HFILE_FOUT.is_null() && hclose(TEST_HFILE_FOUT) != 0 {
        test_hfile_c_38_fail!(c"hclose(output)".as_ptr());
    }

    TEST_HFILE_FIN = hopen(infname, c"r".as_ptr());
    if TEST_HFILE_FIN.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"%s\")".as_ptr(), infname);
    }

    TEST_HFILE_FOUT = hopen(outfname, c"w".as_ptr());
    if TEST_HFILE_FOUT.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"%s\")".as_ptr(), outfname);
    }
}

// original: main (htslib/test/hfile.c:97)
pub unsafe fn test_hfile_c_97_main() -> c_int {
    static SIZE: [c_int; 5] = [1, 13, 403, 999, 30000];

    let mut buffer = [0 as c_char; 40000];
    let mut c: c_int;
    let mut i: c_int;
    let mut n: libc::ssize_t;
    let mut off: libc::off_t;

    test_hfile_c_85_reopen(c"vcf.c".as_ptr(), c"test/hfile1.tmp".as_ptr());
    loop {
        c = hgetc(TEST_HFILE_FIN);
        if c == libc::EOF {
            break;
        }
        if hputc(c, TEST_HFILE_FOUT) == libc::EOF {
            test_hfile_c_38_fail!(c"hputc".as_ptr());
        }
    }
    if herrno(TEST_HFILE_FIN) != 0 {
        *crate::htslib_rs::c_compat::__errno_location() = herrno(TEST_HFILE_FIN);
        test_hfile_c_38_fail!(c"hgetc".as_ptr());
    }

    test_hfile_c_85_reopen(c"test/hfile1.tmp".as_ptr(), c"test/hfile2.tmp".as_ptr());
    if hpeek(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 50) < 0 {
        test_hfile_c_38_fail!(c"hpeek".as_ptr());
    }
    loop {
        n = hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 17);
        if n <= 0 {
            break;
        }
        if hwrite(TEST_HFILE_FOUT, buffer.as_ptr().cast(), n as size_t) != n {
            test_hfile_c_38_fail!(c"hwrite".as_ptr());
        }
    }
    if n < 0 {
        test_hfile_c_38_fail!(c"hread".as_ptr());
    }

    test_hfile_c_85_reopen(c"test/hfile2.tmp".as_ptr(), c"test/hfile3.tmp".as_ptr());
    loop {
        n = hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), buffer.len());
        if n <= 0 {
            break;
        }
        if hwrite(TEST_HFILE_FOUT, buffer.as_ptr().cast(), n as size_t) != n {
            test_hfile_c_38_fail!(c"hwrite".as_ptr());
        }
        if hpeek(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 700) < 0 {
            test_hfile_c_38_fail!(c"hpeek".as_ptr());
        }
    }
    if n < 0 {
        test_hfile_c_38_fail!(c"hread".as_ptr());
    }

    test_hfile_c_85_reopen(c"test/hfile3.tmp".as_ptr(), c"test/hfile4.tmp".as_ptr());
    i = 0;
    off = 0;
    loop {
        n = hread(
            TEST_HFILE_FIN,
            buffer.as_mut_ptr().cast(),
            SIZE[(i % 5) as usize] as size_t,
        );
        i += 1;
        if n <= 0 {
            break;
        }
        off += n as libc::off_t;
        buffer[n as usize] = 0;
        test_hfile_c_51_check_offset(TEST_HFILE_FIN, off, c"pre-peek".as_ptr());
        if hputs(buffer.as_ptr(), TEST_HFILE_FOUT) == libc::EOF {
            test_hfile_c_38_fail!(c"hputs".as_ptr());
        }
        n = hpeek(
            TEST_HFILE_FIN,
            buffer.as_mut_ptr().cast(),
            SIZE[((i + 3) % 5) as usize] as size_t,
        );
        if n < 0 {
            test_hfile_c_38_fail!(c"hpeek".as_ptr());
        }
        test_hfile_c_51_check_offset(TEST_HFILE_FIN, off, c"post-peek".as_ptr());
    }
    if n < 0 {
        test_hfile_c_38_fail!(c"hread".as_ptr());
    }

    test_hfile_c_85_reopen(c"test/hfile4.tmp".as_ptr(), c"test/hfile5.tmp".as_ptr());
    while !hgets(buffer.as_mut_ptr(), 80, TEST_HFILE_FIN).is_null() {
        let l = libc::strlen(buffer.as_ptr());
        if l > 79 {
            test_hfile_c_38_fail!(c"hgets read %zu bytes, should be < 80".as_ptr(), l);
        }
        if hwrite(TEST_HFILE_FOUT, buffer.as_ptr().cast(), l) != l as libc::ssize_t {
            test_hfile_c_38_fail!(c"hwrite".as_ptr());
        }
    }
    if herrno(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hgets".as_ptr());
    }

    test_hfile_c_85_reopen(c"test/hfile5.tmp".as_ptr(), c"test/hfile6.tmp".as_ptr());
    n = hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 200);
    if n < 0 {
        test_hfile_c_38_fail!(c"hread".as_ptr());
    } else if n != 200 {
        test_hfile_c_38_fail!(c"hread only got %d".as_ptr(), n as c_int);
    }
    if hwrite(TEST_HFILE_FOUT, buffer.as_ptr().cast(), 1000) != 1000 {
        test_hfile_c_38_fail!(c"hwrite".as_ptr());
    }
    test_hfile_c_51_check_offset(TEST_HFILE_FIN, 200, c"input/first200".as_ptr());
    test_hfile_c_51_check_offset(TEST_HFILE_FOUT, 1000, c"output/first200".as_ptr());

    if hseek(TEST_HFILE_FIN, 800, libc::SEEK_CUR) < 0 {
        test_hfile_c_38_fail!(c"hseek/cur".as_ptr());
    }
    test_hfile_c_51_check_offset(TEST_HFILE_FIN, 1000, c"input/seek".as_ptr());
    off = 1000;
    loop {
        n = hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), buffer.len());
        if n <= 0 {
            break;
        }
        if hwrite(TEST_HFILE_FOUT, buffer.as_ptr().cast(), n as size_t) != n {
            test_hfile_c_38_fail!(c"hwrite".as_ptr());
        }
        off += n as libc::off_t;
    }
    if n < 0 {
        test_hfile_c_38_fail!(c"hread".as_ptr());
    }
    test_hfile_c_51_check_offset(TEST_HFILE_FIN, off, c"input/eof".as_ptr());
    test_hfile_c_51_check_offset(TEST_HFILE_FOUT, off, c"output/eof".as_ptr());

    if hseek(TEST_HFILE_FIN, 200, libc::SEEK_SET) < 0 {
        test_hfile_c_38_fail!(c"hseek/set".as_ptr());
    }
    if hseek(TEST_HFILE_FOUT, 200, libc::SEEK_SET) < 0 {
        test_hfile_c_38_fail!(c"hseek(output)".as_ptr());
    }
    test_hfile_c_51_check_offset(TEST_HFILE_FIN, 200, c"input/backto200".as_ptr());
    test_hfile_c_51_check_offset(TEST_HFILE_FOUT, 200, c"output/backto200".as_ptr());
    n = hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 800);
    if n < 0 {
        test_hfile_c_38_fail!(c"hread".as_ptr());
    } else if n != 800 {
        test_hfile_c_38_fail!(c"hread only got %d".as_ptr(), n as c_int);
    }
    if hwrite(TEST_HFILE_FOUT, buffer.as_ptr().cast(), 800) != 800 {
        test_hfile_c_38_fail!(c"hwrite".as_ptr());
    }
    test_hfile_c_51_check_offset(TEST_HFILE_FIN, 1000, c"input/wrote800".as_ptr());
    test_hfile_c_51_check_offset(TEST_HFILE_FOUT, 1000, c"output/wrote800".as_ptr());

    if hflush(TEST_HFILE_FOUT) == libc::EOF {
        test_hfile_c_38_fail!(c"hflush".as_ptr());
    }

    let original = test_hfile_c_62_slurp(c"vcf.c".as_ptr());
    for i in 1..=6 {
        let text: *mut c_char;
        libc::snprintf(
            buffer.as_mut_ptr(),
            buffer.len(),
            c"test/hfile%d.tmp".as_ptr(),
            i,
        );
        text = test_hfile_c_62_slurp(buffer.as_ptr());
        if libc::strcmp(original, text) != 0 {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"%s differs from vcf.c\n".as_ptr(),
                buffer.as_ptr(),
            );
            libc::free(text.cast());
            libc::free(original.cast());
            return libc::EXIT_FAILURE;
        }
        libc::free(text.cast());
    }
    libc::free(original.cast());

    if hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hclose(input)".as_ptr());
    }
    if hclose(TEST_HFILE_FOUT) != 0 {
        test_hfile_c_38_fail!(c"hclose(output)".as_ptr());
    }

    TEST_HFILE_FOUT = hopen(c"test/hfile_chars.tmp".as_ptr(), c"w".as_ptr());
    if TEST_HFILE_FOUT.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"test/hfile_chars.tmp\")".as_ptr());
    }
    for i in 0..256 {
        if hputc(i, TEST_HFILE_FOUT) != i {
            test_hfile_c_38_fail!(c"chars: hputc (%d)".as_ptr(), i);
        }
    }
    if hclose(TEST_HFILE_FOUT) != 0 {
        test_hfile_c_38_fail!(c"hclose(test/hfile_chars.tmp)".as_ptr());
    }

    TEST_HFILE_FIN = hopen(c"test/hfile_chars.tmp".as_ptr(), c"r".as_ptr());
    if TEST_HFILE_FIN.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"test/hfile_chars.tmp\") for reading".as_ptr());
    }
    for i in 0..256 {
        c = hgetc(TEST_HFILE_FIN);
        if c != i {
            test_hfile_c_38_fail!(
                c"chars: hgetc (%d = 0x%x) returned %d = 0x%x".as_ptr(),
                i,
                i,
                c,
                c,
            );
        }
    }
    c = hgetc(TEST_HFILE_FIN);
    if c != libc::EOF {
        test_hfile_c_38_fail!(c"chars: hgetc (EOF) returned %d".as_ptr(), c);
    }
    if hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hclose(test/hfile_chars.tmp) for reading".as_ptr());
    }

    TEST_HFILE_FIN = hopen(c"preload:test/hfile_chars.tmp".as_ptr(), c"r".as_ptr());
    if TEST_HFILE_FIN.is_null() {
        test_hfile_c_38_fail!(c"preloading \"test/hfile_chars.tmp\" for reading".as_ptr());
    }
    for i in 0..256 {
        c = hgetc(TEST_HFILE_FIN);
        if c != i {
            test_hfile_c_38_fail!(
                c"preloading chars: hgetc (%d = 0x%x) returned %d = 0x%x".as_ptr(),
                i,
                i,
                c,
                c,
            );
        }
    }
    c = hgetc(TEST_HFILE_FIN);
    if c != libc::EOF {
        test_hfile_c_38_fail!(c"preloading chars: hgetc (EOF) returned %d".as_ptr(), c);
    }
    if hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"preloading hclose(test/hfile_chars.tmp) for reading".as_ptr());
    }

    let mut test_string = libc::strdup(c"Test string".as_ptr());
    // Native hopen is non-variadic, so route the mem-buffer open through the
    // function the variadic mem vopen handler dispatches to, giving a native
    // MEM_BACKEND hfile that the native hfile_mem_get_buffer recognises.
    TEST_HFILE_FIN = hfile_c_878_hopenv_mem(c"mem:".as_ptr(), c"r:".as_ptr(), test_string, 12usize);
    if TEST_HFILE_FIN.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"mem:\", \"r:\", ...)".as_ptr());
    }
    if hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 12) != 12 {
        test_hfile_c_38_fail!(c"hopen('mem:', 'r') failed read".as_ptr());
    }
    if libc::strcmp(buffer.as_ptr(), test_string) != 0 {
        test_hfile_c_38_fail!(
            c"hopen('mem:', 'r') missread '%s' != '%s'".as_ptr(),
            buffer.as_ptr(),
            test_string,
        );
    }
    let mut interval_buf_len: size_t = 0;
    let mut internal_buf = hfile_mem_get_buffer(TEST_HFILE_FIN, &mut interval_buf_len);
    if internal_buf.is_null() {
        test_hfile_c_38_fail!(c"hopen('mem:', 'r') failed to get internal buffer".as_ptr());
    }
    if hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hclose mem for reading".as_ptr());
    }

    test_string = libc::strdup(c"Test string".as_ptr());
    TEST_HFILE_FIN = hfile_c_878_hopenv_mem(c"mem:".as_ptr(), c"wr:".as_ptr(), test_string, 12usize);
    if TEST_HFILE_FIN.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"mem:\", \"w:\", ...)".as_ptr());
    }
    if hseek(TEST_HFILE_FIN, -1, libc::SEEK_END) < 0 {
        test_hfile_c_38_fail!(c"hopen('mem:', 'wr') failed seek".as_ptr());
    }
    if hwrite(TEST_HFILE_FIN, c" extra".as_ptr().cast(), 7) != 7 {
        test_hfile_c_38_fail!(c"hopen('mem:', 'wr') failed write".as_ptr());
    }
    if hseek(TEST_HFILE_FIN, 0, libc::SEEK_SET) < 0 {
        test_hfile_c_38_fail!(c"hopen('mem:', 'wr') failed seek".as_ptr());
    }
    if hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 18) != 18 {
        test_hfile_c_38_fail!(c"hopen('mem:', 'wr') failed read".as_ptr());
    }
    if libc::strcmp(buffer.as_ptr(), c"Test string extra".as_ptr()) != 0 {
        test_hfile_c_38_fail!(
            c"hopen('mem:', 'wr') misswrote '%s' != '%s'".as_ptr(),
            buffer.as_ptr(),
            c"Test string extra".as_ptr(),
        );
    }
    internal_buf = hfile_mem_steal_buffer(TEST_HFILE_FIN, &mut interval_buf_len);
    if internal_buf.is_null() {
        test_hfile_c_38_fail!(c"hopen('mem:', 'wr') failed to get internal buffer".as_ptr());
    }
    libc::free(internal_buf.cast());
    if hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hclose mem for writing".as_ptr());
    }

    TEST_HFILE_FIN = hopen(c"data:,hello, world!%0A".as_ptr(), c"r".as_ptr());
    if TEST_HFILE_FIN.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"data:...\")".as_ptr());
    }
    n = hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 300);
    if n < 0 {
        test_hfile_c_38_fail!(c"hread".as_ptr());
    }
    buffer[n as usize] = 0;
    if libc::strcmp(buffer.as_ptr(), c"hello, world!\n".as_ptr()) != 0 {
        test_hfile_c_38_fail!(c"hread result".as_ptr());
    }
    if hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hclose(\"data:...\")".as_ptr());
    }

    TEST_HFILE_FIN = hopen(c"test/emptyfile".as_ptr(), c"r".as_ptr());
    if TEST_HFILE_FIN.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"test/emptyfile\") for reading".as_ptr());
    }
    if hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 100) != 0 {
        test_hfile_c_38_fail!(c"test/emptyfile is non-empty".as_ptr());
    }
    if hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hclose(\"test/emptyfile\") for reading".as_ptr());
    }

    TEST_HFILE_FIN = hopen(c"data:,".as_ptr(), c"r".as_ptr());
    if TEST_HFILE_FIN.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"data:\") for reading".as_ptr());
    }
    if hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 100) != 0 {
        test_hfile_c_38_fail!(c"empty data: URL is non-empty".as_ptr());
    }
    if hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hclose(\"data:\") for reading".as_ptr());
    }

    TEST_HFILE_FIN = hopen(
        c"data:;base64,\
TWFuIGlzIGRpc3Rpbmd1aXNoZWQsIG5vdCBvbmx5IGJ5IGhpcyByZWFzb24sIGJ1dCBieSB0aGlz\
IHNpbmd1bGFyIHBhc3Npb24gZnJvbSBvdGhlciBhbmltYWxzLCB3aGljaCBpcyBhIGx1c3Qgb2Yg\
dGhlIG1pbmQsIHRoYXQgYnkgYSBwZXJzZXZlcmFuY2Ugb2YgZGVsaWdodCBpbiB0aGUgY29udGlu\
dWVkIGFuZCBpbmRlZmF0aWdhYmxlIGdlbmVyYXRpb24gb2Yga25vd2xlZGdlLCBleGNlZWRzIHRo\
ZSBzaG9ydCB2ZWhlbWVuY2Ugb2YgYW55IGNhcm5hbCBwbGVhc3VyZS4="
            .as_ptr(),
        c"r".as_ptr(),
    );
    if TEST_HFILE_FIN.is_null() {
        test_hfile_c_38_fail!(c"hopen(\"data:;base64,...\")".as_ptr());
    }
    n = hread(TEST_HFILE_FIN, buffer.as_mut_ptr().cast(), 300);
    if n < 0 {
        test_hfile_c_38_fail!(c"hread for base64".as_ptr());
    }
    buffer[n as usize] = 0;
    if libc::strcmp(
        buffer.as_ptr(),
        c"Man is distinguished, not only by his reason, but by \
this singular passion from other animals, which is a lust of the mind, that \
by a perseverance of delight in the continued and indefatigable generation \
of knowledge, exceeds the short vehemence of any carnal pleasure."
            .as_ptr(),
    ) != 0
    {
        test_hfile_c_38_fail!(c"hread result for base64".as_ptr());
    }
    if hclose(TEST_HFILE_FIN) != 0 {
        test_hfile_c_38_fail!(c"hclose(\"data:;base64,...\")".as_ptr());
    }

    let mut kstr = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };

    if libc::strcmp(
        haddextension(&mut kstr, c"foo/bar.bam".as_ptr(), 0, c".bai".as_ptr()),
        c"foo/bar.bam.bai".as_ptr(),
    ) != 0
    {
        test_hfile_c_38_fail!(c"haddextension foo/bar.bam[.bai]".as_ptr());
    }
    if libc::strcmp(
        haddextension(&mut kstr, c"foo/bar.bam".as_ptr(), 1, c".bai".as_ptr()),
        c"foo/bar.bai".as_ptr(),
    ) != 0
    {
        test_hfile_c_38_fail!(c"haddextension foo/bar[.bai]".as_ptr());
    }
    if libc::strcmp(
        haddextension(&mut kstr, c"foo.bar/baz".as_ptr(), 1, c".bai".as_ptr()),
        c"foo.bar/baz.bai".as_ptr(),
    ) != 0
    {
        test_hfile_c_38_fail!(c"haddextension foo.bar/baz[.bai]".as_ptr());
    }
    if libc::strcmp(
        haddextension(&mut kstr, c"foo#bar.bam".as_ptr(), 0, c".bai".as_ptr()),
        c"foo#bar.bam.bai".as_ptr(),
    ) != 0
    {
        test_hfile_c_38_fail!(c"haddextension foo#bar.bam[.bai]".as_ptr());
    }
    if libc::strcmp(
        haddextension(&mut kstr, c".bam".as_ptr(), 1, c".bai".as_ptr()),
        c".bai".as_ptr(),
    ) != 0
    {
        test_hfile_c_38_fail!(c"haddextension [.bai]".as_ptr());
    }
    if libc::strcmp(
        haddextension(&mut kstr, c"foo".as_ptr(), 1, c".csi".as_ptr()),
        c"foo.csi".as_ptr(),
    ) != 0
    {
        test_hfile_c_38_fail!(c"haddextension foo[.csi]".as_ptr());
    }

    if libc::strcmp(
        haddextension(
            &mut kstr,
            c"http://host/bar.cram?a&b&c".as_ptr(),
            0,
            c".crai".as_ptr(),
        ),
        c"http://host/bar.cram.crai?a&b&c".as_ptr(),
    ) != 0
    {
        test_hfile_c_38_fail!(c"haddextension http://host/bar.cram[.crai]?a&b&c".as_ptr());
    }

    if libc::strcmp(
        haddextension(
            &mut kstr,
            c"http://host/bar.cram#frag".as_ptr(),
            1,
            c".crai".as_ptr(),
        ),
        c"http://host/bar.crai#frag".as_ptr(),
    ) != 0
    {
        test_hfile_c_38_fail!(c"haddextension http://host/bar[.crai]#frag".as_ptr());
    }

    libc::free(ks_release(&mut kstr).cast());

    libc::EXIT_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::process::Command;

    fn temp_dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "htslib-rs-test-hfile-main-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ))
    }

    #[test]
    fn original_hfile_main_child_entry() {
        let Some(workdir) = std::env::var_os("HTSLIB_RS_HFILE_MAIN_WORKDIR") else {
            return;
        };
        unsafe {
            let cwd = CString::new(
                std::path::PathBuf::from(workdir)
                    .to_string_lossy()
                    .as_bytes(),
            )
            .unwrap();
            if libc::chdir(cwd.as_ptr()) != 0 {
                std::process::exit(120);
            }
            std::process::exit(test_hfile_c_97_main());
        }
    }

    #[test]
    fn original_hfile_main_runs_in_isolated_workdir() {
        let workdir = temp_dir();
        let test_dir = workdir.join("test");
        let _ = std::fs::remove_dir_all(&workdir);
        std::fs::create_dir_all(&test_dir).unwrap();
        std::fs::write(
            workdir.join("vcf.c"),
            b"line one\nline two with more text\nline three\n".repeat(200),
        )
        .unwrap();
        std::fs::write(test_dir.join("emptyfile"), b"").unwrap();

        let output = Command::new(std::env::current_exe().unwrap())
            .env("HTSLIB_RS_HFILE_MAIN_WORKDIR", &workdir)
            .args([
                "htslib_rs::test::hfile::tests::original_hfile_main_child_entry",
                "--exact",
                "--nocapture",
                "--test-threads=1",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "child stdout:\n{}\nchild stderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let _ = std::fs::remove_dir_all(workdir);
    }
}
