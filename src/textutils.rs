use crate::htslib_rs::hts::{hFILE, hts_json_token, isprint_c, kstring_t, size_t};
use std::ffi::{c_char, c_int, c_void};

type hts_json_nextfn =
    unsafe fn(arg1: *mut c_void, arg2: *mut c_void, token: *mut hts_json_token) -> c_char;

// original: dehex (htslib/textutils.c:37)
unsafe fn dehex(c: c_char) -> c_int {
    let c = c as u8;
    if (b'a'..=b'f').contains(&c) {
        (c - b'a' + 10) as c_int
    } else if (b'A'..=b'F').contains(&c) {
        (c - b'A' + 10) as c_int
    } else if c.is_ascii_digit() {
        (c - b'0') as c_int
    } else {
        -1
    }
}

// original: encode_utf8 (htslib/textutils.c:103)
unsafe fn encode_utf8(mut s: *mut c_char, x: u32) -> *mut c_char {
    if x >= 0x10000 {
        *s = (0xf0 | (x >> 18)) as c_char;
        s = s.add(1);
        *s = (0x80 | ((x >> 12) & 0x3f)) as c_char;
        s = s.add(1);
        *s = (0x80 | ((x >> 6) & 0x3f)) as c_char;
        s = s.add(1);
        *s = (0x80 | (x & 0x3f)) as c_char;
        s = s.add(1);
    } else if x >= 0x800 {
        *s = (0xe0 | (x >> 12)) as c_char;
        s = s.add(1);
        *s = (0x80 | ((x >> 6) & 0x3f)) as c_char;
        s = s.add(1);
        *s = (0x80 | (x & 0x3f)) as c_char;
        s = s.add(1);
    } else if x >= 0x80 {
        *s = (0xc0 | (x >> 6)) as c_char;
        s = s.add(1);
        *s = (0x80 | (x & 0x3f)) as c_char;
        s = s.add(1);
    } else {
        *s = x as c_char;
        s = s.add(1);
    }

    s
}

// original: fscan_string (htslib/textutils.c:164)
pub unsafe fn textutils_c_164_fscan_string(fp: *mut hFILE, d: *mut kstring_t) -> c_int {
    let mut e: u32 = 0;

    loop {
        let mut c = crate::htslib_rs::hfile::htslib_hfile_h_163_hgetc(fp);
        if c == libc::EOF {
            break;
        }

        match c as u8 {
            b'\\' => {
                c = crate::htslib_rs::hfile::htslib_hfile_h_163_hgetc(fp);
                if c == libc::EOF {
                    return if e == 0 { 0 } else { -1 };
                }

                match c as u8 {
                    b'b' => e |= (crate::htslib_rs::hts::kputc(b'\x08' as c_int, d) < 0) as u32,
                    b'f' => e |= (crate::htslib_rs::hts::kputc(b'\x0c' as c_int, d) < 0) as u32,
                    b'n' => e |= (crate::htslib_rs::hts::kputc(b'\n' as c_int, d) < 0) as u32,
                    b'r' => e |= (crate::htslib_rs::hts::kputc(b'\r' as c_int, d) < 0) as u32,
                    b't' => e |= (crate::htslib_rs::hts::kputc(b'\t' as c_int, d) < 0) as u32,
                    b'u' => {
                        c = crate::htslib_rs::hfile::htslib_hfile_h_163_hgetc(fp);
                        if c != libc::EOF {
                            let d1 = dehex(c as c_char);
                            if d1 >= 0 {
                                c = crate::htslib_rs::hfile::htslib_hfile_h_163_hgetc(fp);
                                if c != libc::EOF {
                                    let d2 = dehex(c as c_char);
                                    if d2 >= 0 {
                                        c = crate::htslib_rs::hfile::htslib_hfile_h_163_hgetc(fp);
                                        if c != libc::EOF {
                                            let d3 = dehex(c as c_char);
                                            if d3 >= 0 {
                                                c = crate::htslib_rs::hfile::htslib_hfile_h_163_hgetc(fp);
                                                if c != libc::EOF {
                                                    let d4 = dehex(c as c_char);
                                                    if d4 >= 0 {
                                                        let mut buf = [0 as c_char; 8];
                                                        let lim = encode_utf8(
                                                            buf.as_mut_ptr(),
                                                            ((d1 << 12)
                                                                | (d2 << 8)
                                                                | (d3 << 4)
                                                                | d4)
                                                                as u32,
                                                        );
                                                        e |= (crate::htslib_rs::hts::kputsn(
                                                            buf.as_ptr(),
                                                            lim.offset_from(buf.as_ptr()) as usize,
                                                            d,
                                                        ) < 0)
                                                            as u32;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => e |= (crate::htslib_rs::hts::kputc(c, d) < 0) as u32,
                }
            }
            b'"' => return if e == 0 { 0 } else { -1 },
            _ => e |= (crate::htslib_rs::hts::kputc(c, d) < 0) as u32,
        }
    }

    if e == 0 {
        0
    } else {
        -1
    }
}

// original: token_type (htslib/textutils.c:202)
unsafe fn token_type(token: *mut hts_json_token) -> c_char {
    let s = (*token).str_;

    match *s as u8 {
        b'f' => {
            if libc::strcmp(s, c"false".as_ptr()) == 0 {
                b'b' as c_char
            } else {
                b'?' as c_char
            }
        }
        b'n' => {
            if libc::strcmp(s, c"null".as_ptr()) == 0 {
                b'.' as c_char
            } else {
                b'?' as c_char
            }
        }
        b't' => {
            if libc::strcmp(s, c"true".as_ptr()) == 0 {
                b'b' as c_char
            } else {
                b'?' as c_char
            }
        }
        b'-' | b'0'..=b'9' => b'n' as c_char,
        _ => b'?' as c_char,
    }
}

// original: hts_json_fnext (htslib/textutils.c:292)
pub unsafe fn textutils_c_292_hts_json_fnext(
    fp: *mut hFILE,
    token: *mut hts_json_token,
    kstr: *mut kstring_t,
) -> c_char {
    loop {
        let mut c = crate::htslib_rs::hfile::htslib_hfile_h_163_hgetc(fp);

        match c {
            x if matches!(x as u8, b' ' | b'\t' | b'\r' | b'\n' | b',' | b':') => continue,
            x if x == libc::EOF => {
                (*token).type_ = 0;
                return (*token).type_;
            }
            x if matches!(x as u8, b'{' | b'[' | b'}' | b']') => {
                (*token).type_ = x as c_char;
                return (*token).type_;
            }
            x if x == b'"' as c_int => {
                (*kstr).l = 0;
                textutils_c_164_fscan_string(fp, kstr);
                if (*kstr).l == 0 {
                    crate::htslib_rs::hts::kputsn(c"".as_ptr(), 0, kstr);
                }
                (*token).str_ = (*kstr).s;
                (*token).type_ = b's' as c_char;
                return (*token).type_;
            }
            _ => {
                let mut peek = 0 as c_char;

                (*kstr).l = 0;
                crate::htslib_rs::hts::kputc(c, kstr);
                while crate::htslib_rs::hfile::hpeek(fp, (&mut peek as *mut c_char).cast(), 1) == 1
                    && !matches!(
                        peek as u8,
                        b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}'
                    )
                {
                    c = crate::htslib_rs::hfile::htslib_hfile_h_163_hgetc(fp);
                    if c == libc::EOF {
                        break;
                    }
                    crate::htslib_rs::hts::kputc(c, kstr);
                }
                (*token).str_ = (*kstr).s;
                (*token).type_ = token_type(token);
                return (*token).type_;
            }
        }
    }
}

// original: skip_value (htslib/textutils.c:338)
unsafe fn skip_value(
    type_: c_char,
    next: hts_json_nextfn,
    arg1: *mut c_void,
    arg2: *mut c_void,
) -> c_char {
    let mut token = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };

    let first = if type_ != 0 {
        type_
    } else {
        next(arg1, arg2, &mut token)
    };

    let mut level;
    match first as u8 {
        0 => return 0,
        b'?' | b'}' | b']' => return b'?' as c_char,
        b'{' | b'[' => level = 1,
        _ => return b'v' as c_char,
    }

    while level > 0 {
        match next(arg1, arg2, &mut token) as u8 {
            0 => return 0,
            b'?' => return b'?' as c_char,
            b'{' | b'[' => level += 1,
            b'}' | b']' => level -= 1,
            _ => {}
        }
    }

    b'v' as c_char
}

// original: fnext (htslib/textutils.c:397)
pub unsafe fn textutils_c_397_fnext(
    arg1: *mut c_void,
    arg2: *mut c_void,
    token: *mut hts_json_token,
) -> c_char {
    textutils_c_292_hts_json_fnext(arg1.cast(), token, arg2.cast())
}

// original: hts_json_fskip_value (htslib/textutils.c:402)
pub unsafe fn textutils_c_402_hts_json_fskip_value(fp: *mut hFILE, type_: c_char) -> c_char {
    let mut str_: kstring_t = std::mem::zeroed();
    let ret = skip_value(
        type_,
        textutils_c_397_fnext,
        fp.cast(),
        (&mut str_ as *mut kstring_t).cast(),
    );
    libc::free(str_.s.cast());
    ret
}

// ----------------------------------------------------------------------
// Functions translated from htslib/textutils.c (moved from src/hts.rs).
// ----------------------------------------------------------------------

pub unsafe fn hts_decode_percent(
    dest: *mut c_char,
    destlen: *mut size_t,
    mut s: *const c_char,
) -> c_int {
    let mut d = dest;
    while *s != 0 {
        let hi;
        let lo;
        if *s == b'%' as c_char
            && {
                hi = dehex(*s.add(1));
                hi >= 0
            }
            && {
                lo = dehex(*s.add(2));
                lo >= 0
            }
        {
            *d = ((hi << 4) | lo) as c_char;
            d = d.add(1);
            s = s.add(3);
        } else {
            *d = *s;
            d = d.add(1);
            s = s.add(1);
        }
    }
    *d = 0;
    *destlen = d.offset_from(dest) as size_t;
    0
}

pub unsafe fn debase64(c: c_char) -> c_int {
    let c = c as u8;
    if c.is_ascii_lowercase() {
        (c - b'a' + 26) as c_int
    } else if c.is_ascii_uppercase() {
        (c - b'A') as c_int
    } else if c.is_ascii_digit() {
        (c - b'0' + 52) as c_int
    } else if c == b'/' {
        63
    } else if c == b'+' {
        62
    } else {
        -1
    }
}

pub unsafe fn hts_base64_decoded_length(len: size_t) -> size_t {
    let nquartets = (len + 2) / 4;
    3 * nquartets
}

pub unsafe fn hts_decode_base64(
    dest: *mut c_char,
    destlen: *mut size_t,
    mut s: *const c_char,
) -> c_int {
    let mut d = dest;
    let mut x0;
    let mut x1;
    let mut x2;
    let mut x3;

    loop {
        x0 = debase64(*s);
        s = s.add(1);
        x1 = if x0 >= 0 {
            let x = debase64(*s);
            s = s.add(1);
            x
        } else {
            -1
        };
        x2 = if x1 >= 0 {
            let x = debase64(*s);
            s = s.add(1);
            x
        } else {
            -1
        };
        x3 = if x2 >= 0 {
            let x = debase64(*s);
            s = s.add(1);
            x
        } else {
            -1
        };
        if x3 < 0 {
            break;
        }

        *d = ((x0 << 2) | (x1 >> 4)) as c_char;
        d = d.add(1);
        *d = ((x1 << 4) | (x2 >> 2)) as c_char;
        d = d.add(1);
        *d = ((x2 << 6) | x3) as c_char;
        d = d.add(1);
    }

    if x1 >= 0 {
        *d = ((x0 << 2) | (x1 >> 4)) as c_char;
        d = d.add(1);
    }
    if x2 >= 0 {
        *d = ((x1 << 4) | (x2 >> 2)) as c_char;
        d = d.add(1);
    }

    *destlen = d.offset_from(dest) as size_t;
    0
}

pub unsafe fn sscan_string(mut s: *mut c_char) -> *mut c_char {
    let mut d = s;
    loop {
        match *s as u8 {
            b'\\' => match *s.add(1) as u8 {
                0 => {
                    *d = 0;
                    return s.add(1);
                }
                b'b' => {
                    *d = b'\x08' as c_char;
                    d = d.add(1);
                    s = s.add(2);
                }
                b'f' => {
                    *d = b'\x0c' as c_char;
                    d = d.add(1);
                    s = s.add(2);
                }
                b'n' => {
                    *d = b'\n' as c_char;
                    d = d.add(1);
                    s = s.add(2);
                }
                b'r' => {
                    *d = b'\r' as c_char;
                    d = d.add(1);
                    s = s.add(2);
                }
                b't' => {
                    *d = b'\t' as c_char;
                    d = d.add(1);
                    s = s.add(2);
                }
                b'u' => {
                    let d1 = dehex(*s.add(2));
                    let d2 = dehex(*s.add(3));
                    let d3 = dehex(*s.add(4));
                    let d4 = dehex(*s.add(5));
                    if d1 >= 0 && d2 >= 0 && d3 >= 0 && d4 >= 0 {
                        d = encode_utf8(d, ((d1 << 12) | (d2 << 8) | (d3 << 4) | d4) as u32);
                        s = s.add(6);
                    }
                }
                _ => {
                    *d = *s.add(1);
                    d = d.add(1);
                    s = s.add(2);
                }
            },
            b'"' => {
                *d = 0;
                return s.add(1);
            }
            0 => {
                *d = 0;
                return s;
            }
            _ => {
                *d = *s;
                d = d.add(1);
                s = s.add(1);
            }
        }
    }
}

pub unsafe fn hts_json_alloc_token() -> *mut hts_json_token {
    crate::htslib_rs::c_compat::calloc(1, std::mem::size_of::<hts_json_token>() as u64).cast()
}

pub unsafe fn hts_json_token_type(token: *mut hts_json_token) -> c_char {
    (*token).type_
}

pub unsafe fn hts_json_free_token(token: *mut hts_json_token) {
    crate::htslib_rs::c_compat::free(token.cast());
}

pub unsafe fn hts_json_token_str(token: *mut hts_json_token) -> *mut c_char {
    (*token).str_
}

pub unsafe fn hts_json_snext(
    str_: *mut c_char,
    state: *mut size_t,
    token: *mut hts_json_token,
) -> c_char {
    let mut s = str_.add(*state >> 2);
    let mut hidden = *state & 3;

    if hidden != 0 {
        *state &= !3;
        (*token).type_ = *c"?}]?".as_ptr().add(hidden) as c_char;
        return (*token).type_;
    }

    loop {
        match *s as u8 {
            b' ' | b'\t' | b'\r' | b'\n' | b',' | b':' => {
                s = s.add(1);
            }
            0 => {
                (*token).type_ = 0;
                return 0;
            }
            b'{' | b'[' | b'}' | b']' => {
                *state = (s.add(1).offset_from(str_) as size_t) << 2;
                (*token).type_ = *s;
                return (*token).type_;
            }
            b'"' => {
                (*token).str_ = s.add(1);
                *state = (sscan_string(s.add(1)).offset_from(str_) as size_t) << 2;
                (*token).type_ = b's' as c_char;
                return (*token).type_;
            }
            _ => {
                (*token).str_ = s;
                while *s != 0
                    && !matches!(*s as u8, b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}')
                {
                    s = s.add(1);
                }
                hidden = if *s == b'}' as c_char {
                    1
                } else if *s == b']' as c_char {
                    2
                } else {
                    0
                };
                if *s != 0 {
                    *s = 0;
                    s = s.add(1);
                }
                *state = ((s.offset_from(str_) as size_t) << 2) | hidden;
                (*token).type_ = crate::htslib_rs::hts::token_type(token);
                return (*token).type_;
            }
        }
    }
}

pub unsafe fn snext(arg1: *mut c_void, arg2: *mut c_void, token: *mut hts_json_token) -> c_char {
    hts_json_snext(arg1.cast(), arg2.cast(), token)
}

pub unsafe fn hts_json_sskip_value(str_: *mut c_char, state: *mut size_t, type_: c_char) -> c_char {
    crate::htslib_rs::hts::skip_value(type_, Some(snext), str_.cast(), state.cast())
}

pub unsafe fn stringify_argv(argc: c_int, argv: *mut *mut c_char) -> *mut c_char {
    let mut nbytes = 1usize;
    for i in 0..argc {
        if i > 0 {
            nbytes += 1;
        }
        nbytes += libc::strlen(*argv.add(i as usize));
    }
    let str_ = crate::htslib_rs::c_compat::malloc(nbytes as u64).cast::<c_char>();
    if str_.is_null() {
        return std::ptr::null_mut();
    }

    let mut cp = str_;
    for i in 0..argc {
        if i > 0 {
            *cp = b' ' as c_char;
            cp = cp.add(1);
        }
        let arg = *argv.add(i as usize);
        let mut j = 0usize;
        while *arg.add(j) != 0 {
            if *arg.add(j) == b'\t' as c_char {
                *cp = b' ' as c_char;
            } else {
                *cp = *arg.add(j);
            }
            cp = cp.add(1);
            j += 1;
        }
    }
    *cp = 0;
    str_
}

pub unsafe fn hts_strprint(
    buf: *mut c_char,
    buflen: size_t,
    quote: c_char,
    mut s: *const c_char,
    len: size_t,
) -> *const c_char {
    let slim = if len < size_t::MAX {
        s.add(len)
    } else {
        std::ptr::null()
    };
    let mut t = buf;
    let qlen = if quote != 0 { 1usize } else { 0usize };
    if quote != 0 {
        *t = quote;
        t = t.add(1);
    }

    while if !slim.is_null() { s < slim } else { *s != 0 } {
        let c;
        let clen;
        match *s as u8 {
            b'\n' => {
                c = b'n' as c_char;
                clen = 2usize;
            }
            b'\r' => {
                c = b'r' as c_char;
                clen = 2usize;
            }
            b'\t' => {
                c = b't' as c_char;
                clen = 2usize;
            }
            0 => {
                c = b'0' as c_char;
                clen = 2usize;
            }
            b'\\' => {
                c = b'\\' as c_char;
                clen = 2usize;
            }
            _ => {
                c = *s;
                if c == quote {
                    clen = 2usize;
                } else if isprint_c(c) != 0 {
                    clen = 1usize;
                } else {
                    clen = 4usize;
                }
            }
        }

        if t.offset_from(buf) as usize + clen + qlen >= buflen {
            while t.offset_from(buf) as usize + 3 + qlen >= buflen {
                t = t.sub(1);
            }
            if quote != 0 {
                *t = quote;
                t = t.add(1);
            }
            crate::htslib_rs::c_compat::memcpy(t.cast(), c"...".as_ptr().cast(), 4);
            return buf;
        }

        if clen == 4 {
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            let c = c as u8;
            *t = b'\\' as c_char;
            *t.add(1) = b'x' as c_char;
            *t.add(2) = HEX[(c >> 4) as usize] as c_char;
            *t.add(3) = HEX[(c & 0x0f) as usize] as c_char;
            t = t.add(4);
        } else {
            if clen == 2 {
                *t = b'\\' as c_char;
                t = t.add(1);
            }
            *t = c;
            t = t.add(1);
        }
        s = s.add(1);
    }

    if quote != 0 {
        *t = quote;
        t = t.add(1);
    }
    *t = 0;
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TokenStream {
        tokens: &'static [u8],
        pos: usize,
    }

    unsafe fn next_token(
        arg1: *mut c_void,
        _arg2: *mut c_void,
        token: *mut hts_json_token,
    ) -> c_char {
        let stream = &mut *arg1.cast::<TokenStream>();
        if stream.pos == stream.tokens.len() {
            (*token).type_ = 0;
            return 0;
        }

        let type_ = stream.tokens[stream.pos] as c_char;
        stream.pos += 1;
        (*token).type_ = type_;
        type_
    }

    #[test]
    fn dehex_accepts_hex_digits_only() {
        unsafe {
            assert_eq!(dehex(b'0' as c_char), 0);
            assert_eq!(dehex(b'9' as c_char), 9);
            assert_eq!(dehex(b'a' as c_char), 10);
            assert_eq!(dehex(b'F' as c_char), 15);
            assert_eq!(dehex(b'g' as c_char), -1);
            assert_eq!(dehex(b':' as c_char), -1);
        }
    }

    #[test]
    fn encode_utf8_writes_minimal_width_sequences() {
        unsafe {
            let cases = [
                (0x24, b"$".as_slice()),
                (0xa2, b"\xc2\xa2".as_slice()),
                (0x20ac, b"\xe2\x82\xac".as_slice()),
                (0x1f600, b"\xf0\x9f\x98\x80".as_slice()),
            ];

            for (codepoint, expected) in cases {
                let mut buf = [0 as c_char; 8];
                let end = encode_utf8(buf.as_mut_ptr(), codepoint);
                let len = end.offset_from(buf.as_ptr()) as usize;
                assert_eq!(
                    std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), len),
                    expected
                );
            }
        }
    }

    #[test]
    fn token_type_classifies_json_atoms() {
        unsafe {
            let mut token = hts_json_token {
                type_: 0,
                str_: c"false".as_ptr().cast_mut(),
            };
            assert_eq!(token_type(&mut token), b'b' as c_char);

            token.str_ = c"true".as_ptr().cast_mut();
            assert_eq!(token_type(&mut token), b'b' as c_char);

            token.str_ = c"null".as_ptr().cast_mut();
            assert_eq!(token_type(&mut token), b'.' as c_char);

            token.str_ = c"-12.5e3".as_ptr().cast_mut();
            assert_eq!(token_type(&mut token), b'n' as c_char);

            token.str_ = c"truthy".as_ptr().cast_mut();
            assert_eq!(token_type(&mut token), b'?' as c_char);
        }
    }

    #[test]
    fn skip_value_consumes_nested_values_and_reports_bad_tokens() {
        unsafe {
            let mut nested = TokenStream {
                tokens: b"n[bn]}",
                pos: 0,
            };
            assert_eq!(
                skip_value(
                    b'{' as c_char,
                    next_token,
                    (&mut nested as *mut TokenStream).cast(),
                    std::ptr::null_mut(),
                ),
                b'v' as c_char
            );
            assert_eq!(nested.pos, nested.tokens.len());

            let mut bad = TokenStream {
                tokens: b"?",
                pos: 0,
            };
            assert_eq!(
                skip_value(
                    b'[' as c_char,
                    next_token,
                    (&mut bad as *mut TokenStream).cast(),
                    std::ptr::null_mut(),
                ),
                b'?' as c_char
            );

            let mut truncated = TokenStream {
                tokens: b"n",
                pos: 0,
            };
            assert_eq!(
                skip_value(
                    b'{' as c_char,
                    next_token,
                    (&mut truncated as *mut TokenStream).cast(),
                    std::ptr::null_mut(),
                ),
                0
            );
        }
    }
}
