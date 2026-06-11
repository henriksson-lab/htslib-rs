// Parity tests for functions recently MOVED from src/hts.rs into src/textutils.rs.
//
// Functions covered (all originally from htslib/textutils.c):
//   - hts_decode_percent
//   - hts_base64_decoded_length
//   - hts_decode_base64
//   - hts_json_alloc_token
//   - hts_json_token_type
//   - hts_json_free_token
//   - hts_json_token_str
//   - hts_json_snext
//   - hts_json_sskip_value
//   - stringify_argv             (exposed via sam.h -> in hts_sys bindings)
//   - hts_strprint
//
// Note: most textutils functions live in `textutils_internal.h`, which is NOT
// included by hts-sys's wrapper.h, so hts-sys does not auto-generate bindings
// for them. They DO have external C linkage in libhts.a though, so we declare
// our own `extern "C"` block to call them directly for parity. (hts-sys is
// linked in the test build because `htslib_rs` is a dev-dep of itself via
// `cfg(feature = "parity")` and the build pulls libhts.a in.)
//
// sscan_string is `static` in textutils.c, so it has no external linkage at
// all — that one we cover only with a native smoke test and #[ignore] the
// parity comparison.

#![cfg(feature = "parity")]

use htslib_rs::{hts_json_token, kstring_t};
use std::ffi::{c_char, c_int, c_void, CStr};

// ---------------------------------------------------------------------------
// Manual externs into libhts for symbols not in hts-sys's bindgen output.
// These signatures mirror textutils_internal.h verbatim.
// ---------------------------------------------------------------------------

extern "C" {
    fn hts_decode_percent(dest: *mut c_char, destlen: *mut usize, s: *const c_char) -> c_int;

    fn hts_base64_decoded_length(len: usize) -> usize;

    fn hts_decode_base64(dest: *mut c_char, destlen: *mut usize, s: *const c_char) -> c_int;

    fn hts_json_alloc_token() -> *mut c_void;
    fn hts_json_free_token(t: *mut c_void);
    fn hts_json_token_type(t: *mut c_void) -> c_char;
    fn hts_json_token_str(t: *mut c_void) -> *mut c_char;
    fn hts_json_snext(str_: *mut c_char, state: *mut usize, token: *mut c_void) -> c_char;
    fn hts_json_sskip_value(str_: *mut c_char, state: *mut usize, type_: c_char) -> c_char;

    fn hts_strprint(
        buf: *mut c_char,
        buflen: usize,
        quote: c_char,
        s: *const c_char,
        len: usize,
    ) -> *const c_char;
}

// ---------------------------------------------------------------------------
// hts_decode_percent
// ---------------------------------------------------------------------------

unsafe fn decode_percent_native(input: &CStr) -> (Vec<u8>, usize) {
    let mut buf = vec![0i8; input.to_bytes().len() + 1];
    let mut len: usize = 0;
    let ret = htslib_rs::textutils::hts_decode_percent(buf.as_mut_ptr(), &mut len, input.as_ptr());
    assert_eq!(ret, 0);
    let bytes = std::slice::from_raw_parts(buf.as_ptr() as *const u8, len).to_vec();
    (bytes, len)
}

unsafe fn decode_percent_c(input: &CStr) -> (Vec<u8>, usize) {
    let mut buf = vec![0i8; input.to_bytes().len() + 1];
    let mut len: usize = 0;
    let ret = hts_decode_percent(buf.as_mut_ptr(), &mut len, input.as_ptr());
    assert_eq!(ret, 0);
    let bytes = std::slice::from_raw_parts(buf.as_ptr() as *const u8, len).to_vec();
    (bytes, len)
}

#[test]
fn parity_hts_decode_percent() {
    let cases: &[&CStr] = &[
        c"hello%20world",
        c"%2Fpath%2Fto%2Ffile",
        c"no-escapes-here",
        c"",
        c"%41%42%43", // ABC
        c"mixed%20stuff%21here",
        c"%XYbad", // invalid escape — should be left as %XY
        c"%2",     // truncated escape
        c"a%00b",  // %00 produces an embedded NUL
    ];
    unsafe {
        for input in cases {
            let (n, n_len) = decode_percent_native(input);
            let (c, c_len) = decode_percent_c(input);
            assert_eq!(n_len, c_len, "decode_percent len mismatch for {:?}", input);
            assert_eq!(n, c, "decode_percent bytes mismatch for {:?}", input);
        }
    }
}

// ---------------------------------------------------------------------------
// hts_base64_decoded_length + hts_decode_base64 round-trip
// ---------------------------------------------------------------------------

#[test]
fn parity_hts_base64_decoded_length() {
    let lens: &[usize] = &[0, 1, 2, 3, 4, 5, 8, 100, 1024, 65536];
    unsafe {
        for &len in lens {
            let n = htslib_rs::textutils::hts_base64_decoded_length(len);
            let c = hts_base64_decoded_length(len);
            assert_eq!(n, c, "base64_decoded_length mismatch for {len}");
        }
    }
}

#[test]
fn parity_hts_decode_base64() {
    // standard base64 strings
    let cases: &[(&CStr, &[u8])] = &[
        (c"SGVsbG8=", b"Hello"),
        (c"SGVsbG8gV29ybGQ=", b"Hello World"),
        (c"YQ==", b"a"),
        (c"YWI=", b"ab"),
        (c"YWJj", b"abc"),
        (c"AAAA", &[0, 0, 0]),
        (c"", b""),
    ];
    unsafe {
        for (input, expected) in cases {
            let cap_n = htslib_rs::textutils::hts_base64_decoded_length(input.to_bytes().len()) + 8;
            let mut buf_n = vec![0i8; cap_n];
            let mut len_n: usize = 0;
            htslib_rs::textutils::hts_decode_base64(buf_n.as_mut_ptr(), &mut len_n, input.as_ptr());

            let cap_c = hts_base64_decoded_length(input.to_bytes().len()) + 8;
            let mut buf_c = vec![0i8; cap_c];
            let mut len_c: usize = 0;
            hts_decode_base64(buf_c.as_mut_ptr(), &mut len_c, input.as_ptr());

            assert_eq!(
                len_n, len_c,
                "base64 decoded length mismatch for {:?}",
                input
            );
            let nb = std::slice::from_raw_parts(buf_n.as_ptr() as *const u8, len_n);
            let cb = std::slice::from_raw_parts(buf_c.as_ptr() as *const u8, len_c);
            assert_eq!(nb, cb, "base64 bytes mismatch for {:?}", input);
            if !expected.is_empty() {
                assert_eq!(nb, *expected, "base64 wrong decode for {:?}", input);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// hts_json_* (parsing JSON via the s-variant API)
// ---------------------------------------------------------------------------

unsafe fn parse_json_native(json: &mut [u8]) -> Vec<(c_char, Vec<u8>)> {
    let token = htslib_rs::textutils::hts_json_alloc_token();
    assert!(!token.is_null());
    let mut state: usize = 0;
    let mut tokens: Vec<(c_char, Vec<u8>)> = Vec::new();
    loop {
        let t = htslib_rs::textutils::hts_json_snext(
            json.as_mut_ptr().cast::<c_char>(),
            &mut state,
            token,
        );
        if t == 0 {
            break;
        }
        let type_ = htslib_rs::textutils::hts_json_token_type(token);
        let str_ = htslib_rs::textutils::hts_json_token_str(token);
        let val = if !str_.is_null() {
            CStr::from_ptr(str_).to_bytes().to_vec()
        } else {
            Vec::new()
        };
        tokens.push((type_, val));
        if tokens.len() > 4096 {
            break;
        }
    }
    htslib_rs::textutils::hts_json_free_token(token);
    tokens
}

unsafe fn parse_json_c(json: &mut [u8]) -> Vec<(c_char, Vec<u8>)> {
    let token = hts_json_alloc_token();
    assert!(!token.is_null());
    let mut state: usize = 0;
    let mut tokens: Vec<(c_char, Vec<u8>)> = Vec::new();
    loop {
        let t = hts_json_snext(json.as_mut_ptr().cast::<c_char>(), &mut state, token);
        if t == 0 {
            break;
        }
        let type_ = hts_json_token_type(token);
        let str_ = hts_json_token_str(token);
        let val = if !str_.is_null() {
            CStr::from_ptr(str_).to_bytes().to_vec()
        } else {
            Vec::new()
        };
        tokens.push((type_, val));
        if tokens.len() > 4096 {
            break;
        }
    }
    hts_json_free_token(token);
    tokens
}

#[test]
fn parity_hts_json_simple_snippets() {
    // Inputs need to be mutable (the JSON parser modifies them in place).
    let snippets: &[&[u8]] = &[
        b"{\"a\": 1, \"b\": \"hello\"}\0",
        b"[1, 2, 3, 4]\0",
        b"true\0",
        b"false\0",
        b"null\0",
        b"42\0",
        b"\"plain string\"\0",
        b"{}\0",
        b"[]\0",
        b"{\"nested\": {\"x\": [1, 2, 3]}}\0",
        b"\"with \\\"escaped\\\" quotes\"\0",
    ];
    unsafe {
        for snippet in snippets {
            let mut a: Vec<u8> = snippet.to_vec();
            let mut b: Vec<u8> = snippet.to_vec();
            let n = parse_json_native(&mut a);
            let c = parse_json_c(&mut b);
            assert_eq!(
                n,
                c,
                "hts_json_snext token stream mismatch for {:?}",
                std::str::from_utf8(snippet).unwrap_or("<bin>")
            );
        }
    }
}

#[test]
fn parity_hts_json_sskip_value() {
    // The s-variant of skip_value walks past a top-level value. We compare
    // the post-skip state offset and the next token type.
    let inputs: &[&[u8]] = &[
        b"[1, 2, 3] \"trailing\"\0",
        b"{\"x\": 1, \"y\": 2} 42\0",
        b"\"string\" 99\0",
    ];
    unsafe {
        for snippet in inputs {
            let mut a = snippet.to_vec();
            let mut b = snippet.to_vec();

            // Native
            let tok_n = htslib_rs::textutils::hts_json_alloc_token();
            let mut state_n: usize = 0;
            // advance to the first token to set up a type
            let t0_n =
                htslib_rs::textutils::hts_json_snext(a.as_mut_ptr().cast(), &mut state_n, tok_n);
            let after_n = htslib_rs::textutils::hts_json_sskip_value(
                a.as_mut_ptr().cast(),
                &mut state_n,
                t0_n,
            );
            htslib_rs::textutils::hts_json_free_token(tok_n);

            // C
            let tok_c = hts_json_alloc_token();
            let mut state_c: usize = 0;
            let t0_c = hts_json_snext(b.as_mut_ptr().cast(), &mut state_c, tok_c);
            let after_c = hts_json_sskip_value(b.as_mut_ptr().cast(), &mut state_c, t0_c);
            hts_json_free_token(tok_c);

            assert_eq!(t0_n, t0_c, "initial token type mismatch for {:?}", snippet);
            assert_eq!(after_n, after_c, "sskip_value mismatch for {:?}", snippet);
            assert_eq!(
                state_n, state_c,
                "post-skip state mismatch for {:?}",
                snippet
            );
        }
    }
}

// ---------------------------------------------------------------------------
// stringify_argv
// ---------------------------------------------------------------------------

unsafe fn stringify_argv_native(args: &[&CStr]) -> Vec<u8> {
    let owned: Vec<&[u8]> = args.iter().map(|s| s.to_bytes()).collect();
    let bytes = htslib_rs::textutils::stringify_argv(&owned).expect("stringify_argv returned None");
    bytes
}

unsafe fn stringify_argv_c(args: &[&CStr]) -> Vec<u8> {
    let mut ptrs: Vec<*mut c_char> = args.iter().map(|s| s.as_ptr() as *mut c_char).collect();
    let p = hts_sys::stringify_argv(ptrs.len() as c_int, ptrs.as_mut_ptr());
    assert!(!p.is_null());
    let bytes = CStr::from_ptr(p).to_bytes().to_vec();
    libc::free(p.cast());
    bytes
}

#[test]
fn parity_stringify_argv() {
    let cases: &[Vec<&CStr>] = &[
        vec![c"prog"],
        vec![c"samtools", c"view", c"-h", c"in.bam"],
        vec![c"a\tb", c"normal", c"with spaces"], // tabs -> spaces
        vec![c"single"],
        vec![],
    ];
    unsafe {
        for args in cases {
            let n = stringify_argv_native(args);
            let c = stringify_argv_c(args);
            assert_eq!(
                n,
                c,
                "stringify_argv mismatch for args={:?}",
                args.iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// hts_strprint
// ---------------------------------------------------------------------------

unsafe fn strprint_native(quote: c_char, s: &CStr, len: usize, buflen: usize) -> Vec<u8> {
    let mut buf = vec![0i8; buflen];
    htslib_rs::textutils::hts_strprint(buf.as_mut_ptr(), buflen, quote, s.as_ptr(), len);
    // Read up to first NUL
    let s = CStr::from_ptr(buf.as_ptr()).to_bytes().to_vec();
    s
}

unsafe fn strprint_c(quote: c_char, s: &CStr, len: usize, buflen: usize) -> Vec<u8> {
    let mut buf = vec![0i8; buflen];
    hts_strprint(buf.as_mut_ptr(), buflen, quote, s.as_ptr(), len);
    let s = CStr::from_ptr(buf.as_ptr()).to_bytes().to_vec();
    s
}

#[test]
fn parity_hts_strprint() {
    // (quote, content, len, buflen)
    let cases: &[(c_char, &CStr, usize, usize)] = &[
        (b'"' as c_char, c"hello world", usize::MAX, 64),
        (b'\'' as c_char, c"a\tb\nc", usize::MAX, 64),
        (0, c"plain", usize::MAX, 64),
        (b'"' as c_char, c"backslash\\here", usize::MAX, 64),
        (b'"' as c_char, c"non-printable \x01 char", usize::MAX, 64),
        (b'"' as c_char, c"unterminated and then some", 5, 64), // explicit len
        (
            b'"' as c_char,
            c"tiny buf needs ellipsis maybe",
            usize::MAX,
            8,
        ),
        (0, c"", usize::MAX, 16),
    ];
    unsafe {
        for (q, s, l, b) in cases {
            let n = strprint_native(*q, s, *l, *b);
            let cc = strprint_c(*q, s, *l, *b);
            assert_eq!(
                n,
                cc,
                "hts_strprint mismatch for quote={} s={:?} len={} buflen={}",
                *q as u8 as char,
                s.to_string_lossy(),
                l,
                b
            );
        }
    }
}

#[test]
fn native_sscan_string_smoke() {
    // Native smoke test only (verifies our translation still works after the
    // move from src/hts.rs to src/textutils.rs).
    let mut buf = b"hello\\nworld\"trail\0".to_vec();
    unsafe {
        let rest = htslib_rs::textutils::sscan_string(buf.as_mut_ptr().cast::<c_char>());
        assert!(!rest.is_null());
        let parsed = CStr::from_ptr(buf.as_ptr().cast()).to_bytes();
        assert_eq!(parsed, b"hello\nworld");
    }
    // Quiet unused-import warning if any
    let _ = std::mem::size_of::<kstring_t>();
    let _ = std::mem::size_of::<hts_json_token>();
}

// ---------------------------------------------------------------------------
// hts_json_alloc_token / hts_json_token_type / hts_json_free_token /
// hts_json_token_str  — already exercised indirectly above in the snext tests.
// Add a direct alloc/free sanity comparison.
// ---------------------------------------------------------------------------

#[test]
fn parity_hts_json_alloc_and_free() {
    unsafe {
        let n = htslib_rs::textutils::hts_json_alloc_token();
        let c = hts_json_alloc_token();
        assert!(!n.is_null());
        assert!(!c.is_null());
        // Freshly allocated tokens have type 0 and str = NULL (calloc zeros).
        assert_eq!(htslib_rs::textutils::hts_json_token_type(n), 0);
        assert_eq!(hts_json_token_type(c), 0);
        assert!(htslib_rs::textutils::hts_json_token_str(n).is_null());
        assert!(hts_json_token_str(c).is_null());
        htslib_rs::textutils::hts_json_free_token(n);
        hts_json_free_token(c);
    }
}
