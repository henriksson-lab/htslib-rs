use crate::htslib_mini_rs::hts::kstring_t;
use std::ffi::{c_char, c_int, c_void};

// original: lookup (htslib/test/test_expr.c:31)
pub unsafe extern "C" fn test_test_expr_c_31_lookup(
    _data: *mut c_void,
    str_: *mut c_char,
    end: *mut *mut c_char,
    res: *mut crate::htslib_mini_rs::hts::hts_expr_val_t,
) -> c_int {
    (*res).is_str = 0;
    if libc::strncmp(str_, c"foo".as_ptr(), 3) == 0 {
        *end = str_.add(3);
        (*res).d = 15551.0;
    } else if *str_ == b'a' as c_char {
        *end = str_.add(1);
        (*res).d = 1.0;
    } else if *str_ == b'b' as c_char {
        *end = str_.add(1);
        (*res).d = 2.0;
    } else if *str_ == b'c' as c_char {
        *end = str_.add(1);
        (*res).d = 3.0;
    } else if libc::strncmp(str_, c"magic".as_ptr(), 5) == 0 {
        *end = str_.add(5);
        (*res).is_str = 1;
        crate::htslib_mini_rs::hts::kputs(
            c"plugh".as_ptr(),
            crate::htslib_mini_rs::hts::ks_clear(&mut (*res).s),
        );
    } else if libc::strncmp(str_, c"empty-but-true".as_ptr(), 14) == 0 {
        *end = str_.add(14);
        (*res).is_true = 1;
        (*res).is_str = 1;
        crate::htslib_mini_rs::hts::kputs(
            c"".as_ptr(),
            crate::htslib_mini_rs::hts::ks_clear(&mut (*res).s),
        );
    } else if libc::strncmp(str_, c"empty".as_ptr(), 5) == 0 {
        *end = str_.add(5);
        (*res).is_str = 1;
        crate::htslib_mini_rs::hts::kputs(
            c"".as_ptr(),
            crate::htslib_mini_rs::hts::ks_clear(&mut (*res).s),
        );
    } else if libc::strncmp(str_, c"zero-but-true".as_ptr(), 13) == 0 {
        *end = str_.add(13);
        (*res).d = 0.0;
        (*res).is_true = 1;
    } else if libc::strncmp(str_, c"null-but-true".as_ptr(), 13) == 0 {
        *end = str_.add(13);
        crate::htslib_mini_rs::hts::hts_expr_val_undef(res);
        (*res).is_true = 1;
    } else if libc::strncmp(str_, c"null".as_ptr(), 4) == 0 {
        *end = str_.add(4);
        crate::htslib_mini_rs::hts::hts_expr_val_undef(res);
    } else if libc::strncmp(str_, c"nan".as_ptr(), 3) == 0 {
        *end = str_.add(3);
        crate::htslib_mini_rs::hts::hts_expr_val_undef(res);
    } else {
        return -1;
    }

    0
}

// original: strcmpnull (htslib/test/test_expr.c:97)
pub unsafe fn test_test_expr_c_97_strcmpnull(a: *const c_char, b: *const c_char) -> c_int {
    if a.is_null() && b.is_null() {
        return 0;
    }
    if a.is_null() {
        return -1;
    }
    if b.is_null() {
        return 1;
    }
    libc::strcmp(a, b)
}

// original: cmpfloat (htslib/test/test_expr.c:105)
pub unsafe fn test_test_expr_c_105_cmpfloat(d1: f64, d2: f64) -> c_int {
    (d1 == d2 || (d1.is_nan() && d2.is_nan())) as c_int
}

// original: test (htslib/test/test_expr.c:110)
pub unsafe fn test_test_expr_c_110_test() -> c_int {
    struct TestEv {
        truth_val: c_int,
        dval: f64,
        sval: *const c_char,
        str_: *const c_char,
    }

    let tests = [
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"+1".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: -1.0,
            sval: std::ptr::null(),
            str_: c"-1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"!7".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!0".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!(!7)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!!7".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 5.0,
            sval: std::ptr::null(),
            str_: c"2+3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: -1.0,
            sval: std::ptr::null(),
            str_: c"2+-3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 6.0,
            sval: std::ptr::null(),
            str_: c"1+2+3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"-2+3".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"1+null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null-1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"-null".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 6.0,
            sval: std::ptr::null(),
            str_: c"2*3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 6.0,
            sval: std::ptr::null(),
            str_: c"1*2*3".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"2*0".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: c"(7)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: c"((7))".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 21.0,
            sval: std::ptr::null(),
            str_: c"(1+2)*(3+4)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 14.0,
            sval: std::ptr::null(),
            str_: c"(4*5)-(-2*-3)".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"2*null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null/2".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"0/0".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"(1+2)*3==9".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"(1+2)*3!=8".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"(1+2)*3!=9".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"(1+2)*3==8".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"1>2".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1<2".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"3<3".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"3>3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"9<=9".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"9>=9".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"2*4==8".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"16==0x10".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"15<0x10".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"17>0x10".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"2*4!=8".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"4+2<3+4".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"4*2<3+4".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 8.0,
            sval: std::ptr::null(),
            str_: c"4*(2<3)+4".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"(1<2) == (3>2)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1<2 == 3>2".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null <= 0".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null >= 0".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null < 0".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null > 0".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null == null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null != null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null < 10".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"10 > null".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"2 && 1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"2 && 0".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"0 && 2".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"2 || 1".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"2 || 0".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"0 || 2".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1 || 2 && 3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"2 && 3 || 1".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"0 && 3 || 2".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"0 && 3 || 0".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c" 5 - 5 && 1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"+5 - 5 && 1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"null && 1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"1 && null".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!null && 1".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1 && !null".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1 && null-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"null || 0".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"0 || null".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!null || 0".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"0 || !null".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"0 || null-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"null || 1".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1 || null".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"3 & 1".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: c"3 & 2".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 3.0,
            sval: std::ptr::null(),
            str_: c"1 | 2".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 3.0,
            sval: std::ptr::null(),
            str_: c"1 | 3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: c"1 | 6".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: c"1 ^ 3".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"1 | null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null | 1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"1 & null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null & 1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"0 ^ null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null ^ 0".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"1 ^ null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null ^ 1".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"(1^0)&(4^3)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: c"1 ^(0&4)^ 3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: c"1 ^ 0&4 ^ 3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 6.0,
            sval: std::ptr::null(),
            str_: c"(1|0)^(4|3)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: c"1 |(0^4)| 3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 7.0,
            sval: std::ptr::null(),
            str_: c"1 | 0^4 | 3".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"4 & 2 || 1".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"(4 & 2) || 1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"4 & (2 || 1)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1 || 4 & 2".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1 || (4 & 2)".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"(1 || 4) & 2".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c" (2*3)&7  > 4".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c" (2*3)&(7 > 4)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"((2*3)&7) > 4".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"((2*3)&7) > 4 && 2*2 <= 4".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: c"plugh".as_ptr(),
            str_: c"magic".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: c"".as_ptr(),
            str_: c"empty".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"magic == \"plugh\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"magic != \"xyzzy\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"\"abc\" < \"def\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"\"abc\" <= \"abc\"".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"\"abc\" < \"ab\"".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"\"abc\" <= \"ab\"".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"\"abc\" > \"def\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"\"abc\" >= \"abc\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"\"abc\" > \"ab\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"\"abc\" >= \"ab\"".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null == \"x\"".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null != \"x\"".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null < \"x\"".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null > \"x\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"\"abbc\" =~ \"^a+b+c+$\"".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"\"aBBc\" =~ \"^a+b+c+$\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"\"aBBc\" !~ \"^a+b+c+$\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"\"xyzzy plugh abracadabra\" =~ magic".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: c"".as_ptr(),
            str_: c"empty-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"!empty-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!!empty-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"1 && empty-but-true && 1".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"1 && empty-but-true && 0".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"!!null".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"!\"foo\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!!\"foo\"".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"null-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"!null-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!!null-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"zero-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"!zero-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"!!zero-but-true".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 2.0f64.ln(),
            sval: std::ptr::null(),
            str_: c"log(2)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 9.0f64.exp(),
            sval: std::ptr::null(),
            str_: c"exp(9)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 9.0,
            sval: std::ptr::null(),
            str_: c"log(exp(9))".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 8.0,
            sval: std::ptr::null(),
            str_: c"pow(2,3)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 3.0,
            sval: std::ptr::null(),
            str_: c"sqrt(9)".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"sqrt(-9)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 2.0,
            sval: std::ptr::null(),
            str_: c"default(2,3)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 3.0,
            sval: std::ptr::null(),
            str_: c"default(null,3)".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"default(null,0)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"default(null-but-true,0)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"default(null-but-true,null)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: f64::NAN,
            sval: std::ptr::null(),
            str_: c"default(null,null-but-true)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"exists(\"foo\")".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"exists(12)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"exists(\"\")".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"exists(0)".as_ptr(),
        },
        TestEv {
            truth_val: 0,
            dval: 0.0,
            sval: std::ptr::null(),
            str_: c"exists(null)".as_ptr(),
        },
        TestEv {
            truth_val: 1,
            dval: 1.0,
            sval: std::ptr::null(),
            str_: c"exists(null-but-true)".as_ptr(),
        },
    ];

    let mut res = 0;
    let mut r = crate::htslib_mini_rs::hts::hts_expr_val_t {
        is_true: 0,
        is_str: 0,
        s: kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        },
        d: 0.0,
    };
    for test in tests.iter() {
        let filt = crate::htslib_mini_rs::hts::hts_filter_init(test.str_);
        if filt.is_null() {
            return 1;
        }
        if crate::htslib_mini_rs::hts::hts_filter_eval2(
            filt,
            std::ptr::null_mut(),
            Some(test_test_expr_c_31_lookup),
            &mut r,
        ) != 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Failed to parse filter string %s\n".as_ptr(),
                test.str_,
            );
            res = 1;
            crate::htslib_mini_rs::hts::hts_filter_free(filt);
            continue;
        }

        if crate::htslib_mini_rs::hts::hts_expr_val_exists(&mut r) == 0 {
            if r.is_true as c_int != test.truth_val
                || test_test_expr_c_105_cmpfloat(r.d, test.dval) == 0
            {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Failed test: \"%s\" == \"%f\", got %s, \"%s\", %f\n".as_ptr(),
                    test.str_,
                    test.dval,
                    if r.is_true != 0 {
                        c"true".as_ptr()
                    } else {
                        c"false".as_ptr()
                    },
                    r.s.s,
                    r.d,
                );
                res = 1;
            }
        } else if r.is_str != 0
            && (test_test_expr_c_97_strcmpnull(r.s.s, test.sval) != 0
                || test_test_expr_c_105_cmpfloat(r.d, test.dval) == 0
                || r.is_true as c_int != test.truth_val)
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Failed test: \"%s\" == \"%s\", got %s, \"%s\", %f\n".as_ptr(),
                test.str_,
                test.sval,
                if r.is_true != 0 {
                    c"true".as_ptr()
                } else {
                    c"false".as_ptr()
                },
                r.s.s,
                r.d,
            );
            res = 1;
        } else if r.is_str == 0
            && (test_test_expr_c_105_cmpfloat(r.d, test.dval) == 0
                || r.is_true as c_int != test.truth_val)
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Failed test: %s == %f, got %s, %f\n".as_ptr(),
                test.str_,
                test.dval,
                if r.is_true != 0 {
                    c"true".as_ptr()
                } else {
                    c"false".as_ptr()
                },
                r.d,
            );
            res = 1;
        }

        crate::htslib_mini_rs::hts::hts_expr_val_free(&mut r);
        crate::htslib_mini_rs::hts::hts_filter_free(filt);
    }

    res
}

// original: main (htslib/test/test_expr.c:346)
pub unsafe fn test_test_expr_c_346_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    if argc > 1 {
        let mut v = crate::htslib_mini_rs::hts::hts_expr_val_t {
            is_true: 0,
            is_str: 0,
            s: kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
            d: 0.0,
        };
        let filt = crate::htslib_mini_rs::hts::hts_filter_init(*argv.add(1));
        if crate::htslib_mini_rs::hts::hts_filter_eval2(
            filt,
            std::ptr::null_mut(),
            Some(test_test_expr_c_31_lookup),
            &mut v,
        ) != 0
        {
            return 1;
        }

        libc::printf(
            c"%s\t".as_ptr(),
            if v.is_true != 0 {
                c"true".as_ptr()
            } else {
                c"false".as_ptr()
            },
        );

        if v.is_str != 0 {
            libc::puts(v.s.s);
        } else {
            libc::printf(c"%g\n".as_ptr(), v.d);
        }

        crate::htslib_mini_rs::hts::hts_expr_val_free(&mut v);
        crate::htslib_mini_rs::hts::hts_filter_free(filt);
        return 0;
    }

    test_test_expr_c_110_test()
}
