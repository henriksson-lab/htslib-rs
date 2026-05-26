use crate::htslib_rs::hts::{
    kputc, kputd, kputsn, ks_resize, ks_tokaux_t, ksplit, kstring_t, kstrstr, kstrtok,
};
use std::ffi::{c_char, c_int, CStr};

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

    let mut l = hts_sys::vsnprintf(
        (*s).s.add((*s).l),
        ((*s).m - (*s).l) as u64,
        fmt,
        (&mut args as *mut crate::htslib_rs::c_compat::__va_list_tag).cast(),
    );
    if l + 1 > ((*s).m - (*s).l) as c_int {
        if ks_resize(s, (*s).l + l as usize + 2) < 0 {
            return -1;
        }
        let mut args = std::mem::MaybeUninit::<crate::htslib_rs::c_compat::__va_list_tag>::uninit();
        std::ptr::copy_nonoverlapping(ap, args.as_mut_ptr(), 1);
        let mut args = args.assume_init();
        l = hts_sys::vsnprintf(
            (*s).s.add((*s).l),
            ((*s).m - (*s).l) as u64,
            fmt,
            (&mut args as *mut crate::htslib_rs::c_compat::__va_list_tag).cast(),
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
