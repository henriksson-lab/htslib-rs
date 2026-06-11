use crate::htslib_rs::hts::{hFILE, hts_json_token, isprint_c, kstring_t, size_t};
use std::ffi::c_void;
use std::ptr::NonNull;

const EOF: i32 = -1;

// original: dehex (htslib/textutils.c:37)
fn dehex(c: i8) -> i32 {
    let c = c as u8;
    if (b'a'..=b'f').contains(&c) {
        (c - b'a' + 10) as i32
    } else if (b'A'..=b'F').contains(&c) {
        (c - b'A' + 10) as i32
    } else if c.is_ascii_digit() {
        (c - b'0') as i32
    } else {
        -1
    }
}

fn encode_utf8_slice(s: &mut [i8], x: u32) -> usize {
    if x >= 0x10000 {
        s[0] = (0xf0 | (x >> 18)) as i8;
        s[1] = (0x80 | ((x >> 12) & 0x3f)) as i8;
        s[2] = (0x80 | ((x >> 6) & 0x3f)) as i8;
        s[3] = (0x80 | (x & 0x3f)) as i8;
        4
    } else if x >= 0x800 {
        s[0] = (0xe0 | (x >> 12)) as i8;
        s[1] = (0x80 | ((x >> 6) & 0x3f)) as i8;
        s[2] = (0x80 | (x & 0x3f)) as i8;
        3
    } else if x >= 0x80 {
        s[0] = (0xc0 | (x >> 6)) as i8;
        s[1] = (0x80 | (x & 0x3f)) as i8;
        2
    } else {
        s[0] = x as i8;
        1
    }
}

// original: fscan_string (htslib/textutils.c:164)
pub unsafe fn fscan_string(fp: &mut hFILE, d: &mut kstring_t) -> i32 {
    let mut e: u32 = 0;

    loop {
        let mut c = crate::htslib_rs::hfile::hgetc(fp);
        if c == EOF {
            break;
        }

        match c as u8 {
            b'\\' => {
                c = crate::htslib_rs::hfile::hgetc(fp);
                if c == EOF {
                    return if e == 0 { 0 } else { -1 };
                }

                match c as u8 {
                    b'b' => e |= (crate::htslib_rs::hts::kputc(b'\x08' as i32, d) < 0) as u32,
                    b'f' => e |= (crate::htslib_rs::hts::kputc(b'\x0c' as i32, d) < 0) as u32,
                    b'n' => e |= (crate::htslib_rs::hts::kputc(b'\n' as i32, d) < 0) as u32,
                    b'r' => e |= (crate::htslib_rs::hts::kputc(b'\r' as i32, d) < 0) as u32,
                    b't' => e |= (crate::htslib_rs::hts::kputc(b'\t' as i32, d) < 0) as u32,
                    b'u' => {
                        c = crate::htslib_rs::hfile::hgetc(fp);
                        if c != EOF {
                            let d1 = dehex(c as i8);
                            if d1 >= 0 {
                                c = crate::htslib_rs::hfile::hgetc(fp);
                                if c != EOF {
                                    let d2 = dehex(c as i8);
                                    if d2 >= 0 {
                                        c = crate::htslib_rs::hfile::hgetc(fp);
                                        if c != EOF {
                                            let d3 = dehex(c as i8);
                                            if d3 >= 0 {
                                                c = crate::htslib_rs::hfile::hgetc(fp);
                                                if c != EOF {
                                                    let d4 = dehex(c as i8);
                                                    if d4 >= 0 {
                                                        let mut buf = [0 as i8; 8];
                                                        let len = encode_utf8_slice(
                                                            &mut buf,
                                                            ((d1 << 12)
                                                                | (d2 << 8)
                                                                | (d3 << 4)
                                                                | d4)
                                                                as u32,
                                                        );
                                                        let buf_u8: &[u8] = std::slice::from_raw_parts(
                                                            buf.as_ptr().cast::<u8>(),
                                                            buf.len(),
                                                        );
                                                        e |= (crate::htslib_rs::hts::kputsn(
                                                            buf_u8,
                                                            len,
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

unsafe fn c_strlen(s: *const i8) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

unsafe fn token_bytes(token: &hts_json_token) -> Option<&[u8]> {
    let s = token.str_;
    if s.is_null() {
        return None;
    }
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    Some(std::slice::from_raw_parts(s.cast::<u8>(), len))
}

unsafe fn token_type(token: &hts_json_token) -> i8 {
    match token_bytes(token) {
        Some(b"false") | Some(b"true") => b'b' as i8,
        Some(b"null") => b'.' as i8,
        Some([b'-', ..] | [b'0'..=b'9', ..]) => b'n' as i8,
        _ => b'?' as i8,
    }
}

// original: hts_json_fnext (htslib/textutils.c:292)
pub unsafe fn hts_json_fnext(
    fp: &mut hFILE,
    token: &mut hts_json_token,
    kstr: &mut kstring_t,
) -> i8 {
    loop {
        let mut c = crate::htslib_rs::hfile::hgetc(fp);

        match c {
            x if matches!(x as u8, b' ' | b'\t' | b'\r' | b'\n' | b',' | b':') => continue,
            x if x == EOF => {
                token.type_ = 0;
                return token.type_;
            }
            x if matches!(x as u8, b'{' | b'[' | b'}' | b']') => {
                token.type_ = x as i8;
                return token.type_;
            }
            x if x == b'"' as i32 => {
                kstr.data.clear();
                fscan_string(fp, kstr);
                // NUL-terminate the owned buffer so the raw token.str_ pointer
                // exposes a valid C string at this FFI-style boundary.
                kstr.data.push(0);
                token.str_ = kstr.data.as_mut_ptr().cast();
                token.type_ = b's' as i8;
                return token.type_;
            }
            _ => {
                let mut peek = [0u8; 1];

                kstr.data.clear();
                crate::htslib_rs::hts::kputc(c, kstr);
                while crate::htslib_rs::hfile::hpeek_impl(fp, &mut peek) == 1
                    && !matches!(
                        peek[0],
                        b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}'
                    )
                {
                    c = crate::htslib_rs::hfile::hgetc(fp);
                    if c == EOF {
                        break;
                    }
                    crate::htslib_rs::hts::kputc(c, kstr);
                }
                // NUL-terminate the owned buffer so the raw token.str_ pointer
                // exposes a valid C string at this FFI-style boundary.
                kstr.data.push(0);
                token.str_ = kstr.data.as_mut_ptr().cast();
                token.type_ = token_type(token);
                return token.type_;
            }
        }
    }
}

// original: skip_value (htslib/textutils.c:338)
unsafe fn skip_value(type_: i8, next: &mut dyn FnMut(&mut hts_json_token) -> i8) -> i8 {
    let mut token = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };

    let first = if type_ != 0 { type_ } else { next(&mut token) };

    let mut level;
    match first as u8 {
        0 => return 0,
        b'?' | b'}' | b']' => return b'?' as i8,
        b'{' | b'[' => level = 1,
        _ => return b'v' as i8,
    }

    while level > 0 {
        match next(&mut token) as u8 {
            0 => return 0,
            b'?' => return b'?' as i8,
            b'{' | b'[' => level += 1,
            b'}' | b']' => level -= 1,
            _ => {}
        }
    }

    b'v' as i8
}

// original: fnext (htslib/textutils.c:397)
pub unsafe fn textutils_fnext(
    arg1: *mut c_void,
    arg2: *mut c_void,
    token: *mut hts_json_token,
) -> i8 {
    if token.is_null() {
        return 0;
    }
    let Some(fp) = arg1.cast::<hFILE>().as_mut() else {
        return 0;
    };
    let Some(kstr) = arg2.cast::<kstring_t>().as_mut() else {
        return 0;
    };
    hts_json_fnext(fp, &mut *token, kstr)
}

// original: hts_json_fskip_value (htslib/textutils.c:402)
pub unsafe fn hts_json_fskip_value(fp: &mut hFILE, type_: i8) -> i8 {
    let mut str_: kstring_t = kstring_t::default();
    skip_value(type_, &mut |token| {
        hts_json_fnext(fp, token, &mut str_)
    })
}

// ----------------------------------------------------------------------
// Functions translated from htslib/textutils.c (moved from src/hts.rs).
// ----------------------------------------------------------------------

pub fn hts_decode_percent(dest: &mut [i8], destlen: &mut size_t, src: &[u8]) -> i32 {
    if dest.is_empty() {
        return -1;
    }
    let mut si = 0usize;
    let mut di = 0usize;
    while si < src.len() {
        if di + 1 >= dest.len() {
            return -1;
        }
        if src[si] == b'%' && si + 2 < src.len() {
            let hi = dehex(src[si + 1] as i8);
            let lo = dehex(src[si + 2] as i8);
            if hi >= 0 && lo >= 0 {
                dest[di] = ((hi << 4) | lo) as i8;
                di += 1;
                si += 3;
                continue;
            }
        }
        dest[di] = src[si] as i8;
        di += 1;
        si += 1;
    }
    if di >= dest.len() {
        -1
    } else {
        dest[di] = 0;
        *destlen = di as size_t;
        0
    }
}

pub fn debase64(c: i8) -> i32 {
    let c = c as u8;
    if c.is_ascii_lowercase() {
        (c - b'a' + 26) as i32
    } else if c.is_ascii_uppercase() {
        (c - b'A') as i32
    } else if c.is_ascii_digit() {
        (c - b'0' + 52) as i32
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

pub fn hts_decode_base64(dest: &mut [i8], destlen: &mut size_t, src: &[u8]) -> i32 {
    if dest.is_empty() && !src.is_empty() {
        return -1;
    }
    let mut si = 0usize;
    let mut di = 0usize;
    let mut x0;
    let mut x1;
    let mut x2;
    let mut x3;

    loop {
        x0 = src.get(si).map_or(-1, |&c| debase64(c as i8));
        si += 1;
        x1 = if x0 >= 0 {
            let x = src.get(si).map_or(-1, |&c| debase64(c as i8));
            si += 1;
            x
        } else {
            -1
        };
        x2 = if x1 >= 0 {
            let x = src.get(si).map_or(-1, |&c| debase64(c as i8));
            si += 1;
            x
        } else {
            -1
        };
        x3 = if x2 >= 0 {
            let x = src.get(si).map_or(-1, |&c| debase64(c as i8));
            si += 1;
            x
        } else {
            -1
        };
        if x3 < 0 {
            break;
        }

        if di + 3 > dest.len() {
            return -1;
        }
        dest[di] = ((x0 << 2) | (x1 >> 4)) as i8;
        di += 1;
        dest[di] = ((x1 << 4) | (x2 >> 2)) as i8;
        di += 1;
        dest[di] = ((x2 << 6) | x3) as i8;
        di += 1;
    }

    if x1 >= 0 {
        if di >= dest.len() {
            return -1;
        }
        dest[di] = ((x0 << 2) | (x1 >> 4)) as i8;
        di += 1;
    }
    if x2 >= 0 {
        if di >= dest.len() {
            return -1;
        }
        dest[di] = ((x1 << 4) | (x2 >> 2)) as i8;
        di += 1;
    }

    *destlen = di as size_t;
    0
}

pub unsafe fn sscan_string(s: &mut [i8]) -> Option<usize> {
    let mut i = 0usize;
    let mut d = 0usize;
    loop {
        let c = *s.get(i)?;
        match c as u8 {
            b'\\' => {
                let escaped = *s.get(i + 1)?;
                match escaped as u8 {
                    0 => {
                        *s.get_mut(d)? = 0;
                        return Some(i + 1);
                    }
                    b'b' => {
                        *s.get_mut(d)? = b'\x08' as i8;
                        d += 1;
                        i += 2;
                    }
                    b'f' => {
                        *s.get_mut(d)? = b'\x0c' as i8;
                        d += 1;
                        i += 2;
                    }
                    b'n' => {
                        *s.get_mut(d)? = b'\n' as i8;
                        d += 1;
                        i += 2;
                    }
                    b'r' => {
                        *s.get_mut(d)? = b'\r' as i8;
                        d += 1;
                        i += 2;
                    }
                    b't' => {
                        *s.get_mut(d)? = b'\t' as i8;
                        d += 1;
                        i += 2;
                    }
                    b'u' => {
                        let d1 = dehex(*s.get(i + 2)?);
                        let d2 = dehex(*s.get(i + 3)?);
                        let d3 = dehex(*s.get(i + 4)?);
                        let d4 = dehex(*s.get(i + 5)?);
                        if d1 >= 0 && d2 >= 0 && d3 >= 0 && d4 >= 0 {
                            let len = encode_utf8_slice(
                                s.get_mut(d..d + 4)?,
                                ((d1 << 12) | (d2 << 8) | (d3 << 4) | d4) as u32,
                            );
                            d += len;
                            i += 6;
                        } else {
                            return None;
                        }
                    }
                    _ => {
                        *s.get_mut(d)? = escaped;
                        d += 1;
                        i += 2;
                    }
                }
            }
            b'"' => {
                *s.get_mut(d)? = 0;
                return Some(i + 1);
            }
            0 => {
                *s.get_mut(d)? = 0;
                return Some(i);
            }
            _ => {
                *s.get_mut(d)? = c;
                d += 1;
                i += 1;
            }
        }
    }
}

pub unsafe fn hts_json_alloc_token() -> *mut hts_json_token {
    Box::into_raw(hts_json_token_new())
}

pub fn hts_json_token_new() -> Box<hts_json_token> {
    Box::new(hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    })
}

pub unsafe fn hts_json_token_type(token: *mut hts_json_token) -> i8 {
    token.as_ref().map_or(0, |token| token.type_)
}

pub unsafe fn hts_json_free_token(token: *mut hts_json_token) {
    if !token.is_null() {
        drop(Box::from_raw(token));
    }
}

pub fn hts_json_token_str(token: &hts_json_token) -> *mut i8 {
    token.str_
}

pub unsafe fn hts_json_snext(
    str_: *mut i8,
    state: &mut size_t,
    token: &mut hts_json_token,
) -> i8 {
    let Some(str_) = NonNull::new(str_) else {
        return 0;
    };
    hts_json_snext_nonnull(str_, state, token)
}

unsafe fn hts_json_snext_nonnull(
    str_: NonNull<i8>,
    state: &mut size_t,
    token: &mut hts_json_token,
) -> i8 {
    let str_ptr = str_.as_ptr();
    let mut s = str_ptr.add(*state >> 2);
    let mut hidden = *state & 3;

    if hidden != 0 {
        *state &= !3;
        token.type_ = b"?}]?"[hidden] as i8;
        return token.type_;
    }

    loop {
        match *s as u8 {
            b' ' | b'\t' | b'\r' | b'\n' | b',' | b':' => {
                s = s.add(1);
            }
            0 => {
                token.type_ = 0;
                return 0;
            }
            b'{' | b'[' | b'}' | b']' => {
                *state = (s.add(1).offset_from(str_ptr) as size_t) << 2;
                token.type_ = *s;
                return token.type_;
            }
            b'"' => {
                token.str_ = s.add(1);
                let scan_start = s.add(1);
                let scan_len = c_strlen(scan_start).saturating_add(1);
                let scan_bytes = std::slice::from_raw_parts_mut(scan_start, scan_len);
                let Some(consumed) = sscan_string(scan_bytes) else {
                    token.type_ = b'?' as i8;
                    return token.type_;
                };
                *state = (scan_start.add(consumed).offset_from(str_ptr) as size_t) << 2;
                token.type_ = b's' as i8;
                return token.type_;
            }
            _ => {
                token.str_ = s;
                while *s != 0
                    && !matches!(*s as u8, b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}')
                {
                    s = s.add(1);
                }
                hidden = if *s == b'}' as i8 {
                    1
                } else if *s == b']' as i8 {
                    2
                } else {
                    0
                };
                if *s != 0 {
                    *s = 0;
                    s = s.add(1);
                }
                *state = ((s.offset_from(str_ptr) as size_t) << 2) | hidden;
                token.type_ = token_type(token);
                return token.type_;
            }
        }
    }
}

pub unsafe fn snext(arg1: *mut c_void, arg2: *mut c_void, token: *mut hts_json_token) -> i8 {
    if token.is_null() {
        return 0;
    }
    let Some(state) = arg2.cast::<size_t>().as_mut() else {
        return 0;
    };
    hts_json_snext(arg1.cast(), state, &mut *token)
}

pub unsafe fn hts_json_sskip_value(str_: *mut i8, state: &mut size_t, type_: i8) -> i8 {
    let Some(str_) = NonNull::new(str_) else {
        return 0;
    };
    skip_value(type_, &mut |token| {
        hts_json_snext_nonnull(str_, state, token)
    })
}

pub fn stringify_argv(args: &[&[u8]]) -> Option<Vec<u8>> {
    let mut bytes = Vec::<u8>::with_capacity(
        args.iter().map(|arg| arg.len()).sum::<usize>() + args.len().saturating_sub(1),
    );

    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            bytes.push(b' ');
        }
        bytes.extend(
            arg.iter()
                .map(|&byte| if byte == b'\t' { b' ' } else { byte }),
        );
    }
    Some(bytes)
}

pub fn hts_strprint(buf: &mut [i8], quote: i8, s: &[u8]) -> bool {
    let Some(bytes) = hts_strprint_bytes(buf.len(), quote, s) else {
        return false;
    };
    for (dest, src) in buf.iter_mut().zip(bytes) {
        *dest = src as i8;
    }
    true
}

fn hts_strprint_bytes(buflen: usize, quote: i8, s: &[u8]) -> Option<Vec<u8>> {
    let quote = (quote != 0).then_some(quote as u8);
    let full = hts_strprint_full_bytes(quote, s);
    if full.len() <= buflen {
        return Some(full);
    }

    let qlen = usize::from(quote.is_some());
    let suffix_len = qlen + 4;
    if buflen < suffix_len {
        return None;
    }

    let mut bytes = Vec::with_capacity(buflen);
    if let Some(quote) = quote {
        bytes.push(quote);
    }

    for &byte in s {
        let chunk = hts_strprint_chunk(quote, byte);
        if bytes.len() + chunk.len() + suffix_len > buflen {
            break;
        }
        bytes.extend(chunk);
    }

    if let Some(quote) = quote {
        bytes.push(quote);
    }
    bytes.extend_from_slice(b"...\0");
    Some(bytes)
}

fn hts_strprint_full_bytes(quote: Option<u8>, s: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(s.len().saturating_add(2));
    if let Some(quote) = quote {
        bytes.push(quote);
    }
    for &byte in s {
        bytes.extend(hts_strprint_chunk(quote, byte));
    }
    if let Some(quote) = quote {
        bytes.push(quote);
    }
    bytes.push(0);
    bytes
}

fn hts_strprint_chunk(quote: Option<u8>, c: u8) -> Vec<u8> {
    match c {
        b'\n' => b"\\n".to_vec(),
        b'\r' => b"\\r".to_vec(),
        b'\t' => b"\\t".to_vec(),
        0 => b"\\0".to_vec(),
        b'\\' => b"\\\\".to_vec(),
        c if Some(c) == quote => vec![b'\\', c],
        c if isprint_c(c as i8) != 0 => vec![c],
        c => {
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            vec![
                b'\\',
                b'x',
                HEX[(c >> 4) as usize],
                HEX[(c & 0x0f) as usize],
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TokenStream {
        tokens: &'static [u8],
        pos: usize,
    }

    fn next_token(stream: &mut TokenStream, token: &mut hts_json_token) -> i8 {
        if stream.pos == stream.tokens.len() {
            token.type_ = 0;
            return 0;
        }

        let type_ = stream.tokens[stream.pos] as i8;
        stream.pos += 1;
        token.type_ = type_;
        type_
    }

    #[test]
    fn dehex_accepts_hex_digits_only() {
        assert_eq!(dehex(b'0' as i8), 0);
        assert_eq!(dehex(b'9' as i8), 9);
        assert_eq!(dehex(b'a' as i8), 10);
        assert_eq!(dehex(b'F' as i8), 15);
        assert_eq!(dehex(b'g' as i8), -1);
        assert_eq!(dehex(b':' as i8), -1);
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
                let mut buf = [0 as i8; 8];
                let len = encode_utf8_slice(&mut buf, codepoint);
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
            assert_eq!(token_type(&token), b'b' as i8);

            token.str_ = c"true".as_ptr().cast_mut();
            assert_eq!(token_type(&token), b'b' as i8);

            token.str_ = c"null".as_ptr().cast_mut();
            assert_eq!(token_type(&token), b'.' as i8);

            token.str_ = c"-12.5e3".as_ptr().cast_mut();
            assert_eq!(token_type(&token), b'n' as i8);

            token.str_ = c"truthy".as_ptr().cast_mut();
            assert_eq!(token_type(&token), b'?' as i8);
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
                skip_value(b'{' as i8, &mut |token| next_token(&mut nested, token)),
                b'v' as i8
            );
            assert_eq!(nested.pos, nested.tokens.len());

            let mut bad = TokenStream {
                tokens: b"?",
                pos: 0,
            };
            assert_eq!(
                skip_value(b'[' as i8, &mut |token| next_token(&mut bad, token)),
                b'?' as i8
            );

            let mut truncated = TokenStream {
                tokens: b"n",
                pos: 0,
            };
            assert_eq!(
                skip_value(b'{' as i8, &mut |token| {
                    next_token(&mut truncated, token)
                }),
                0
            );
        }
    }
}
