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

use htslib_rs::{ks_tokaux_t, kstring_t};

// ---------------------------------------------------------------------------
// kputd
// ---------------------------------------------------------------------------

unsafe fn kputd_native(d: f64) -> (Vec<u8>, i32) {
    let mut s = kstring_t::default();
    let ret = htslib_rs::kstring::kputd(d, &mut s);
    let bytes = s.data.as_slice().to_vec();
    (bytes, ret)
}

unsafe fn kputd_c(d: f64) -> (Vec<u8>, i32) {
    let mut s: hts_sys::kstring_t = std::mem::zeroed();
    let ret = hts_sys::kputd(d, &mut s);
    let bytes = if s.s.is_null() {
        Vec::new()
    } else {
        std::slice::from_raw_parts(s.s as *const u8, s.l as usize).to_vec()
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
                n_bytes,
                c_bytes,
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

unsafe fn ksplit_core_native(input: &str, delim: i32) -> (i32, Vec<u8>, Vec<i32>) {
    // ksplit_core writes NUL bytes into the input buffer — work on a copy.
    let mut buf: Vec<u8> = input.bytes().chain(std::iter::once(0u8)).collect();
    let off_slice = htslib_rs::kstring::ksplit_core(&mut buf, delim, true).unwrap_or_default();
    let n = off_slice.len() as i32;
    (n, buf, off_slice)
}

unsafe fn ksplit_core_c(input: &str, delim: i32) -> (i32, Vec<u8>, Vec<i32>) {
    let mut buf: Vec<u8> = input.bytes().chain(std::iter::once(0u8)).collect();
    let mut max: i32 = 0;
    let mut offsets: *mut i32 = std::ptr::null_mut();
    let n = hts_sys::ksplit_core(
        buf.as_mut_ptr().cast(),
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
    let cases: &[(&str, i32)] = &[
        ("a\tb\tc", b'\t' as i32),
        ("a,b,c,d,e", b',' as i32),
        ("singleword", b'\t' as i32),
        ("", b'\t' as i32),
        // whitespace splitting (delimiter==0)
        ("  hello  world   foo bar  ", 0),
        ("a\tb\nc d", 0),
        ("a,,b,,c", b',' as i32), // empty fields between delimiters
        ("/path/to/file", b'/' as i32),
        ("one|two|three|four|five|six|seven", b'|' as i32),
        (":a:b:c:", b':' as i32),
    ];
    unsafe {
        for (input, delim) in cases {
            let (n_n, buf_n, off_n) = ksplit_core_native(input, *delim);
            let (n_c, buf_c, off_c) = ksplit_core_c(input, *delim);
            assert_eq!(n_n, n_c, "ksplit_core n mismatch for {input:?}/{delim}");
            assert_eq!(
                off_n, off_c,
                "ksplit_core offsets mismatch for {input:?}/{delim}"
            );
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
        let mut ks = kstring_t::default();
        ks.data.extend_from_slice(b"foo\tbar\tbaz");
        // ksplit returns owned offsets; it writes NULs into the buf.
        let mut n: i32 = 0;
        let off_slice = htslib_rs::kstring::ksplit(&mut ks, b'\t' as i32, &mut n).unwrap_or_default();
        assert_eq!(n, 3);

        let (n_c, _buf_c, off_c) = ksplit_core_c("foo\tbar\tbaz", b'\t' as i32);
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
unsafe fn run_kstrtok_native(input: &[u8], sep: &[u8]) -> Vec<(isize, i32)> {
    let mut aux: ks_tokaux_t = std::mem::zeroed();
    let mut results = Vec::new();
    // The native kstrtok scans for a trailing NUL on its continuation calls
    // (str_ = None), just like the C implementation. Give it the same
    // NUL-terminated buffer the C path uses so both observe an identical string.
    let input_c: Vec<u8> = input.iter().copied().chain(std::iter::once(0u8)).collect();
    let base = input_c.as_ptr();
    let mut p = htslib_rs::kstring::kstrtok(Some(&input_c[..input.len()]), Some(sep), &mut aux);
    while !p.is_null() {
        let start = p.offset_from(base);
        // aux.p points at the separator (or terminator) — distance to start is
        // the token length.
        let end = aux.p;
        let len = end.offset_from(p.cast::<i8>()) as i32;
        results.push((start, len));
        p = htslib_rs::kstring::kstrtok(None, None, &mut aux);
        if results.len() > 1024 {
            break; // safety
        }
    }
    results
}

unsafe fn run_kstrtok_c(input: &[u8], sep: &[u8]) -> Vec<(isize, i32)> {
    let mut aux: hts_sys::ks_tokaux_t = std::mem::zeroed();
    let mut results = Vec::new();
    // hts_sys::kstrtok wants NUL-terminated C strings.
    let input_c: Vec<u8> = input.iter().copied().chain(std::iter::once(0u8)).collect();
    let sep_c: Vec<u8> = sep.iter().copied().chain(std::iter::once(0u8)).collect();
    let base = input_c.as_ptr().cast();
    let mut p = hts_sys::kstrtok(base, sep_c.as_ptr().cast(), &mut aux);
    while !p.is_null() {
        let start = p.offset_from(base);
        let end = aux.p;
        let len = end.offset_from(p) as i32;
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
    let cases: &[(&[u8], &[u8])] = &[
        (b"a,b,c,d", b","),
        (b"hello world foo bar", b" "),
        (b"a;b,c;d,e", b";,"),
        (b"singleton", b","),
        (b"", b","),
        (b",leading,trailing,", b","),
        (b"path/to/file", b"/"),
        (b"a\tb c\nd", b"\t \n"),
    ];
    unsafe {
        for (input, sep) in cases {
            let n = run_kstrtok_native(input, sep);
            let c = run_kstrtok_c(input, sep);
            assert_eq!(
                n,
                c,
                "kstrtok mismatch for input={:?} sep={:?}",
                String::from_utf8_lossy(input),
                String::from_utf8_lossy(sep)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// kmemmem, kstrstr, kstrnstr (substring search)
// ---------------------------------------------------------------------------

#[test]
fn parity_kstrstr_substring_search() {
    let cases: &[(&[u8], &[u8])] = &[
        (b"hello world", b"world"),
        (b"hello world", b"hello"),
        (b"hello world", b"o w"),
        (b"hello world", b"xyz"),
        (b"", b"foo"),
        (b"abcabcabc", b"abc"),
        (b"abcabcabc", b"bca"),
        (b"banana", b"ana"),
        (b"a", b""),
        // long haystack
        (
            b"the quick brown fox jumps over the lazy dog the quick brown fox",
            b"fox",
        ),
    ];
    unsafe {
        for (haystack, needle) in cases {
            let mut prep_n: Option<Box<[i32]>> = None;
            let mut prep_c: *mut i32 = std::ptr::null_mut();
            // hts_sys::kstrstr wants NUL-terminated C strings.
            let haystack_c: Vec<u8> = haystack.iter().copied().chain(std::iter::once(0u8)).collect();
            let needle_c: Vec<u8> = needle.iter().copied().chain(std::iter::once(0u8)).collect();
            let n_res = htslib_rs::kstring::kstrstr(
                haystack,
                needle,
                Some(&mut prep_n),
            );
            let cptr = hts_sys::kstrstr(haystack_c.as_ptr().cast(), needle_c.as_ptr().cast(), &mut prep_c);
            let n_off = match n_res {
                None => -1,
                Some(off) => off as isize,
            };
            let c_off = if cptr.is_null() {
                -1
            } else {
                cptr.offset_from(haystack_c.as_ptr().cast())
            };
            assert_eq!(
                n_off,
                c_off,
                "kstrstr offset mismatch for haystack={:?} needle={:?}",
                String::from_utf8_lossy(haystack),
                String::from_utf8_lossy(needle)
            );
            if !prep_c.is_null() {
                libc::free(prep_c.cast());
            }
        }
    }
}

#[test]
fn parity_kstrnstr_bounded_search() {
    // (haystack, needle, n)
    let cases: &[(&[u8], &[u8], i32)] = &[
        (b"hello world", b"world", 11),
        (b"hello world", b"world", 7), // not found within n
        (b"hello world", b"hello", 5),
        (b"hello world", b"hello", 4), // pat longer than n
        (b"abcabcabc", b"abc", 9),
        (b"abcabcabc", b"abc", 2),
        (b"foo", b"", 3),    // empty pattern -> haystack
        (b"foo", b"bar", 0), // n<=0
    ];
    unsafe {
        for (haystack, needle, n) in cases {
            let mut prep_n: Option<Box<[i32]>> = None;
            let mut prep_c: *mut i32 = std::ptr::null_mut();
            // hts_sys::kstrnstr wants NUL-terminated C strings.
            let haystack_c: Vec<u8> = haystack.iter().copied().chain(std::iter::once(0u8)).collect();
            let needle_c: Vec<u8> = needle.iter().copied().chain(std::iter::once(0u8)).collect();
            let n_res = htslib_rs::kstring::kstrnstr(
                haystack,
                needle,
                if *n >= 0 { *n as usize } else { 0 },
                Some(&mut prep_n),
            );
            let cptr = hts_sys::kstrnstr(haystack_c.as_ptr().cast(), needle_c.as_ptr().cast(), *n, &mut prep_c);
            let n_off = match n_res {
                None => -1,
                Some(off) => off as isize,
            };
            let c_off = if cptr.is_null() {
                -1
            } else {
                cptr.offset_from(haystack_c.as_ptr().cast())
            };
            assert_eq!(
                n_off,
                c_off,
                "kstrnstr offset mismatch for haystack={:?} needle={:?} n={}",
                String::from_utf8_lossy(haystack),
                String::from_utf8_lossy(needle),
                n
            );
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
            let mut prep_n: Option<Box<[i32]>> = None;
            let mut prep_c: *mut i32 = std::ptr::null_mut();
            let n_res = htslib_rs::kstring::kmemmem(haystack, needle, Some(&mut prep_n));
            let cptr = hts_sys::kmemmem(
                haystack.as_ptr().cast(),
                haystack.len() as i32,
                needle.as_ptr().cast(),
                needle.len() as i32,
                &mut prep_c,
            );
            let n_off = match n_res {
                None => -1,
                Some(off) => off as isize,
            };
            let c_off = if cptr.is_null() {
                -1
            } else {
                (cptr as *const u8).offset_from(haystack.as_ptr())
            };
            assert_eq!(
                n_off,
                c_off,
                "kmemmem offset mismatch for haystack_len={} needle={:?}",
                haystack.len(),
                needle
            );
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
    let cpath: Vec<u8> = file_path
        .to_str()
        .unwrap()
        .bytes()
        .chain(std::iter::once(0u8))
        .collect();
    let mode = b"r\0";
    let fp = libc::fopen(cpath.as_ptr().cast(), mode.as_ptr().cast());
    assert!(!fp.is_null());

    let mut s = kstring_t::default();
    let mut lines = Vec::new();
    loop {
        s.data.clear();
        let ret = htslib_rs::kstring::kgetline(
            &mut s,
            Some(htslib_rs::kstring::fgets_wrapper),
            fp.cast::<()>(),
        );
        if ret == libc::EOF {
            break;
        }
        let bytes = s.data.as_slice().to_vec();
        lines.push(bytes);
    }
    libc::fclose(fp);
    lines
}

unsafe fn read_all_with_kgetline_c(file_path: &std::path::Path) -> Vec<Vec<u8>> {
    let cpath: Vec<u8> = file_path
        .to_str()
        .unwrap()
        .bytes()
        .chain(std::iter::once(0u8))
        .collect();
    let mode = b"r\0";
    let fp = libc::fopen(cpath.as_ptr().cast(), mode.as_ptr().cast());
    assert!(!fp.is_null());

    // Bridge: hts_sys's kgetline wants a kgets_func (extern "C" fn(*mut c_char, c_int, *mut c_void) -> *mut c_char).
    // This is a genuine external-C callback boundary, so it keeps the C ABI types.
    // libc::fgets has that signature too — match types.
    unsafe extern "C" fn fgets_bridge(
        buf: *mut std::ffi::c_char,
        n: std::ffi::c_int,
        fp: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_char {
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

// On Linux64: usize = u64 on both sides; isize = i64 on both sides. So the
// callbacks below are ABI-compatible across both sides (both reduce to
// extern "C" fn(*mut c_char, u64, *mut c_void) -> i64).

// Genuine extern "C" callback boundary for kgets_func2 — keeps the C ABI types.
unsafe extern "C" fn fgetln_bridge_native(
    buf: *mut std::ffi::c_char,
    sz: usize,
    fp: *mut std::ffi::c_void,
) -> isize {
    let fp = fp.cast::<libc::FILE>();
    let mut n = 0isize;
    while (n as usize) + 1 < sz {
        let c = libc::fgetc(fp);
        if c == libc::EOF {
            break;
        }
        *buf.offset(n) = c as std::ffi::c_char;
        n += 1;
        if c == b'\n' as std::ffi::c_int {
            break;
        }
    }
    n
}

// hts_sys's regenerated bindings use Rust native usize/isize for kgets_func2.
unsafe extern "C" fn fgetln_bridge_c(
    buf: *mut std::ffi::c_char,
    sz: usize,
    fp: *mut std::ffi::c_void,
) -> isize {
    let fp = fp.cast::<libc::FILE>();
    let mut n: isize = 0;
    while (n as usize) + 1 < sz {
        let c = libc::fgetc(fp);
        if c == libc::EOF {
            break;
        }
        *buf.offset(n) = c as std::ffi::c_char;
        n += 1;
        if c == b'\n' as std::ffi::c_int {
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

    let cpath: Vec<u8> = path
        .to_str()
        .unwrap()
        .bytes()
        .chain(std::iter::once(0u8))
        .collect();
    unsafe {
        // Native side
        let fp_n = libc::fopen(cpath.as_ptr().cast(), b"r\0".as_ptr().cast());
        assert!(!fp_n.is_null());
        let mut s_n = kstring_t::default();
        let mut lines_n: Vec<Vec<u8>> = Vec::new();
        loop {
            s_n.data.clear();
            let ret = htslib_rs::kstring::kgetline2(
                &mut s_n,
                Some(fgetln_bridge_native),
                fp_n.cast::<()>(),
            );
            if ret == libc::EOF {
                break;
            }
            lines_n.push(s_n.data.as_slice().to_vec());
        }
        libc::fclose(fp_n);

        // C side
        let fp_c = libc::fopen(cpath.as_ptr().cast(), b"r\0".as_ptr().cast());
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
    let cpath: Vec<u8> = path
        .to_str()
        .unwrap()
        .bytes()
        .chain(std::iter::once(0u8))
        .collect();
    unsafe {
        let fp = libc::fopen(cpath.as_ptr().cast(), b"r\0".as_ptr().cast());
        assert!(!fp.is_null());
        let mut s = kstring_t::default();
        let mut lines: Vec<Vec<u8>> = Vec::new();
        loop {
            s.data.clear();
            let ret = htslib_rs::kstring::kfgetline(&mut s, fp.cast::<libc::FILE>());
            if ret == libc::EOF {
                break;
            }
            lines.push(s.data.as_slice().to_vec());
        }
        libc::fclose(fp);
        assert_eq!(
            lines,
            vec![b"one".to_vec(), b"two".to_vec(), b"three".to_vec()]
        );
    }
    let _ = std::fs::remove_file(path);
}
