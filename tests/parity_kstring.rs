// Parity tests for functions recently MOVED from src/hts.rs into src/kstring.rs
// during the canonical-mapping realignment. For each function we run the native
// Rust translation and the equivalent C HTSlib function (via hts_sys), then
// assert byte-identical results.
//
// Functions covered (from htslib/kstring.c):
//   - ksplit_core    (free function, exposed in kstring.h)
//   - ksplit         (static inline in kstring.h; we test via ksplit_core+offset
//                     wrapper for the native side and compare offsets)
//   - kgetline       (free function; uses an fgets-like callback)
//   - kgetline2      (free function; uses an hgetln-like callback)
//   - kstrtok        (free function)
//   - kmemmem        (free function)
//   - kstrstr        (free function)
//   - kstrnstr       (free function)
//   - kputd          (free function)
//   - kfgetline      (Rust-only convenience over kgetline; no direct hts_sys
//                     equivalent — covered with native-only smoke test below)

#![cfg(feature = "parity")]

use htslib_rs::{kstring_t, ks_tokaux_t};
use std::ffi::{c_char, c_int, c_void, CStr, CString};

// ---------------------------------------------------------------------------
// kputd
// ---------------------------------------------------------------------------

unsafe fn kputd_native(d: f64) -> (Vec<u8>, c_int) {
    let mut s: kstring_t = std::mem::zeroed();
    let ret = htslib_rs::kstring::kputd(d, &mut s);
    let bytes = if s.s.is_null() {
        Vec::new()
    } else {
        CStr::from_ptr(s.s).to_bytes().to_vec()
    };
    if !s.s.is_null() {
        libc::free(s.s.cast());
    }
    (bytes, ret)
}

unsafe fn kputd_c(d: f64) -> (Vec<u8>, c_int) {
    let mut s: hts_sys::kstring_t = std::mem::zeroed();
    let ret = hts_sys::kputd(d, &mut s);
    let bytes = if s.s.is_null() {
        Vec::new()
    } else {
        CStr::from_ptr(s.s).to_bytes().to_vec()
    };
    if !s.s.is_null() {
        libc::free(s.s.cast());
    }
    (bytes, ret)
}

#[test]
fn parity_kputd_various_values() {
    let values: &[f64] = &[
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.1,
        -0.1,
        0.00001,
        0.0001,
        0.5,
        1.5,
        12345.6789,
        999999.0,
        1000000.0,
        1.23e10,
        -1.23e10,
        1e-15,
        1e15,
        1234567890.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    unsafe {
        for &d in values {
            let (n_bytes, n_ret) = kputd_native(d);
            let (c_bytes, c_ret) = kputd_c(d);
            assert_eq!(
                n_bytes, c_bytes,
                "kputd({d}) byte mismatch: native={:?} c={:?}",
                String::from_utf8_lossy(&n_bytes),
                String::from_utf8_lossy(&c_bytes),
            );
            // Both should return >= 0 on success; NaN/inf may differ in exact
            // length but the bytes test above is the real parity check.
            assert_eq!(
                n_ret.signum(),
                c_ret.signum(),
                "kputd({d}) return sign mismatch native={n_ret} c={c_ret}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// ksplit_core / ksplit
// ---------------------------------------------------------------------------

unsafe fn ksplit_core_native(input: &str, delim: c_int) -> (c_int, Vec<u8>, Vec<c_int>) {
    // ksplit_core writes NUL bytes into the input buffer — work on a copy.
    let mut buf: Vec<u8> = input.bytes().chain(std::iter::once(0u8)).collect();
    let mut max: c_int = 0;
    let mut offsets: *mut c_int = std::ptr::null_mut();
    let n = htslib_rs::kstring::ksplit_core(
        buf.as_mut_ptr().cast::<c_char>(),
        delim,
        &mut max,
        &mut offsets,
    );
    let off_slice = if offsets.is_null() {
        Vec::new()
    } else {
        std::slice::from_raw_parts(offsets, n as usize).to_vec()
    };
    if !offsets.is_null() {
        libc::free(offsets.cast());
    }
    (n, buf, off_slice)
}

unsafe fn ksplit_core_c(input: &str, delim: c_int) -> (c_int, Vec<u8>, Vec<c_int>) {
    let mut buf: Vec<u8> = input.bytes().chain(std::iter::once(0u8)).collect();
    let mut max: c_int = 0;
    let mut offsets: *mut c_int = std::ptr::null_mut();
    let n = hts_sys::ksplit_core(
        buf.as_mut_ptr().cast::<c_char>(),
        delim,
        &mut max,
        &mut offsets,
    );
    let off_slice = if offsets.is_null() {
        Vec::new()
    } else {
        std::slice::from_raw_parts(offsets, n as usize).to_vec()
    };
    if !offsets.is_null() {
        libc::free(offsets.cast());
    }
    (n, buf, off_slice)
}

#[test]
fn parity_ksplit_core_various_inputs() {
    let cases: &[(&str, c_int)] = &[
        ("a\tb\tc", b'\t' as c_int),
        ("a,b,c,d,e", b',' as c_int),
        ("singleword", b'\t' as c_int),
        ("", b'\t' as c_int),
        // whitespace splitting (delimiter==0)
        ("  hello  world   foo bar  ", 0),
        ("a\tb\nc d", 0),
        ("a,,b,,c", b',' as c_int), // empty fields between delimiters
        ("/path/to/file", b'/' as c_int),
        ("one|two|three|four|five|six|seven", b'|' as c_int),
        (":a:b:c:", b':' as c_int),
    ];
    unsafe {
        for (input, delim) in cases {
            let (n_n, buf_n, off_n) = ksplit_core_native(input, *delim);
            let (n_c, buf_c, off_c) = ksplit_core_c(input, *delim);
            assert_eq!(n_n, n_c, "ksplit_core n mismatch for {input:?}/{delim}");
            assert_eq!(off_n, off_c, "ksplit_core offsets mismatch for {input:?}/{delim}");
            // The buffer must be byte-identical (both sides should have written
            // the same NUL terminators in the same places).
            assert_eq!(
                buf_n, buf_c,
                "ksplit_core buffer mismatch for {input:?}/{delim}"
            );
        }
    }
}

// ksplit is a static-inline in kstring.h — not exported. We exercise the
// equivalent native wrapper and confirm it produces the same offsets as
// ksplit_core (the wrapper just calls ksplit_core internally on both sides,
// so this is a sanity check for the native wrapper).
#[test]
fn parity_ksplit_native_matches_ksplit_core() {
    unsafe {
        let input = b"foo\tbar\tbaz\0";
        let mut buf = input.to_vec();
        let mut ks = kstring_t {
            l: input.len() - 1,
            m: input.len(),
            s: buf.as_mut_ptr().cast::<c_char>(),
        };
        // ksplit allocates ks.s-pointed offsets; it writes NULs into the buf.
        let mut n: c_int = 0;
        let offsets = htslib_rs::kstring::ksplit(&mut ks, b'\t' as c_int, &mut n);
        assert_eq!(n, 3);
        assert!(!offsets.is_null());
        let off_slice = std::slice::from_raw_parts(offsets, n as usize).to_vec();
        libc::free(offsets.cast());

        let (n_c, _buf_c, off_c) = ksplit_core_c("foo\tbar\tbaz", b'\t' as c_int);
        assert_eq!(n, n_c);
        assert_eq!(off_slice, off_c);
    }
}

// ---------------------------------------------------------------------------
// kstrtok
// ---------------------------------------------------------------------------

// kstrtok writes through `aux` to track state. We iterate the native and C
// implementations in parallel and assert each step produces the same token
// span (start offset + length until next separator).
unsafe fn run_kstrtok_native(input: &CStr, sep: &CStr) -> Vec<(isize, c_int)> {
    let mut aux: ks_tokaux_t = std::mem::zeroed();
    let mut results = Vec::new();
    let base = input.as_ptr();
    let mut p = htslib_rs::kstring::kstrtok(base, sep.as_ptr(), &mut aux);
    while !p.is_null() {
        let start = p.offset_from(base);
        // aux.p points at the separator (or terminator) — distance to start is
        // the token length.
        let end = aux.p;
        let len = end.offset_from(p) as c_int;
        results.push((start, len));
        p = htslib_rs::kstring::kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
        if results.len() > 1024 {
            break; // safety
        }
    }
    results
}

unsafe fn run_kstrtok_c(input: &CStr, sep: &CStr) -> Vec<(isize, c_int)> {
    let mut aux: hts_sys::ks_tokaux_t = std::mem::zeroed();
    let mut results = Vec::new();
    let base = input.as_ptr();
    let mut p = hts_sys::kstrtok(base, sep.as_ptr(), &mut aux);
    while !p.is_null() {
        let start = p.offset_from(base);
        let end = aux.p;
        let len = end.offset_from(p) as c_int;
        results.push((start, len));
        p = hts_sys::kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
        if results.len() > 1024 {
            break;
        }
    }
    results
}

#[test]
fn parity_kstrtok_various_separators() {
    let cases: &[(&CStr, &CStr)] = &[
        (c"a,b,c,d", c","),
        (c"hello world foo bar", c" "),
        (c"a;b,c;d,e", c";,"),
        (c"singleton", c","),
        (c"", c","),
        (c",leading,trailing,", c","),
        (c"path/to/file", c"/"),
        (c"a\tb c\nd", c"\t \n"),
    ];
    unsafe {
        for (input, sep) in cases {
            let n = run_kstrtok_native(input, sep);
            let c = run_kstrtok_c(input, sep);
            assert_eq!(
                n,
                c,
                "kstrtok mismatch for input={:?} sep={:?}",
                input.to_string_lossy(),
                sep.to_string_lossy()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// kmemmem, kstrstr, kstrnstr (substring search)
// ---------------------------------------------------------------------------

#[test]
fn parity_kstrstr_substring_search() {
    let cases: &[(&CStr, &CStr)] = &[
        (c"hello world", c"world"),
        (c"hello world", c"hello"),
        (c"hello world", c"o w"),
        (c"hello world", c"xyz"),
        (c"", c"foo"),
        (c"abcabcabc", c"abc"),
        (c"abcabcabc", c"bca"),
        (c"banana", c"ana"),
        (c"a", c""),
        // long haystack
        (
            c"the quick brown fox jumps over the lazy dog the quick brown fox",
            c"fox",
        ),
    ];
    unsafe {
        for (haystack, needle) in cases {
            let mut prep_n: *mut c_int = std::ptr::null_mut();
            let mut prep_c: *mut c_int = std::ptr::null_mut();
            let nptr =
                htslib_rs::kstring::kstrstr(haystack.as_ptr(), needle.as_ptr(), &mut prep_n);
            let cptr = hts_sys::kstrstr(haystack.as_ptr(), needle.as_ptr(), &mut prep_c);
            let n_off = if nptr.is_null() {
                -1
            } else {
                nptr.offset_from(haystack.as_ptr())
            };
            let c_off = if cptr.is_null() {
                -1
            } else {
                cptr.offset_from(haystack.as_ptr())
            };
            assert_eq!(
                n_off,
                c_off,
                "kstrstr offset mismatch for haystack={:?} needle={:?}",
                haystack.to_string_lossy(),
                needle.to_string_lossy()
            );
            if !prep_n.is_null() {
                libc::free(prep_n.cast());
            }
            if !prep_c.is_null() {
                libc::free(prep_c.cast());
            }
        }
    }
}

#[test]
fn parity_kstrnstr_bounded_search() {
    // (haystack, needle, n)
    let cases: &[(&CStr, &CStr, c_int)] = &[
        (c"hello world", c"world", 11),
        (c"hello world", c"world", 7), // not found within n
        (c"hello world", c"hello", 5),
        (c"hello world", c"hello", 4), // pat longer than n
        (c"abcabcabc", c"abc", 9),
        (c"abcabcabc", c"abc", 2),
        (c"foo", c"", 3), // empty pattern -> haystack
        (c"foo", c"bar", 0), // n<=0
    ];
    unsafe {
        for (haystack, needle, n) in cases {
            let mut prep_n: *mut c_int = std::ptr::null_mut();
            let mut prep_c: *mut c_int = std::ptr::null_mut();
            let nptr = htslib_rs::kstring::kstrnstr(
                haystack.as_ptr(),
                needle.as_ptr(),
                *n,
                &mut prep_n,
            );
            let cptr = hts_sys::kstrnstr(haystack.as_ptr(), needle.as_ptr(), *n, &mut prep_c);
            let n_off = if nptr.is_null() {
                -1
            } else {
                nptr.offset_from(haystack.as_ptr())
            };
            let c_off = if cptr.is_null() {
                -1
            } else {
                cptr.offset_from(haystack.as_ptr())
            };
            assert_eq!(
                n_off,
                c_off,
                "kstrnstr offset mismatch for haystack={:?} needle={:?} n={}",
                haystack.to_string_lossy(),
                needle.to_string_lossy(),
                n
            );
            if !prep_n.is_null() {
                libc::free(prep_n.cast());
            }
            if !prep_c.is_null() {
                libc::free(prep_c.cast());
            }
        }
    }
}

#[test]
fn parity_kmemmem_binary_search() {
    // (haystack, needle)
    let cases: &[(&[u8], &[u8])] = &[
        (b"\x00abc\x00def", b"abc"),
        (b"\x00\x01\x02\x03", b"\x01\x02"),
        (b"hello\0world", b"world"),
        (b"", b"foo"),
        (b"foo", b""),
        (b"AAAB", b"AAB"),
        (b"the quick brown fox", b"quick"),
    ];
    unsafe {
        for (haystack, needle) in cases {
            let mut prep_n: *mut c_int = std::ptr::null_mut();
            let mut prep_c: *mut c_int = std::ptr::null_mut();
            let nptr = htslib_rs::kstring::kmemmem(
                haystack.as_ptr().cast(),
                haystack.len() as c_int,
                needle.as_ptr().cast(),
                needle.len() as c_int,
                &mut prep_n,
            );
            let cptr = hts_sys::kmemmem(
                haystack.as_ptr().cast(),
                haystack.len() as c_int,
                needle.as_ptr().cast(),
                needle.len() as c_int,
                &mut prep_c,
            );
            let n_off = if nptr.is_null() {
                -1
            } else {
                (nptr as *const u8).offset_from(haystack.as_ptr())
            };
            let c_off = if cptr.is_null() {
                -1
            } else {
                (cptr as *const u8).offset_from(haystack.as_ptr())
            };
            assert_eq!(
                n_off, c_off,
                "kmemmem offset mismatch for haystack_len={} needle={:?}",
                haystack.len(),
                needle
            );
            if !prep_n.is_null() {
                libc::free(prep_n.cast());
            }
            if !prep_c.is_null() {
                libc::free(prep_c.cast());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// kgetline / kgetline2 (read line from a callback)
// ---------------------------------------------------------------------------

// Write the test content to a temp file, fopen() it, and run both kgetline
// implementations against the FILE*. They should each read identical lines.

fn write_tmp(name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "htslib-rs-parity-kstring-{}-{name}",
        std::process::id()
    ));
    std::fs::write(&path, contents).expect("write tmp");
    path
}

unsafe fn read_all_with_kgetline_native(file_path: &std::path::Path) -> Vec<Vec<u8>> {
    let cpath = CString::new(file_path.to_str().unwrap()).unwrap();
    let mode = c"r".as_ptr();
    let fp = libc::fopen(cpath.as_ptr(), mode);
    assert!(!fp.is_null());

    let mut s: kstring_t = std::mem::zeroed();
    let mut lines = Vec::new();
    loop {
        s.l = 0;
        if !s.s.is_null() {
            *s.s = 0;
        }
        let ret = htslib_rs::kstring::kgetline(
            &mut s,
            Some(htslib_rs::kstring::fgets_wrapper),
            fp.cast(),
        );
        if ret == libc::EOF {
            break;
        }
        let bytes = std::slice::from_raw_parts(s.s as *const u8, s.l).to_vec();
        lines.push(bytes);
    }
    if !s.s.is_null() {
        libc::free(s.s.cast());
    }
    libc::fclose(fp);
    lines
}

unsafe fn read_all_with_kgetline_c(file_path: &std::path::Path) -> Vec<Vec<u8>> {
    let cpath = CString::new(file_path.to_str().unwrap()).unwrap();
    let mode = c"r".as_ptr();
    let fp = libc::fopen(cpath.as_ptr(), mode);
    assert!(!fp.is_null());

    // Bridge: hts_sys's kgetline wants a kgets_func (extern "C" fn(*mut c_char, c_int, *mut c_void) -> *mut c_char).
    // libc::fgets has that signature too — match types.
    unsafe extern "C" fn fgets_bridge(
        buf: *mut c_char,
        n: c_int,
        fp: *mut c_void,
    ) -> *mut c_char {
        libc::fgets(buf, n, fp.cast::<libc::FILE>())
    }

    let mut s: hts_sys::kstring_t = std::mem::zeroed();
    let mut lines = Vec::new();
    loop {
        s.l = 0;
        if !s.s.is_null() {
            *s.s = 0;
        }
        let ret = hts_sys::kgetline(&mut s, Some(fgets_bridge), fp.cast());
        if ret == libc::EOF {
            break;
        }
        let bytes = std::slice::from_raw_parts(s.s as *const u8, s.l as usize).to_vec();
        lines.push(bytes);
    }
    if !s.s.is_null() {
        libc::free(s.s.cast());
    }
    libc::fclose(fp);
    lines
}

#[test]
fn parity_kgetline_synthetic_stream() {
    let contents: &[u8] = b"first line\nsecond\r\nthird with no newline at end";
    let path = write_tmp("kgetline.txt", contents);
    unsafe {
        let n = read_all_with_kgetline_native(&path);
        let c = read_all_with_kgetline_c(&path);
        assert_eq!(n, c, "kgetline line set mismatch");
        assert_eq!(n.len(), 3);
        assert_eq!(&n[0], b"first line");
        assert_eq!(&n[1], b"second");
        assert_eq!(&n[2], b"third with no newline at end");
    }
    let _ = std::fs::remove_file(path);
}

// On Linux64: htslib_rs::size_t = usize = u64, hts_sys::size_t = c_ulong = u64.
// On Linux64: isize = i64 = c_long = hts_sys::ssize_t. So callbacks below are
// ABI-compatible across both sides (both reduce to extern "C" fn(*mut c_char,
// u64, *mut c_void) -> i64).

unsafe extern "C" fn fgetln_bridge_native(
    buf: *mut c_char,
    sz: htslib_rs::size_t,
    fp: *mut c_void,
) -> isize {
    let fp = fp.cast::<libc::FILE>();
    let mut n = 0isize;
    while (n as usize) + 1 < sz {
        let c = libc::fgetc(fp);
        if c == libc::EOF {
            break;
        }
        *buf.offset(n) = c as c_char;
        n += 1;
        if c == b'\n' as c_int {
            break;
        }
    }
    n
}

// hts_sys's regenerated bindings use Rust native usize/isize for kgets_func2.
unsafe extern "C" fn fgetln_bridge_c(
    buf: *mut c_char,
    sz: usize,
    fp: *mut c_void,
) -> isize {
    let fp = fp.cast::<libc::FILE>();
    let mut n: isize = 0;
    while (n as usize) + 1 < sz {
        let c = libc::fgetc(fp);
        if c == libc::EOF {
            break;
        }
        *buf.offset(n) = c as c_char;
        n += 1;
        if c == b'\n' as c_int {
            break;
        }
    }
    n
}

#[test]
fn parity_kgetline2_synthetic_stream() {
    // kgetline2 uses an hgetln-like callback returning ssize_t.
    let contents: &[u8] = b"alpha\nbeta\ngamma\n";
    let path = write_tmp("kgetline2.txt", contents);

    let cpath = CString::new(path.to_str().unwrap()).unwrap();
    unsafe {
        // Native side
        let fp_n = libc::fopen(cpath.as_ptr(), c"r".as_ptr());
        assert!(!fp_n.is_null());
        let mut s_n: kstring_t = std::mem::zeroed();
        let mut lines_n: Vec<Vec<u8>> = Vec::new();
        loop {
            s_n.l = 0;
            if !s_n.s.is_null() {
                *s_n.s = 0;
            }
            let ret =
                htslib_rs::kstring::kgetline2(&mut s_n, Some(fgetln_bridge_native), fp_n.cast());
            if ret == libc::EOF {
                break;
            }
            lines_n.push(std::slice::from_raw_parts(s_n.s as *const u8, s_n.l).to_vec());
        }
        if !s_n.s.is_null() {
            libc::free(s_n.s.cast());
        }
        libc::fclose(fp_n);

        // C side
        let fp_c = libc::fopen(cpath.as_ptr(), c"r".as_ptr());
        assert!(!fp_c.is_null());
        let mut s_c: hts_sys::kstring_t = std::mem::zeroed();
        let mut lines_c: Vec<Vec<u8>> = Vec::new();
        loop {
            s_c.l = 0;
            if !s_c.s.is_null() {
                *s_c.s = 0;
            }
            let ret = hts_sys::kgetline2(&mut s_c, Some(fgetln_bridge_c), fp_c.cast());
            if ret == libc::EOF {
                break;
            }
            lines_c.push(std::slice::from_raw_parts(s_c.s as *const u8, s_c.l as usize).to_vec());
        }
        if !s_c.s.is_null() {
            libc::free(s_c.s.cast());
        }
        libc::fclose(fp_c);

        assert_eq!(lines_n, lines_c, "kgetline2 line set mismatch");
        assert_eq!(lines_n.len(), 3);
        assert_eq!(&lines_n[0], b"alpha");
        assert_eq!(&lines_n[1], b"beta");
        assert_eq!(&lines_n[2], b"gamma");
    }
    let _ = std::fs::remove_file(path);
}

// kfgetline is a thin Rust convenience over kgetline; no direct hts_sys peer.
// We smoke-test it natively to ensure the move didn't break it.
#[test]
fn native_kfgetline_smoke() {
    let contents: &[u8] = b"one\ntwo\nthree\n";
    let path = write_tmp("kfgetline.txt", contents);
    let cpath = CString::new(path.to_str().unwrap()).unwrap();
    unsafe {
        let fp = libc::fopen(cpath.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        let mut s: kstring_t = std::mem::zeroed();
        let mut lines: Vec<Vec<u8>> = Vec::new();
        loop {
            s.l = 0;
            if !s.s.is_null() {
                *s.s = 0;
            }
            let ret = htslib_rs::kstring::kfgetline(&mut s, fp.cast());
            if ret == libc::EOF {
                break;
            }
            lines.push(std::slice::from_raw_parts(s.s as *const u8, s.l).to_vec());
        }
        if !s.s.is_null() {
            libc::free(s.s.cast());
        }
        libc::fclose(fp);
        assert_eq!(lines, vec![b"one".to_vec(), b"two".to_vec(), b"three".to_vec()]);
    }
    let _ = std::fs::remove_file(path);
}
