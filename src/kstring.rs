use crate::htslib_rs::hts::{
    kgets_func, kgets_func2, kputc, kputsn, ks_resize, ks_tokaux_t, kstring_t, size_t,
};
use std::ffi::{c_char, c_int, c_void, CStr};

// original: kvsprintf (htslib/kstring.c:142)
pub unsafe fn kstring_c_142_kvsprintf(
    s: *mut kstring_t,
    fmt: *const c_char,
    ap: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> c_int {
    let mut args = std::mem::MaybeUninit::<crate::htslib_rs::c_compat::__va_list_tag>::uninit();
    std::ptr::copy_nonoverlapping(ap, args.as_mut_ptr(), 1);
    let mut args = args.assume_init();

    if *fmt == b'%' as c_char && *fmt.add(1) == b'g' as c_char && *fmt.add(2) == 0 {
        let d = if args.fp_offset <= 160 {
            let p = args.reg_save_area.cast::<u8>().add(args.fp_offset as usize);
            std::ptr::read_unaligned(p.cast::<f64>())
        } else {
            let p = args.overflow_arg_area.cast::<u8>();
            std::ptr::read_unaligned(p.cast::<f64>())
        };
        return kputd(d, s);
    }

    if (*s).s.is_null() {
        let sz = 64usize;
        (*s).s = libc::malloc(sz).cast::<c_char>();
        if (*s).s.is_null() {
            return -1;
        }
        (*s).m = sz;
        (*s).l = 0;
    }

    let mut l = crate::htslib_rs::c_compat::vsnprintf(
        (*s).s.add((*s).l),
        ((*s).m - (*s).l) as u64,
        fmt,
        &mut args,
    );
    if l + 1 > ((*s).m - (*s).l) as c_int {
        if ks_resize(s, (*s).l + l as usize + 2) < 0 {
            return -1;
        }
        let mut args = std::mem::MaybeUninit::<crate::htslib_rs::c_compat::__va_list_tag>::uninit();
        std::ptr::copy_nonoverlapping(ap, args.as_mut_ptr(), 1);
        let mut args = args.assume_init();
        l = crate::htslib_rs::c_compat::vsnprintf(
            (*s).s.add((*s).l),
            ((*s).m - (*s).l) as u64,
            fmt,
            &mut args,
        );
    }
    (*s).l += l as usize;
    l
}

pub enum KsPrintfArg {
    Int(c_int),
    Str(*const c_char),
}

// Stable Rust cannot define C-variadic functions.  This helper keeps the
// translated local callers usable without relying on nightly-only syntax.
pub unsafe fn kstring_c_177_ksprintf(
    s: *mut kstring_t,
    fmt: *const c_char,
    args: &[KsPrintfArg],
) -> c_int {
    if fmt.is_null() {
        return -1;
    }

    let fmt = CStr::from_ptr(fmt).to_bytes();
    let mut out = Vec::new();
    let mut arg_i = 0usize;
    let mut i = 0usize;
    while i < fmt.len() {
        if fmt[i] != b'%' {
            out.push(fmt[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= fmt.len() {
            return -1;
        }
        match fmt[i] {
            b'%' => out.push(b'%'),
            b'd' => {
                let Some(KsPrintfArg::Int(v)) = args.get(arg_i) else {
                    return -1;
                };
                out.extend_from_slice(v.to_string().as_bytes());
                arg_i += 1;
            }
            b's' => {
                let Some(KsPrintfArg::Str(v)) = args.get(arg_i) else {
                    return -1;
                };
                if v.is_null() {
                    return -1;
                }
                out.extend_from_slice(CStr::from_ptr(*v).to_bytes());
                arg_i += 1;
            }
            _ => return -1,
        }
        i += 1;
    }

    kputsn(out.as_ptr().cast(), out.len(), s)
}

// original: main (htslib/kstring.c:531)
pub unsafe fn kstring_c_531_main() -> c_int {
    let s = libc::calloc(1, std::mem::size_of::<kstring_t>()).cast::<kstring_t>();
    let mut n = 0;
    let mut aux: ks_tokaux_t = std::mem::zeroed();

    kstring_c_177_ksprintf(s, c" abcdefg:    %d ".as_ptr(), &[KsPrintfArg::Int(100)]);
    libc::printf(c"'%s'\n".as_ptr(), (*s).s);

    let fields = ksplit(s, 0, &mut n);
    let mut i = 0;
    while i < n {
        libc::printf(
            c"field[%d] = '%s'\n".as_ptr(),
            i,
            (*s).s.add(*fields.add(i as usize) as usize),
        );
        i += 1;
    }

    (*s).l = 0;
    let mut p = kstrtok(c"ab:cde:fg/hij::k".as_ptr(), c":/".as_ptr(), &mut aux);
    while !p.is_null() {
        kputsn(p, aux.p.offset_from(p) as usize, s);
        kputc(b'\n' as c_int, s);
        p = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
    }
    libc::printf(c"%s".as_ptr(), (*s).s);

    libc::free((*s).s.cast());
    libc::free(s.cast());
    libc::free(fields.cast());

    {
        let str_ = c"abcdefgcdgcagtcakcdcd".as_ptr();
        let pat = c"cd".as_ptr();
        let mut s = str_;
        let mut prep: *mut c_int = std::ptr::null_mut();
        loop {
            let ret = kstrstr(s, pat, &mut prep);
            if ret.is_null() {
                break;
            }
            libc::printf(c"match: %s\n".as_ptr(), ret);
            s = ret.add(*prep as usize);
        }
        libc::free(prep.cast());
    }

    0
}

// ----------------------------------------------------------------------
// Functions translated from htslib/kstring.c (moved from src/hts.rs).
// ----------------------------------------------------------------------

pub unsafe fn ksplit_core(
    s: *mut c_char,
    delimiter: c_int,
    _max: *mut c_int,
    _offsets: *mut *mut c_int,
) -> c_int {
    let mut n = 0;
    let mut max = *_max;
    let mut offsets = *_offsets;
    let l = CStr::from_ptr(s).to_bytes().len() as c_int;

    let mut last_char = 0;
    let mut last_start = 0;
    let mut i = 0;
    while i <= l {
        let ch = *s.add(i as usize);
        let signed_ch = ch as c_int;
        let unsigned_ch = (ch as u8) as c_int;
        if delimiter == 0 {
            if libc::isspace(unsigned_ch) != 0 || ch == 0 {
                if libc::isgraph(last_char) != 0 {
                    if !_offsets.is_null() {
                        *s.add(i as usize) = 0;
                        if n == max {
                            max = if max != 0 { max << 1 } else { 2 };
                            let tmp = crate::htslib_rs::c_compat::realloc(
                                offsets.cast(),
                                std::mem::size_of::<c_int>() as u64 * max as u64,
                            )
                            .cast::<c_int>();
                            if tmp.is_null() {
                                crate::htslib_rs::c_compat::free(offsets.cast());
                                *_offsets = std::ptr::null_mut();
                                return 0;
                            }
                            offsets = tmp;
                        }
                        *offsets.add(n as usize) = last_start;
                        n += 1;
                    } else {
                        n += 1;
                    }
                }
            } else if libc::isspace(last_char) != 0 || last_char == 0 {
                last_start = i;
            }
        } else if signed_ch == delimiter || ch == 0 {
            if last_char != 0 && last_char != delimiter {
                if !_offsets.is_null() {
                    *s.add(i as usize) = 0;
                    if n == max {
                        max = if max != 0 { max << 1 } else { 2 };
                        let tmp = crate::htslib_rs::c_compat::realloc(
                            offsets.cast(),
                            std::mem::size_of::<c_int>() as u64 * max as u64,
                        )
                        .cast::<c_int>();
                        if tmp.is_null() {
                            crate::htslib_rs::c_compat::free(offsets.cast());
                            *_offsets = std::ptr::null_mut();
                            return 0;
                        }
                        offsets = tmp;
                    }
                    *offsets.add(n as usize) = last_start;
                    n += 1;
                } else {
                    n += 1;
                }
            }
        } else if last_char == delimiter || last_char == 0 {
            last_start = i;
        }
        last_char = unsigned_ch;
        i += 1;
    }
    *_max = max;
    *_offsets = offsets;
    n
}

pub unsafe fn ksplit(s: *mut kstring_t, delimiter: c_int, n: *mut c_int) -> *mut c_int {
    let mut max = 0;
    let mut offsets: *mut c_int = std::ptr::null_mut();
    *n = ksplit_core((*s).s, delimiter, &mut max, &mut offsets);
    offsets
}

pub unsafe fn kgetline(s: *mut kstring_t, fgets_fn: kgets_func, fp: *mut c_void) -> c_int {
    let l0 = (*s).l;
    while (*s).l == l0 || *(*s).s.add((*s).l - 1) != b'\n' as c_char {
        if (*s).m - (*s).l < 200 {
            if ks_resize(s, (*s).m + 200) < 0 {
                return libc::EOF;
            }
        }
        let ret = fgets_fn.unwrap_unchecked()((*s).s.add((*s).l), ((*s).m - (*s).l) as c_int, fp);
        if ret.is_null() {
            break;
        }
        (*s).l += CStr::from_ptr((*s).s.add((*s).l)).to_bytes().len();
    }

    if (*s).l == l0 {
        return libc::EOF;
    }

    if (*s).l > l0 && *(*s).s.add((*s).l - 1) == b'\n' as c_char {
        (*s).l -= 1;
        if (*s).l > l0 && *(*s).s.add((*s).l - 1) == b'\r' as c_char {
            (*s).l -= 1;
        }
    }
    *(*s).s.add((*s).l) = 0;
    0
}

pub unsafe extern "C" fn fgets_wrapper(
    buffer: *mut c_char,
    size: c_int,
    stream: *mut c_void,
) -> *mut c_char {
    libc::fgets(buffer, size, stream.cast::<libc::FILE>())
}

pub unsafe fn kfgetline(s: *mut kstring_t, fp: *mut libc::FILE) -> c_int {
    if s.is_null() || fp.is_null() {
        return libc::EOF;
    }
    kgetline(s, Some(fgets_wrapper), fp.cast())
}

pub unsafe fn kgetline2(s: *mut kstring_t, fgets_fn: kgets_func2, fp: *mut c_void) -> c_int {
    let l0 = (*s).l;
    while (*s).l == l0 || *(*s).s.add((*s).l - 1) != b'\n' as c_char {
        if (*s).m - (*s).l < 200 {
            if ks_resize(s, (*s).m + 200) < 0 {
                fgets_fn.unwrap_unchecked()((*s).s.add((*s).l), 0, fp);
                return libc::EOF;
            }
        }
        let len = fgets_fn.unwrap_unchecked()((*s).s.add((*s).l), (*s).m - (*s).l, fp);
        if len <= 0 {
            break;
        }
        (*s).l += len as usize;
    }

    if (*s).l == l0 {
        return libc::EOF;
    }

    if (*s).l > l0 && *(*s).s.add((*s).l - 1) == b'\n' as c_char {
        (*s).l -= 1;
        if (*s).l > l0 && *(*s).s.add((*s).l - 1) == b'\r' as c_char {
            (*s).l -= 1;
        }
    }
    *(*s).s.add((*s).l) = 0;
    0
}

pub unsafe fn kstrtok(
    str_: *const c_char,
    sep_in: *const c_char,
    aux: *mut ks_tokaux_t,
) -> *mut c_char {
    let sep = sep_in.cast::<u8>();
    if !sep.is_null() {
        if str_.is_null() && (*aux).finished != 0 {
            return std::ptr::null_mut();
        }
        (*aux).finished = 0;
        if *sep != 0 && *sep.add(1) != 0 {
            (*aux).sep = -1;
            (*aux).tab = [0; 4];
            let mut p = sep;
            while *p != 0 {
                (*aux).tab[(*p >> 6) as usize] |= 1u64 << (*p & 0x3f);
                p = p.add(1);
            }
        } else {
            (*aux).sep = *sep as c_int;
        }
    }
    if (*aux).finished != 0 {
        return std::ptr::null_mut();
    }
    let start = if !str_.is_null() {
        (*aux).finished = 0;
        str_.cast::<u8>()
    } else {
        (*aux).p.add(1).cast::<u8>()
    };

    let p = if (*aux).sep < 0 {
        let mut p = start;
        while *p != 0 {
            if (((*aux).tab[(*p >> 6) as usize] >> (*p & 0x3f)) & 1) != 0 {
                break;
            }
            p = p.add(1);
        }
        p
    } else {
        let p2 = libc::strchr(start.cast::<c_char>(), (*aux).sep).cast::<u8>();
        if p2.is_null() {
            start.add(libc::strlen(start.cast::<c_char>()))
        } else {
            p2
        }
    };
    (*aux).p = p.cast::<c_char>();
    if *p == 0 {
        (*aux).finished = 1;
    }
    start.cast::<c_char>().cast_mut()
}

pub fn fast_exp(mut x: u64, mut n: u64) -> u64 {
    let mut y = 1u64;
    if n == 0 {
        return 1;
    }
    while n > 1 {
        if (n & 1) != 0 {
            y = y.wrapping_mul(x);
        }
        x = x.wrapping_mul(x);
        n >>= 1;
    }
    y.wrapping_mul(x)
}

pub unsafe fn karp_rabin(
    str_: *const c_void,
    n: size_t,
    pat_: *const c_void,
    m: size_t,
) -> *mut c_void {
    let str_ubytes = str_.cast::<u8>();
    let pat = pat_.cast::<u8>();
    let b = 31u64;
    let mut hash_pat = 0u64;
    let mut hash_str = 0u64;
    let b_to_m = fast_exp(b, m as u64);
    let mut mismatch = 0u8;

    if m > n {
        return std::ptr::null_mut();
    }

    let mut i = 0usize;
    while i < m {
        mismatch |= *str_ubytes.add(i) ^ *pat.add(i);
        hash_pat = hash_pat
            .wrapping_mul(b)
            .wrapping_add(*pat.add(i) as u64)
            .wrapping_add(1);
        hash_str = hash_str
            .wrapping_mul(b)
            .wrapping_add(*str_ubytes.add(i) as u64)
            .wrapping_add(1);
        i += 1;
    }

    if mismatch == 0 {
        return str_.cast_mut();
    }

    while i < n {
        hash_str = hash_str
            .wrapping_mul(b)
            .wrapping_add(*str_ubytes.add(i) as u64)
            .wrapping_add(1)
            .wrapping_sub(b_to_m.wrapping_mul((*str_ubytes.add(i - m) as u64).wrapping_add(1)));
        if hash_str == hash_pat
            && libc::memcmp(
                pat.cast(),
                str_ubytes.add(i + 1 - m).cast(),
                m as libc::size_t,
            ) == 0
        {
            return str_ubytes.add(i + 1 - m).cast::<c_void>().cast_mut();
        }
        i += 1;
    }
    std::ptr::null_mut()
}

pub unsafe fn ksBM_prep(pat: *const u8, m: c_int) -> *mut c_int {
    if m < 1 {
        return std::ptr::null_mut();
    }
    let prep =
        crate::htslib_rs::c_compat::calloc(m as u64 + 256, std::mem::size_of::<c_int>() as u64)
            .cast::<c_int>();
    if prep.is_null() {
        return std::ptr::null_mut();
    }
    let bm_gs = prep;
    let bm_bc = prep.add(m as usize);

    let mut i = 0;
    while i < 256 {
        *bm_bc.add(i as usize) = m;
        i += 1;
    }
    i = 0;
    while i < m - 1 {
        *bm_bc.add(*pat.add(i as usize) as usize) = m - i - 1;
        i += 1;
    }

    let suff = crate::htslib_rs::c_compat::calloc(m as u64, std::mem::size_of::<c_int>() as u64)
        .cast::<c_int>();
    if suff.is_null() {
        crate::htslib_rs::c_compat::free(prep.cast());
        return std::ptr::null_mut();
    }

    let mut f = 0;
    *suff.add((m - 1) as usize) = m;
    let mut g = m - 1;
    i = m - 2;
    while i >= 0 {
        if i > g && *suff.add((i + m - 1 - f) as usize) < i - g {
            *suff.add(i as usize) = *suff.add((i + m - 1 - f) as usize);
        } else {
            if i < g {
                g = i;
            }
            f = i;
            while g >= 0 && *pat.add(g as usize) == *pat.add((g + m - 1 - f) as usize) {
                g -= 1;
            }
            *suff.add(i as usize) = f - g;
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }

    let mut j = 0;
    i = 0;
    while i < m {
        *bm_gs.add(i as usize) = m;
        i += 1;
    }
    i = m - 1;
    while i >= 0 {
        if *suff.add(i as usize) == i + 1 {
            while j < m - 1 - i {
                if *bm_gs.add(j as usize) == m {
                    *bm_gs.add(j as usize) = m - 1 - i;
                }
                j += 1;
            }
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    i = 0;
    while i <= m - 2 {
        *bm_gs.add((m - 1 - *suff.add(i as usize)) as usize) = m - 1 - i;
        i += 1;
    }

    crate::htslib_rs::c_compat::free(suff.cast());
    prep
}

pub unsafe fn boyer_moore(
    str_: *const c_void,
    n: size_t,
    pat_: *const c_void,
    m: c_int,
    stored_prep_ptr: *mut *mut c_int,
) -> *mut c_void {
    if str_.is_null() || pat_.is_null() {
        return str_.cast_mut();
    }

    let str_ubytes = str_.cast::<u8>();
    let pat = pat_.cast::<u8>();

    if m <= 0 {
        return str_.cast_mut();
    }
    if n < m as usize {
        return std::ptr::null_mut();
    }
    if m == 1 {
        return libc::memchr(str_, *pat as c_int, n as libc::size_t);
    }

    let prep;
    if !stored_prep_ptr.is_null() && !(*stored_prep_ptr).is_null() {
        prep = *stored_prep_ptr;
    } else {
        prep = ksBM_prep(pat, m);
        if prep.is_null() {
            return karp_rabin(str_, n, pat_, m as usize);
        }
        if !stored_prep_ptr.is_null() {
            *stored_prep_ptr = prep;
        }
    }

    let bm_gs = prep;
    let bm_bc = prep.add(m as usize);
    let mut j = 0usize;
    while j <= n - m as usize {
        let mut i = m - 1;
        while i >= 0 && *pat.add(i as usize) == *str_ubytes.add(i as usize + j) {
            if i == 0 {
                i = -1;
                break;
            }
            i -= 1;
        }
        if i >= 0 {
            let mut max = *bm_bc.add(*str_ubytes.add(i as usize + j) as usize) - m + 1 + i;
            if max < *bm_gs.add(i as usize) {
                max = *bm_gs.add(i as usize);
            }
            j += max as usize;
        } else {
            if stored_prep_ptr.is_null() {
                crate::htslib_rs::c_compat::free(prep.cast());
            }
            return str_ubytes.add(j).cast::<c_void>().cast_mut();
        }
    }

    if stored_prep_ptr.is_null() {
        crate::htslib_rs::c_compat::free(prep.cast());
    }
    std::ptr::null_mut()
}

pub unsafe fn kmemmem(
    str_: *const c_void,
    n: c_int,
    pat: *const c_void,
    m: c_int,
    prep: *mut *mut c_int,
) -> *mut c_void {
    boyer_moore(str_, if n >= 0 { n as usize } else { 0 }, pat, m, prep)
}

pub unsafe fn kstrstr(
    str_: *const c_char,
    pat: *const c_char,
    prep: *mut *mut c_int,
) -> *mut c_char {
    let patlen = libc::strlen(pat);
    if patlen <= c_int::MAX as usize {
        boyer_moore(
            str_.cast(),
            libc::strlen(str_),
            pat.cast(),
            patlen as c_int,
            prep,
        )
        .cast::<c_char>()
    } else {
        karp_rabin(str_.cast(), libc::strlen(str_), pat.cast(), patlen).cast::<c_char>()
    }
}

pub unsafe fn kstrnstr(
    str_: *const c_char,
    pat: *const c_char,
    mut n: c_int,
    prep: *mut *mut c_int,
) -> *mut c_char {
    if pat.is_null() || *pat == 0 {
        return str_.cast_mut();
    }
    if n <= 0 {
        return std::ptr::null_mut();
    }
    let endp = libc::memchr(str_.cast(), 0, n as libc::size_t).cast::<c_char>();
    if !endp.is_null() && endp.offset_from(str_) < n as isize {
        n = endp.offset_from(str_) as c_int;
    }
    let patlen = libc::strlen(pat);
    if patlen > n as usize {
        return std::ptr::null_mut();
    }
    boyer_moore(str_.cast(), n as usize, pat.cast(), patlen as c_int, prep).cast::<c_char>()
}

pub unsafe fn kputd(d: f64, s: *mut kstring_t) -> c_int {
    if d == 0.0 {
        if d.is_sign_negative() {
            return kputsn(b"-0\0".as_ptr().cast(), 2, s);
        }
        return kputsn(b"0\0".as_ptr().cast(), 1, s);
    }

    let mut d = d;
    let mut len = 0;
    if d < 0.0 {
        if kputc(b'-' as c_int, s) < 0 {
            return -1;
        }
        len = 1;
        d = -d;
    }

    if !(0.0001..=999999.0).contains(&d) {
        if ks_resize(s, (*s).l + 50) < 0 {
            return libc::EOF;
        }
        // We let stdio handle the exponent cases
        let s2 = libc::snprintf(
            (*s).s.add((*s).l),
            ((*s).m - (*s).l) as libc::size_t,
            b"%g\0".as_ptr().cast(),
            d,
        );
        len += s2;
        (*s).l += s2 as size_t;
        return len;
    }

    let decimals = if d < 0.001 {
        9
    } else if d < 0.01 {
        8
    } else if d < 0.1 {
        7
    } else if d < 1.0 {
        6
    } else if d < 10.0 {
        5
    } else if d < 100.0 {
        4
    } else if d < 1000.0 {
        3
    } else if d < 10000.0 {
        2
    } else if d < 100000.0 {
        1
    } else {
        0
    };
    let text = format!("{d:.decimals$}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if kputsn(text.as_ptr().cast(), text.len(), s) < 0 {
        return -1;
    }
    len + text.len() as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn ksprintf_allocates_and_appends_to_kstring() {
        unsafe {
            let mut s: kstring_t = std::mem::zeroed();

            assert_eq!(
                kstring_c_177_ksprintf(&mut s, c"sample-%d".as_ptr(), &[KsPrintfArg::Int(42)]),
                9
            );
            assert!(!s.s.is_null());
            assert_eq!(s.l, 9);
            assert_eq!(CStr::from_ptr(s.s).to_bytes(), b"sample-42");

            assert_eq!(
                kstring_c_177_ksprintf(
                    &mut s,
                    c":%s".as_ptr(),
                    &[KsPrintfArg::Str(c"ok".as_ptr())],
                ),
                3
            );
            assert_eq!(s.l, 12);
            assert_eq!(CStr::from_ptr(s.s).to_bytes(), b"sample-42:ok");

            libc::free(s.s.cast());
        }
    }
}
