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

// ---------------------------------------------------------------------------
// Manual externs into libhts for symbols not in hts-sys's bindgen output.
// These signatures mirror textutils_internal.h verbatim. This is a genuine
// external-C boundary, so the raw FFI is kept; c_char text is carried as u8
// and c_void opaque handles as *mut ().
// ---------------------------------------------------------------------------

extern "C" {
    fn hts_decode_percent(dest: *mut u8, destlen: *mut usize, s: *const u8) -> i32;

    fn hts_base64_decoded_length(len: usize) -> usize;

    fn hts_decode_base64(dest: *mut u8, destlen: *mut usize, s: *const u8) -> i32;

    fn hts_json_alloc_token() -> *mut ();
    fn hts_json_free_token(t: *mut ());
    fn hts_json_token_type(t: *mut ()) -> u8;
    fn hts_json_token_str(t: *mut ()) -> *mut u8;
    fn hts_json_snext(str_: *mut u8, state: *mut usize, token: *mut ()) -> u8;
    fn hts_json_sskip_value(str_: *mut u8, state: *mut usize, type_: u8) -> u8;

    fn hts_strprint(
        buf: *mut u8,
        buflen: usize,
        quote: u8,
        s: *const u8,
        len: usize,
    ) -> *const u8;
}

// ---------------------------------------------------------------------------
// hts_decode_percent
// ---------------------------------------------------------------------------

unsafe fn decode_percent_native(input: &[u8]) -> (Vec<u8>, usize) {
    let mut buf = vec![0u8; input.len() + 1];
    let mut len: usize = 0;
    let ret = htslib_rs::textutils::hts_decode_percent(&mut buf, &mut len, input);
    assert_eq!(ret, 0);
    let bytes = buf[..len].to_vec();
    (bytes, len)
}

unsafe fn decode_percent_c(input: &[u8]) -> (Vec<u8>, usize) {
    let mut cinput = input.to_vec();
    cinput.push(0);
    let mut buf = vec![0u8; input.len() + 1];
    let mut len: usize = 0;
    let ret = hts_decode_percent(buf.as_mut_ptr(), &mut len, cinput.as_ptr());
    assert_eq!(ret, 0);
    let bytes = buf[..len].to_vec();
    (bytes, len)
}

#[test]
fn parity_hts_decode_percent() {
    let cases: &[&[u8]] = &[
        b"hello%20world",
        b"%2Fpath%2Fto%2Ffile",
        b"no-escapes-here",
        b"",
        b"%41%42%43", // ABC
        b"mixed%20stuff%21here",
        b"%XYbad", // invalid escape — should be left as %XY
        b"%2",     // truncated escape
        b"a%00b",  // %00 produces an embedded NUL
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
    let cases: &[(&[u8], &[u8])] = &[
        (b"SGVsbG8=", b"Hello"),
        (b"SGVsbG8gV29ybGQ=", b"Hello World"),
        (b"YQ==", b"a"),
        (b"YWI=", b"ab"),
        (b"YWJj", b"abc"),
        (b"AAAA", &[0, 0, 0]),
        (b"", b""),
    ];
    unsafe {
        for (input, expected) in cases {
            let cap_n = htslib_rs::textutils::hts_base64_decoded_length(input.len()) + 8;
            let mut buf_n = vec![0u8; cap_n];
            let mut len_n: usize = 0;
            htslib_rs::textutils::hts_decode_base64(&mut buf_n, &mut len_n, input);

            let mut cinput = input.to_vec();
            cinput.push(0);
            let cap_c = hts_base64_decoded_length(input.len()) + 8;
            let mut buf_c = vec![0u8; cap_c];
            let mut len_c: usize = 0;
            hts_decode_base64(buf_c.as_mut_ptr(), &mut len_c, cinput.as_ptr());

            assert_eq!(
                len_n, len_c,
                "base64 decoded length mismatch for {:?}",
                input
            );
            let nb = &buf_n[..len_n];
            let cb = &buf_c[..len_c];
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

unsafe fn parse_json_native(json: &mut [u8]) -> Vec<(u8, Vec<u8>)> {
    let token = htslib_rs::textutils::hts_json_alloc_token();
    assert!(!token.is_null());
    let mut state: usize = 0;
    let mut tokens: Vec<(u8, Vec<u8>)> = Vec::new();
    loop {
        let t = htslib_rs::textutils::hts_json_snext(
            json.as_mut_ptr().cast(),
            &mut state,
            &mut *token,
        );
        if t == 0 {
            break;
        }
        let type_ = htslib_rs::textutils::hts_json_token_type(token) as u8;
        let str_ = htslib_rs::textutils::hts_json_token_str(&*token);
        let val = if !str_.is_null() {
            let mut n = 0usize;
            while *str_.add(n) != 0 {
                n += 1;
            }
            std::slice::from_raw_parts(str_.cast::<u8>(), n).to_vec()
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

unsafe fn parse_json_c(json: &mut [u8]) -> Vec<(u8, Vec<u8>)> {
    let token = hts_json_alloc_token();
    assert!(!token.is_null());
    let mut state: usize = 0;
    let mut tokens: Vec<(u8, Vec<u8>)> = Vec::new();
    loop {
        let t = hts_json_snext(json.as_mut_ptr(), &mut state, token);
        if t == 0 {
            break;
        }
        let type_ = hts_json_token_type(token);
        let str_ = hts_json_token_str(token);
        let val = if !str_.is_null() {
            let mut n = 0usize;
            while *str_.add(n) != 0 {
                n += 1;
            }
            std::slice::from_raw_parts(str_, n).to_vec()
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
                htslib_rs::textutils::hts_json_snext(a.as_mut_ptr().cast(), &mut state_n, &mut *tok_n);
            let after_n = htslib_rs::textutils::hts_json_sskip_value(
                a.as_mut_ptr().cast(),
                &mut state_n,
                t0_n,
            );
            htslib_rs::textutils::hts_json_free_token(tok_n);

            // C
            let tok_c = hts_json_alloc_token();
            let mut state_c: usize = 0;
            let t0_c = hts_json_snext(b.as_mut_ptr(), &mut state_c, tok_c);
            let after_c = hts_json_sskip_value(b.as_mut_ptr(), &mut state_c, t0_c);
            hts_json_free_token(tok_c);

            assert_eq!(t0_n as u8, t0_c, "initial token type mismatch for {:?}", snippet);
            assert_eq!(after_n as u8, after_c, "sskip_value mismatch for {:?}", snippet);
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

unsafe fn stringify_argv_native(args: &[&[u8]]) -> Vec<u8> {
    let owned: Vec<&[u8]> = args.to_vec();
    let bytes = htslib_rs::textutils::stringify_argv(&owned).expect("stringify_argv returned None");
    bytes
}

unsafe fn stringify_argv_c(args: &[&[u8]]) -> Vec<u8> {
    // Build NUL-terminated C strings for the genuine hts_sys C boundary.
    let owned: Vec<Vec<u8>> = args
        .iter()
        .map(|s| {
            let mut v = s.to_vec();
            v.push(0);
            v
        })
        .collect();
    let mut ptrs: Vec<*mut std::os::raw::c_char> =
        owned.iter().map(|s| s.as_ptr() as *mut std::os::raw::c_char).collect();
    let p = hts_sys::stringify_argv(ptrs.len() as std::os::raw::c_int, ptrs.as_mut_ptr());
    assert!(!p.is_null());
    let bytes = std::ffi::CStr::from_ptr(p).to_bytes().to_vec();
    libc::free(p.cast());
    bytes
}

#[test]
fn parity_stringify_argv() {
    let cases: &[Vec<&[u8]>] = &[
        vec![b"prog".as_slice()],
        vec![
            b"samtools".as_slice(),
            b"view".as_slice(),
            b"-h".as_slice(),
            b"in.bam".as_slice(),
        ],
        vec![
            b"a\tb".as_slice(),
            b"normal".as_slice(),
            b"with spaces".as_slice(),
        ], // tabs -> spaces
        vec![b"single".as_slice()],
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
                args.iter()
                    .map(|s| String::from_utf8_lossy(s))
                    .collect::<Vec<_>>()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// hts_strprint
// ---------------------------------------------------------------------------

unsafe fn strprint_native(quote: u8, s: &[u8], len: usize, buflen: usize) -> Vec<u8> {
    let mut buf = vec![0u8; buflen];
    let s_slice = &s[..len.min(s.len())];
    htslib_rs::textutils::hts_strprint(&mut buf, quote, s_slice);
    // Read up to first NUL
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    buf[..n].to_vec()
}

unsafe fn strprint_c(quote: u8, s: &[u8], len: usize, buflen: usize) -> Vec<u8> {
    let mut cs = s.to_vec();
    cs.push(0);
    let mut buf = vec![0u8; buflen];
    hts_strprint(buf.as_mut_ptr(), buflen, quote, cs.as_ptr(), len);
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    buf[..n].to_vec()
}

#[test]
fn parity_hts_strprint() {
    // (quote, content, len, buflen)
    let cases: &[(u8, &[u8], usize, usize)] = &[
        (b'"', b"hello world", usize::MAX, 64),
        (b'\'', b"a\tb\nc", usize::MAX, 64),
        (0, b"plain", usize::MAX, 64),
        (b'"', b"backslash\\here", usize::MAX, 64),
        (b'"', b"non-printable \x01 char", usize::MAX, 64),
        (b'"', b"unterminated and then some", 5, 64), // explicit len
        (
            b'"',
            b"tiny buf needs ellipsis maybe",
            usize::MAX,
            8,
        ),
        (0, b"", usize::MAX, 16),
    ];
    unsafe {
        for (q, s, l, b) in cases {
            let n = strprint_native(*q, s, *l, *b);
            let cc = strprint_c(*q, s, *l, *b);
            assert_eq!(
                n,
                cc,
                "hts_strprint mismatch for quote={} s={:?} len={} buflen={}",
                *q as char,
                String::from_utf8_lossy(s),
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
        let buf_len = buf.len();
        let rest = htslib_rs::textutils::sscan_string(std::slice::from_raw_parts_mut(
            buf.as_mut_ptr(),
            buf_len,
        ));
        assert!(rest.is_some());
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let parsed = &buf[..n];
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
        assert!(htslib_rs::textutils::hts_json_token_str(&*n).is_null());
        assert!(hts_json_token_str(c).is_null());
        htslib_rs::textutils::hts_json_free_token(n);
        hts_json_free_token(c);
    }
}
