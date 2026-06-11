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

use crate::htslib_rs::sam;

unsafe extern "C" {
    static mut optarg: *mut u8;
}

static mut NIBBLE: [u8; 5000] = [0; 5000];
static mut BUF: [u8; 10000] = [0; 10000];

// original: gettime (htslib/test/test_nibbles.c:41)
pub unsafe fn test_test_nibbles_c_41_gettime() -> i64 {
    let mut ts: libc::timespec = std::mem::zeroed();
    libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut ts);
    ts.tv_sec as i64 * 1_000_000_000 + ts.tv_nsec as i64
}

// original: fmttime (htslib/test/test_nibbles.c:53)
pub unsafe fn test_test_nibbles_c_53_fmttime(elapsed: i64) -> *mut u8 {
    static mut BUF: [u8; 64] = [0; 64];

    let sec = elapsed / 1_000_000_000;
    let nsec = elapsed % 1_000_000_000;
    let buf = std::ptr::addr_of_mut!(BUF).cast::<u8>();
    libc::sprintf(
        buf.cast(),
        c"%lld.%09lld processor seconds".as_ptr(),
        sec as libc::c_longlong,
        nsec as libc::c_longlong,
    );
    buf
}

// original: nibble2base_single (htslib/test/test_nibbles.c:69)
pub unsafe fn test_test_nibbles_c_69_nibble2base_single(
    nib: *mut u8,
    seq: *mut u8,
    len: i32,
) {
    static SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

    let mut i = 0;
    while i < len {
        *seq.add(i as usize) = SEQ_NT16_STR[sam::bam_seqi(nib, i as usize) as usize];
        i += 1;
    }
}

// original: validate_nibble2base (htslib/test/test_nibbles.c:78)
pub unsafe fn test_test_nibbles_c_78_validate_nibble2base() -> i32 {
    let mut defbuf = [0u8; 500];
    let nibble = std::ptr::addr_of_mut!(NIBBLE).cast::<u8>();
    let buf = std::ptr::addr_of_mut!(BUF).cast::<u8>();
    let mut total = 0u64;
    let mut failed = 0u64;

    let mut i = 0usize;
    while i < 5000 {
        *nibble.add(i) = (i % 256) as u8;
        i += 1;
    }

    let mut start = 0;
    while start < 80 {
        let mut len = 0;
        while len < 400 {
            defbuf.fill(0);
            test_test_nibbles_c_69_nibble2base_single(nibble.add(start), defbuf.as_mut_ptr(), len);

            std::slice::from_raw_parts_mut(buf, defbuf.len()).fill(0);
            let seq_len = len as usize;
            let packed = std::slice::from_raw_parts(nibble.add(start), seq_len.div_ceil(2));
            let out = std::slice::from_raw_parts_mut(buf, seq_len);
            sam::nibble2base(packed, out);

            total += 1;
            let buf_slice = std::slice::from_raw_parts(buf, seq_len);
            if defbuf[..seq_len] != *buf_slice {
                eprintln!(
                    "{} expected\n{} FAIL\n",
                    String::from_utf8_lossy(&defbuf[..seq_len]),
                    String::from_utf8_lossy(buf_slice)
                );
                failed += 1;
            }

            len += 1;
        }
        start += 1;
    }

    if failed > 0 {
        eprintln!("Failures: {failed} (out of {total} tests)");
        return 1;
    }

    0
}

// original: time_nibble2base (htslib/test/test_nibbles.c:109)
pub unsafe fn test_test_nibbles_c_109_time_nibble2base(length: i32, count: u64) -> i32 {
    let nibble = std::ptr::addr_of_mut!(NIBBLE).cast::<u8>();
    let buf = std::ptr::addr_of_mut!(BUF).cast::<u8>();
    let mut total = 0u64;

    let mut i = 0u64;
    while i < length as u64 {
        *nibble.add(i as usize) = (i % 256) as u8;
        i += 1;
    }

    println!("Timing {count} nibble2base iterations with read length {length}...");
    let start = test_test_nibbles_c_41_gettime();

    i = 0;
    while i < count {
        let len = length as usize;
        let packed = std::slice::from_raw_parts(nibble, len.div_ceil(2));
        let out = std::slice::from_raw_parts_mut(buf, len);
        sam::nibble2base(packed, out);
        total = total.wrapping_add(*buf.add((i % length as u64) as usize) as u64);
        i += 1;
    }

    let stop = test_test_nibbles_c_41_gettime();
    let fmt = test_test_nibbles_c_53_fmttime(stop - start);
    let fmt_str = std::ffi::CStr::from_ptr(fmt.cast()).to_string_lossy();
    println!("{fmt_str} (summing to {total})");
    0
}

// original: main (htslib/test/test_nibbles.c:128)
pub unsafe fn test_test_nibbles_c_128_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut readlen = 5000;
    let mut count = 1_000_000u64;
    let mut status = 0;

    if argc == 1 {
        println!(
            "Usage: test_nibbles [-c NUM] [-r NUM] [-n|-v]...\nOptions:\n  -c NUM  Specify number of iterations [{count}]\n  -n      Run nibble2base speed tests\n  -r NUM  Specify read length [{readlen}]\n  -v      Run all validation tests"
        );
    }

    loop {
        let c = libc::getopt(argc, argv.cast(), c"c:nr:v".as_ptr());
        if c < 0 {
            break;
        }
        match c as u8 {
            b'c' => {
                count = libc::strtoul(optarg.cast(), std::ptr::null_mut(), 0);
            }
            b'n' => {
                status += test_test_nibbles_c_109_time_nibble2base(readlen, count);
            }
            b'r' => {
                readlen = libc::atoi(optarg.cast());
            }
            b'v' => {
                status += test_test_nibbles_c_78_validate_nibble2base();
            }
            _ => {}
        }
    }

    status
}
