use crate::htslib_rs::hts::kstring_t;

// original: lookup (htslib/test/test_expr.c:31)
// NOTE: this callback is invoked through the still-raw production
// `hts_expr_sym_func` type, which passes `*mut c_char`/raw pointers across the
// FFI boundary; the raw-pointer parameters are retained until that type is
// converted in parallel.
pub unsafe extern "C" fn test_test_expr_c_31_lookup(
    _data: *mut std::ffi::c_void,
    str_: *mut std::ffi::c_char,
    end: *mut *mut std::ffi::c_char,
    res: *mut crate::htslib_rs::hts::hts_expr_val_t,
) -> i32 {
    let Some(res_ref) = res.as_mut() else {
        return -1;
    };
    (*res).is_str = 0;
    if libc::strncmp(str_.cast(), c"foo".as_ptr(), 3) == 0 {
        *end = str_.add(3);
        (*res).d = 15551.0;
    } else if *str_ == b'a' as std::ffi::c_char {
        *end = str_.add(1);
        (*res).d = 1.0;
    } else if *str_ == b'b' as std::ffi::c_char {
        *end = str_.add(1);
        (*res).d = 2.0;
    } else if *str_ == b'c' as std::ffi::c_char {
        *end = str_.add(1);
        (*res).d = 3.0;
    } else if libc::strncmp(str_.cast(), c"magic".as_ptr(), 5) == 0 {
        *end = str_.add(5);
        (*res).is_str = 1;
        crate::htslib_rs::hts::ks_clear(&mut (*res).s);
        crate::htslib_rs::hts::kputs(b"plugh", &mut (*res).s);
    } else if libc::strncmp(str_.cast(), c"empty-but-true".as_ptr(), 14) == 0 {
        *end = str_.add(14);
        (*res).is_true = 1;
        (*res).is_str = 1;
        crate::htslib_rs::hts::ks_clear(&mut (*res).s);
        crate::htslib_rs::hts::kputs(b"", &mut (*res).s);
    } else if libc::strncmp(str_.cast(), c"empty".as_ptr(), 5) == 0 {
        *end = str_.add(5);
        (*res).is_str = 1;
        crate::htslib_rs::hts::ks_clear(&mut (*res).s);
        crate::htslib_rs::hts::kputs(b"", &mut (*res).s);
    } else if libc::strncmp(str_.cast(), c"zero-but-true".as_ptr(), 13) == 0 {
        *end = str_.add(13);
        (*res).d = 0.0;
        (*res).is_true = 1;
    } else if libc::strncmp(str_.cast(), c"null-but-true".as_ptr(), 13) == 0 {
        *end = str_.add(13);
        crate::htslib_rs::hts::hts_expr_val_undef(res_ref);
        (*res).is_true = 1;
    } else if libc::strncmp(str_.cast(), c"null".as_ptr(), 4) == 0 {
        *end = str_.add(4);
        crate::htslib_rs::hts::hts_expr_val_undef(res_ref);
    } else if libc::strncmp(str_.cast(), c"nan".as_ptr(), 3) == 0 {
        *end = str_.add(3);
        crate::htslib_rs::hts::hts_expr_val_undef(res_ref);
    } else {
        return -1;
    }

    0
}

// original: strcmpnull (htslib/test/test_expr.c:97)
// Retains raw `*const u8` pointers: the test intent is to distinguish NULL from
// non-NULL string values, which an owned `&[u8]` cannot express.
pub unsafe fn test_test_expr_c_97_strcmpnull(a: *const u8, b: *const u8) -> i32 {
    if a.is_null() && b.is_null() {
        return 0;
    }
    if a.is_null() {
        return -1;
    }
    if b.is_null() {
        return 1;
    }
    libc::strcmp(a.cast(), b.cast())
}

// original: cmpfloat (htslib/test/test_expr.c:105)
pub unsafe fn test_test_expr_c_105_cmpfloat(d1: f64, d2: f64) -> i32 {
    (d1 == d2 || (d1.is_nan() && d2.is_nan())) as i32
}

// original: test (htslib/test/test_expr.c:110)
pub unsafe fn test_test_expr_c_110_test() -> i32 {
    struct TestEv {
        truth_val: i32,
        dval: f64,
        sval: *const u8,
        // NUL-terminated expression bytes; the trailing 0 is retained because
        // the production parser walks the buffer via a cursor.
        str_: &'static [u8],
    }

    let tests = [
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"+1\0",
        },
        TestEv {
            truth_val: 1,
            dval: -1.0,
            sval: std::ptr::null(),
            str_: b"-1\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"!7\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!0\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!(!7)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!!7\0",
        },
        TestEv {
            truth_val: 1,
            dval: 5.0,
            sval: std::ptr::null(),
            str_: b"2+3\0",
        },
        TestEv {
            truth_val: 1,
            dval: -1.0,
            sval: std::ptr::null(),
            str_: b"2+-3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 6.0,
            sval: std::ptr::null(),
            str_: b"1+2+3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"-2+3\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"1+null\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null-1\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"-null\0",
        },
        TestEv {
            truth_val: 1,
            dval: 6.0,
            sval: std::ptr::null(),
            str_: b"2*3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 6.0,
            sval: std::ptr::null(),
            str_: b"1*2*3\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"2*0\0",
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: b"(7)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: b"((7))\0",
        },
        TestEv {
            truth_val: 1,
            dval: 21.0,
            sval: std::ptr::null(),
            str_: b"(1+2)*(3+4)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 14.0,
            sval: std::ptr::null(),
            str_: b"(4*5)-(-2*-3)\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"2*null\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null/2\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"0/0\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"(1+2)*3==9\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"(1+2)*3!=8\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"(1+2)*3!=9\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"(1+2)*3==8\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"1>2\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1<2\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"3<3\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"3>3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"9<=9\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"9>=9\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"2*4==8\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"16==0x10\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"15<0x10\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"17>0x10\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"2*4!=8\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"4+2<3+4\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"4*2<3+4\0",
        },
        TestEv {
            truth_val: 1,
            dval: 8.0,
            sval: std::ptr::null(),
            str_: b"4*(2<3)+4\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"(1<2) == (3>2)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1<2 == 3>2\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null <= 0\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null >= 0\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null < 0\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null > 0\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null == null\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null != null\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null < 10\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"10 > null\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"2 && 1\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"2 && 0\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"0 && 2\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"2 || 1\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"2 || 0\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"0 || 2\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1 || 2 && 3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"2 && 3 || 1\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"0 && 3 || 2\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"0 && 3 || 0\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b" 5 - 5 && 1\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"+5 - 5 && 1\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"null && 1\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"1 && null\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!null && 1\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1 && !null\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1 && null-but-true\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"null || 0\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"0 || null\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!null || 0\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"0 || !null\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"0 || null-but-true\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"null || 1\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1 || null\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"3 & 1\0",
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: b"3 & 2\0",
        },
        TestEv {
            truth_val: 1,
            dval: 3.0,
            sval: std::ptr::null(),
            str_: b"1 | 2\0",
        },
        TestEv {
            truth_val: 1,
            dval: 3.0,
            sval: std::ptr::null(),
            str_: b"1 | 3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: b"1 | 6\0",
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: b"1 ^ 3\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"1 | null\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null | 1\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"1 & null\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null & 1\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"0 ^ null\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null ^ 0\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"1 ^ null\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null ^ 1\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"(1^0)&(4^3)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: b"1 ^(0&4)^ 3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: b"1 ^ 0&4 ^ 3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 6.0,
            sval: std::ptr::null(),
            str_: b"(1|0)^(4|3)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: b"1 |(0^4)| 3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: b"1 | 0^4 | 3\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"4 & 2 || 1\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"(4 & 2) || 1\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"4 & (2 || 1)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1 || 4 & 2\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1 || (4 & 2)\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"(1 || 4) & 2\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b" (2*3)&7  > 4\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b" (2*3)&(7 > 4)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"((2*3)&7) > 4\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"((2*3)&7) > 4 && 2*2 <= 4\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: b"plugh\0".as_ptr(),
            str_: b"magic\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: b"\0".as_ptr(),
            str_: b"empty\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"magic == \"plugh\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"magic != \"xyzzy\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"\"abc\" < \"def\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"\"abc\" <= \"abc\"\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"\"abc\" < \"ab\"\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"\"abc\" <= \"ab\"\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"\"abc\" > \"def\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"\"abc\" >= \"abc\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"\"abc\" > \"ab\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"\"abc\" >= \"ab\"\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null == \"x\"\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null != \"x\"\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null < \"x\"\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null > \"x\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"\"abbc\" =~ \"^a+b+c+$\"\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"\"aBBc\" =~ \"^a+b+c+$\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"\"aBBc\" !~ \"^a+b+c+$\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"\"xyzzy plugh abracadabra\" =~ magic\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: b"\0".as_ptr(),
            str_: b"empty-but-true\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"!empty-but-true\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!!empty-but-true\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"1 && empty-but-true && 1\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"1 && empty-but-true && 0\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!null\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"!!null\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"!\"foo\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!!\"foo\"\0",
        },
        TestEv {
            truth_val: 1,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"null-but-true\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"!null-but-true\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!!null-but-true\0",
        },
        TestEv {
            truth_val: 1,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"zero-but-true\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"!zero-but-true\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"!!zero-but-true\0",
        },
        TestEv {
            truth_val: 1,
            dval: 2.0f64.ln(),
            sval: std::ptr::null(),
            str_: b"log(2)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 9.0f64.exp(),
            sval: std::ptr::null(),
            str_: b"exp(9)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 9.0,
            sval: std::ptr::null(),
            str_: b"log(exp(9))\0",
        },
        TestEv {
            truth_val: 1,
            dval: 8.0,
            sval: std::ptr::null(),
            str_: b"pow(2,3)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 3.0,
            sval: std::ptr::null(),
            str_: b"sqrt(9)\0",
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"sqrt(-9)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: b"default(2,3)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 3.0,
            sval: std::ptr::null(),
            str_: b"default(null,3)\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"default(null,0)\0",
        },
        TestEv {
            truth_val: 1,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"default(null-but-true,0)\0",
        },
        TestEv {
            truth_val: 1,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"default(null-but-true,null)\0",
        },
        TestEv {
            truth_val: 1,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: b"default(null,null-but-true)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"exists(\"foo\")\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"exists(12)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"exists(\"\")\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"exists(0)\0",
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: b"exists(null)\0",
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: b"exists(null-but-true)\0",
        },
    ];

    let mut res = 0;
    let mut r = crate::htslib_rs::hts::hts_expr_val_t {
        is_true: 0,
        is_str: 0,
        s: kstring_t { data: Vec::new() },
        d: 0.0,
    };
    for test in tests.iter() {
        let filt = crate::htslib_rs::hts::hts_filter_init(test.str_);
        if filt.is_null() {
            return 1;
        }
        if crate::htslib_rs::hts::hts_filter_eval2(
            &mut *filt,
            std::ptr::null_mut(),
            Some(test_test_expr_c_31_lookup),
            &mut r,
        ) != 0
        {
            eprintln!(
                "Failed to parse filter string {}",
                String::from_utf8_lossy(&test.str_[..test.str_.len() - 1])
            );
            res = 1;
            crate::htslib_rs::hts::hts_filter_free(filt);
            continue;
        }

        let mut r_s_cstr = r.s.data.clone();
        r_s_cstr.push(0);
        if crate::htslib_rs::hts::hts_expr_val_exists(&mut r) == 0 {
            if r.is_true as i32 != test.truth_val
                || test_test_expr_c_105_cmpfloat(r.d, test.dval) == 0
            {
                eprintln!(
                    "Failed test: \"{}\" == \"{}\", got {}, \"{}\", {}",
                    String::from_utf8_lossy(&test.str_[..test.str_.len() - 1]),
                    test.dval,
                    if r.is_true != 0 { "true" } else { "false" },
                    String::from_utf8_lossy(&r.s.data),
                    r.d,
                );
                res = 1;
            }
        } else if r.is_str != 0
            && (test_test_expr_c_97_strcmpnull(r_s_cstr.as_ptr(), test.sval) != 0
                || test_test_expr_c_105_cmpfloat(r.d, test.dval) == 0
                || r.is_true as i32 != test.truth_val)
        {
            eprintln!(
                "Failed test: \"{}\" == \"{}\", got {}, \"{}\", {}",
                String::from_utf8_lossy(&test.str_[..test.str_.len() - 1]),
                if test.sval.is_null() {
                    String::from("(null)")
                } else {
                    String::from_utf8_lossy(
                        std::ffi::CStr::from_ptr(test.sval.cast()).to_bytes(),
                    )
                    .into_owned()
                },
                if r.is_true != 0 { "true" } else { "false" },
                String::from_utf8_lossy(&r.s.data),
                r.d,
            );
            res = 1;
        } else if r.is_str == 0
            && (test_test_expr_c_105_cmpfloat(r.d, test.dval) == 0
                || r.is_true as i32 != test.truth_val)
        {
            eprintln!(
                "Failed test: {} == {}, got {}, {}",
                String::from_utf8_lossy(&test.str_[..test.str_.len() - 1]),
                test.dval,
                if r.is_true != 0 { "true" } else { "false" },
                r.d,
            );
            res = 1;
        }

        crate::htslib_rs::hts::hts_expr_val_free(&mut r);
        crate::htslib_rs::hts::hts_filter_free(filt);
    }

    res
}

// original: main (htslib/test/test_expr.c:346)
// NOTE: argc/argv retain raw pointers at the program-entry boundary; argv[1]
// is a C NUL-terminated string that is read into owned bytes (with its trailing
// 0 retained) before being handed to the production parser, which walks it.
pub unsafe fn test_test_expr_c_346_main(argc: i32, argv: *mut *mut u8) -> i32 {
    if argc > 1 {
        let mut v = crate::htslib_rs::hts::hts_expr_val_t {
            is_true: 0,
            is_str: 0,
            s: kstring_t { data: Vec::new() },
            d: 0.0,
        };
        let mut arg = std::ffi::CStr::from_ptr((*argv.add(1)).cast()).to_bytes().to_vec();
        arg.push(0);
        let filt = crate::htslib_rs::hts::hts_filter_init(&arg);
        if crate::htslib_rs::hts::hts_filter_eval2(
            &mut *filt,
            std::ptr::null_mut(),
            Some(test_test_expr_c_31_lookup),
            &mut v,
        ) != 0
        {
            return 1;
        }

        print!("{}\t", if v.is_true != 0 { "true" } else { "false" });

        if v.is_str != 0 {
            println!("{}", String::from_utf8_lossy(&v.s.data));
        } else {
            println!("{}", v.d);
        }

        crate::htslib_rs::hts::hts_expr_val_free(&mut v);
        crate::htslib_rs::hts::hts_filter_free(filt);
        return 0;
    }

    test_test_expr_c_110_test()
}
