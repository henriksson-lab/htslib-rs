/*  test/test_nibbles.c -- Test SIMD optimised function implementations.

    Copyright (C) 2024 Centre for Population Genomics.

    Author: John Marshall <jmarshall@hey.com>

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

use std::ffi::{c_char, c_int, c_uchar, c_ulong};

use crate::htslib_rs::sam;

unsafe extern "C" {
    static mut optarg: *mut c_char;
}

static mut NIBBLE: [c_uchar; 5000] = [0; 5000];
static mut BUF: [c_char; 10000] = [0; 10000];

// original: gettime (htslib/test/test_nibbles.c:41)
pub unsafe fn test_test_nibbles_c_41_gettime() -> i64 {
    let mut ts: libc::timespec = std::mem::zeroed();
    libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut ts);
    ts.tv_sec as i64 * 1_000_000_000 + ts.tv_nsec as i64
}

// original: fmttime (htslib/test/test_nibbles.c:53)
pub unsafe fn test_test_nibbles_c_53_fmttime(elapsed: i64) -> *mut c_char {
    static mut BUF: [c_char; 64] = [0; 64];

    let sec = elapsed / 1_000_000_000;
    let nsec = elapsed % 1_000_000_000;
    let buf = std::ptr::addr_of_mut!(BUF).cast::<c_char>();
    libc::sprintf(
        buf,
        c"%lld.%09lld processor seconds".as_ptr(),
        sec as libc::c_longlong,
        nsec as libc::c_longlong,
    );
    buf
}

// original: nibble2base_single (htslib/test/test_nibbles.c:69)
pub unsafe fn test_test_nibbles_c_69_nibble2base_single(
    nib: *mut c_uchar,
    seq: *mut c_char,
    len: c_int,
) {
    static SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

    let mut i = 0;
    while i < len {
        *seq.add(i as usize) = SEQ_NT16_STR[sam::bam_seqi(nib, i as usize) as usize] as c_char;
        i += 1;
    }
}

// original: validate_nibble2base (htslib/test/test_nibbles.c:78)
pub unsafe fn test_test_nibbles_c_78_validate_nibble2base() -> c_int {
    let mut defbuf = [0 as c_char; 500];
    let nibble = std::ptr::addr_of_mut!(NIBBLE).cast::<c_uchar>();
    let buf = std::ptr::addr_of_mut!(BUF).cast::<c_char>();
    let mut total = 0 as libc::c_ulonglong;
    let mut failed = 0 as libc::c_ulonglong;

    let mut i = 0usize;
    while i < 5000 {
        *nibble.add(i) = (i % 256) as c_uchar;
        i += 1;
    }

    let mut start = 0;
    while start < 80 {
        let mut len = 0;
        while len < 400 {
            libc::memset(
                defbuf.as_mut_ptr().cast(),
                b'\0' as c_int,
                std::mem::size_of_val(&defbuf),
            );
            test_test_nibbles_c_69_nibble2base_single(nibble.add(start), defbuf.as_mut_ptr(), len);

            libc::memset(buf.cast(), b'\0' as c_int, std::mem::size_of_val(&defbuf));
            sam::nibble2base(nibble.add(start), buf, len);

            total += 1;
            if libc::strcmp(defbuf.as_ptr(), buf) != 0 {
                libc::printf(c"%s expected\n%s FAIL\n\n".as_ptr(), defbuf.as_ptr(), buf);
                failed += 1;
            }

            len += 1;
        }
        start += 1;
    }

    if failed > 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Failures: %llu (out of %llu tests)\n".as_ptr(),
            failed,
            total,
        );
        return 1;
    }

    0
}

// original: time_nibble2base (htslib/test/test_nibbles.c:109)
pub unsafe fn test_test_nibbles_c_109_time_nibble2base(length: c_int, count: c_ulong) -> c_int {
    let nibble = std::ptr::addr_of_mut!(NIBBLE).cast::<c_uchar>();
    let buf = std::ptr::addr_of_mut!(BUF).cast::<c_char>();
    let mut total = 0 as c_ulong;

    let mut i = 0 as c_ulong;
    while i < length as c_ulong {
        *nibble.add(i as usize) = (i % 256) as c_uchar;
        i += 1;
    }

    libc::printf(
        c"Timing %lu nibble2base iterations with read length %d...\n".as_ptr(),
        count,
        length,
    );
    let start = test_test_nibbles_c_41_gettime();

    i = 0;
    while i < count {
        sam::nibble2base(nibble, buf, length);
        total = total.wrapping_add(*buf.add((i % length as c_ulong) as usize) as c_ulong);
        i += 1;
    }

    let stop = test_test_nibbles_c_41_gettime();
    libc::printf(
        c"%s (summing to %lu)\n".as_ptr(),
        test_test_nibbles_c_53_fmttime(stop - start),
        total,
    );
    0
}

// original: main (htslib/test/test_nibbles.c:128)
pub unsafe fn test_test_nibbles_c_128_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut readlen = 5000;
    let mut count = 1_000_000 as c_ulong;
    let mut status = 0;

    if argc == 1 {
        libc::printf(
            c"Usage: test_nibbles [-c NUM] [-r NUM] [-n|-v]...\nOptions:\n  -c NUM  Specify number of iterations [%lu]\n  -n      Run nibble2base speed tests\n  -r NUM  Specify read length [%d]\n  -v      Run all validation tests\n".as_ptr(),
            count,
            readlen,
        );
    }

    loop {
        let c = libc::getopt(argc, argv, c"c:nr:v".as_ptr());
        if c < 0 {
            break;
        }
        match c as u8 {
            b'c' => {
                count = libc::strtoul(optarg, std::ptr::null_mut(), 0);
            }
            b'n' => {
                status += test_test_nibbles_c_109_time_nibble2base(readlen, count);
            }
            b'r' => {
                readlen = libc::atoi(optarg);
            }
            b'v' => {
                status += test_test_nibbles_c_78_validate_nibble2base();
            }
            _ => {}
        }
    }

    status
}
