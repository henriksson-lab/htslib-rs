// Functions translated from htslib/hts_expr.c.
// Extracted from src/hts.rs (2026-06-01).

use std::ffi::{c_char, c_int, c_void};

use crate::htslib_rs::hts::{
    hts_expr_sym_func, hts_log_cstr, hts_str2dbl, kputsn, ks_clear, ks_free, kstring_t, ws,
    HTS_LOG_ERROR,
};

#[derive(Clone, Default)]
pub struct hts_expr_val_t {
    pub is_str: c_char,
    pub is_true: c_char,
    pub s: kstring_t,
    pub d: f64,
}

pub struct hts_filter_t {
    // NUL-terminated expression text. Carried as owned bytes; the trailing 0
    // is retained because the recursive-descent parser walks it via a cursor.
    pub expr: Vec<u8>,
    pub parsed: bool,
    pub curr_regex: usize,
    pub max_regex: usize,
    pub preg: Vec<crate::htslib_rs::c_compat::regex_t>,
}

const MAX_REGEX: usize = 10;

fn expr_val_exists(v: &hts_expr_val_t) -> bool {
    // Mirror C hts_expr_val_exists (htslib/hts_expr.h:65): a string value exists
    // iff its kstring buffer was actually allocated (`s.s != NULL`), NOT merely
    // non-empty. An explicitly-set empty string therefore exists, while an
    // absent value (never written) does not. We use the Vec's capacity as the
    // `s.s != NULL` proxy, since kputs/kputsn force a non-zero capacity.
    !((v.is_str == 1 && v.s.data.capacity() == 0) || (v.is_str == 0 && v.d.is_nan()))
}

fn expr_val_exists_true(v: &hts_expr_val_t) -> bool {
    v.is_true != 0 || expr_val_exists(v)
}

fn expr_val_undef(v: &mut hts_expr_val_t) {
    ks_clear(&mut v.s);
    v.is_true = 0;
    v.is_str = 0;
    v.d = f64::NAN;
}

fn expr_val_free(v: &mut hts_expr_val_t) {
    ks_free(&mut v.s);
}

pub fn hts_expr_val_exists(v: &hts_expr_val_t) -> c_int {
    expr_val_exists(v) as c_int
}

pub fn hts_expr_val_existsT(v: &hts_expr_val_t) -> c_int {
    expr_val_exists_true(v) as c_int
}

pub fn hts_expr_val_undef(v: &mut hts_expr_val_t) {
    expr_val_undef(v);
}

pub fn hts_expr_val_free(f: &mut hts_expr_val_t) {
    expr_val_free(f);
}

pub unsafe fn expr_func_length(res: &mut hts_expr_val_t) -> c_int {
    if res.is_str == 0 {
        return -1;
    }
    res.is_str = 0;
    res.d = res.s.data.len() as f64;
    0
}

pub unsafe fn expr_func_min(res: &mut hts_expr_val_t) -> c_int {
    if res.is_str == 0 {
        return -1;
    }
    let mut v = c_int::MAX;
    for l in 0..res.s.data.len() {
        if v > res.s.data[l] as c_int {
            v = res.s.data[l] as c_int;
        }
    }
    res.is_str = 0;
    res.d = if v == c_int::MAX { f64::NAN } else { v as f64 };
    0
}

pub unsafe fn expr_func_max(res: &mut hts_expr_val_t) -> c_int {
    if res.is_str == 0 {
        return -1;
    }
    let mut v = c_int::MIN;
    for l in 0..res.s.data.len() {
        if v < res.s.data[l] as c_int {
            v = res.s.data[l] as c_int;
        }
    }
    res.is_str = 0;
    res.d = if v == c_int::MIN { f64::NAN } else { v as f64 };
    0
}

pub unsafe fn expr_func_avg(res: &mut hts_expr_val_t) -> c_int {
    if res.is_str == 0 {
        return -1;
    }
    let mut v = 0.0;
    let mut l = 0usize;
    while l < res.s.data.len() {
        v += res.s.data[l] as f64;
        l += 1;
    }
    if l != 0 {
        v /= l as f64;
    }
    res.is_str = 0;
    res.d = v;
    0
}

pub fn expr_val_init() -> hts_expr_val_t {
    hts_expr_val_t {
        is_str: 0,
        is_true: 0,
        s: kstring_t::default(),
        d: 0.0,
    }
}

unsafe fn c_bool(v: bool) -> c_char {
    v as c_int as c_char
}

unsafe fn c_prefix_matches(s: *const c_char, lit: &[u8]) -> bool {
    if s.is_null() {
        return false;
    }
    for (i, &expected) in lit.iter().enumerate() {
        let byte = *s.add(i) as u8;
        if byte == 0 || byte != expected {
            return false;
        }
    }
    true
}

unsafe fn expr_value_bytes(v: &hts_expr_val_t) -> Option<&[u8]> {
    // C compares with `res->s.s && val.s.s ? strcmp(...) : ...`, treating a NULL
    // buffer as "no string". An allocated empty string still compares (strcmp on
    // ""). Use capacity as the `s.s != NULL` proxy rather than emptiness.
    if v.is_str == 0 || v.s.data.capacity() == 0 {
        None
    } else {
        Some(&v.s.data)
    }
}

// original: func_expr (htslib/hts_expr.c:154)
pub unsafe fn func_expr(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    let mut func_ok = -1;
    match *str_ as u8 {
        b'a' => {
            if c_prefix_matches(str_, b"avg(") {
                if expression(filt, data, sym_func, str_.add(4), end, res) != 0 {
                    return -1;
                }
                func_ok = expr_func_avg(res);
            }
        }
        b'd' => {
            if c_prefix_matches(str_, b"default(") {
                if expression(filt, data, sym_func, str_.add(8), end, res) != 0 {
                    return -1;
                }
                if **end != b',' as c_char {
                    return -1;
                }
                *end = (*end).add(1);
                let mut val = expr_val_init();
                if expression(filt, data, sym_func, ws(*end), end, &mut val) != 0 {
                    return -1;
                }
                func_ok = 1;
                if !expr_val_exists_true(&*res) {
                    expr_val_free(&mut *res);
                    *res = val;
                }
            }
        }
        b'e' => {
            if c_prefix_matches(str_, b"exists(") {
                if expression(filt, data, sym_func, str_.add(7), end, res) != 0 {
                    return -1;
                }
                func_ok = 1;
                let exists = expr_val_exists_true(&*res);
                (*res).is_true = exists as c_char;
                (*res).d = exists as c_int as f64;
                (*res).is_str = 0;
            } else if c_prefix_matches(str_, b"exp(") {
                if expression(filt, data, sym_func, str_.add(4), end, res) != 0 {
                    return -1;
                }
                func_ok = 1;
                (*res).d = (*res).d.exp();
                (*res).is_str = 0;
                if (*res).d.is_nan() {
                    expr_val_undef(&mut *res);
                }
            }
        }
        b'l' => {
            if c_prefix_matches(str_, b"length(") {
                if expression(filt, data, sym_func, str_.add(7), end, res) != 0 {
                    return -1;
                }
                func_ok = expr_func_length(res);
            } else if c_prefix_matches(str_, b"log(") {
                if expression(filt, data, sym_func, str_.add(4), end, res) != 0 {
                    return -1;
                }
                func_ok = 1;
                (*res).d = (*res).d.ln();
                (*res).is_str = 0;
                if (*res).d.is_nan() {
                    expr_val_undef(&mut *res);
                }
            }
        }
        b'm' => {
            if c_prefix_matches(str_, b"min(") {
                if expression(filt, data, sym_func, str_.add(4), end, res) != 0 {
                    return -1;
                }
                func_ok = expr_func_min(res);
            } else if c_prefix_matches(str_, b"max(") {
                if expression(filt, data, sym_func, str_.add(4), end, res) != 0 {
                    return -1;
                }
                func_ok = expr_func_max(res);
            }
        }
        b'p' => {
            if c_prefix_matches(str_, b"pow(") {
                if expression(filt, data, sym_func, str_.add(4), end, res) != 0 {
                    return -1;
                }
                func_ok = 1;
                if **end != b',' as c_char {
                    return -1;
                }
                *end = (*end).add(1);
                let mut val = expr_val_init();
                if expression(filt, data, sym_func, ws(*end), end, &mut val) != 0 {
                    return -1;
                }
                if !expr_val_exists(&*res) || !expr_val_exists(&val) {
                    expr_val_undef(&mut *res);
                } else if (*res).is_str != 0 || val.is_str != 0 {
                    expr_val_free(&mut val);
                    return -1;
                } else {
                    func_ok = 1;
                    (*res).d = (*res).d.powf(val.d);
                    expr_val_free(&mut val);
                    (*res).is_str = 0;
                }
                if (*res).d.is_nan() {
                    expr_val_undef(&mut *res);
                }
            }
        }
        b's' => {
            if c_prefix_matches(str_, b"sqrt(") {
                if expression(filt, data, sym_func, str_.add(5), end, res) != 0 {
                    return -1;
                }
                func_ok = 1;
                (*res).d = (*res).d.sqrt();
                (*res).is_str = 0;
                if (*res).d.is_nan() {
                    expr_val_undef(&mut *res);
                }
            }
        }
        _ => {}
    }

    if func_ok < 0 {
        return -1;
    }

    let str_ = ws(*end);
    if *str_ != b')' as c_char {
        eprintln!("Missing ')'");
        return -1;
    }
    *end = str_.add(1);
    0
}

// original: simple_expr (htslib/hts_expr.c:284)
pub unsafe fn simple_expr(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    mut str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    str_ = ws(str_);
    if *str_ == b'(' as c_char {
        if expression(filt, data, sym_func, str_.add(1), end, res) != 0 {
            return -1;
        }
        let e = ws(*end);
        if *e != b')' as c_char {
            eprintln!("Missing ')'");
            return -1;
        }
        *end = e.add(1);
        return 0;
    }

    let mut fail = 0;
    let d = hts_str2dbl(str_, end, &mut fail);
    if str_ != *end {
        (*res).is_str = 0;
        (*res).d = d;
        return 0;
    }

    if *str_ == b'"' as c_char {
        (*res).is_str = 1;
        let mut e = str_.add(1);
        let mut backslash = 0;
        while *e != 0 && *e != b'"' as c_char {
            if *e == b'\\' as c_char {
                backslash = 1;
                e = e.add(1 + (*e.add(1) != 0) as usize);
            } else {
                e = e.add(1);
            }
        }
        let lit_len = e.offset_from(str_.add(1)) as usize;
        (*res).s.data.clear();
        kputsn(
            std::slice::from_raw_parts(str_.add(1).cast::<u8>(), lit_len),
            lit_len,
            &mut (*res).s,
        );
        if backslash != 0 {
            let mut i = 0usize;
            let mut j = 0usize;
            while i < (*res).s.data.len() {
                (*res).s.data[j] = (*res).s.data[i];
                j += 1;
                if (*res).s.data[i] == b'\\' {
                    i += 1;
                    match (*res).s.data[i] {
                        b'"' => (*res).s.data[j - 1] = b'"',
                        b'\\' => (*res).s.data[j - 1] = b'\\',
                        b't' => (*res).s.data[j - 1] = b'\t',
                        b'n' => (*res).s.data[j - 1] = b'\n',
                        b'r' => (*res).s.data[j - 1] = b'\r',
                        _ => {
                            (*res).s.data[j] = (*res).s.data[i];
                            j += 1;
                        }
                    }
                }
                i += 1;
            }
            (*res).s.data.truncate(j);
        }
        if *e != b'"' as c_char {
            return -1;
        }
        *end = e.add(1);
    } else if let Some(fn_) = sym_func {
        if fn_(data, str_, end, res) == 0 {
            return 0;
        }
        return func_expr(filt, data, sym_func, str_, end, res);
    } else {
        return -1;
    }
    0
}

// original: unary_expr (htslib/hts_expr.c:364)
pub unsafe fn unary_expr(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    mut str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    str_ = ws(str_);
    let err;
    if *str_ == b'+' as c_char || *str_ == b'-' as c_char {
        err = simple_expr(filt, data, sym_func, str_.add(1), end, res);
        if !expr_val_exists(&*res) {
            expr_val_undef(&mut *res);
        } else {
            if (*res).is_str != 0 {
                return -1;
            }
            if *str_ == b'-' as c_char {
                (*res).d = -(*res).d;
            }
            (*res).is_true = c_bool((*res).d != 0.0);
        }
    } else if *str_ == b'!' as c_char {
        err = unary_expr(filt, data, sym_func, str_.add(1), end, res);
        if (*res).is_true != 0 {
            (*res).d = 0.0;
            (*res).is_true = 0;
        } else if !expr_val_exists(&*res) {
            (*res).d = ((*res).is_true == 0) as c_int as f64;
            (*res).is_true = c_bool((*res).d != 0.0);
        } else if (*res).is_str != 0 {
            (*res).d = (*res).s.data.is_empty() as c_int as f64;
            (*res).is_true = c_bool((*res).d != 0.0);
        } else {
            (*res).d = ((*res).d as i64 == 0) as c_int as f64;
            (*res).is_true = c_bool((*res).d != 0.0);
        }
        (*res).is_str = 0;
    } else if *str_ == b'~' as c_char {
        err = unary_expr(filt, data, sym_func, str_.add(1), end, res);
        if !expr_val_exists(&*res) {
            expr_val_undef(&mut *res);
        } else {
            if (*res).is_str != 0 {
                return -1;
            }
            (*res).d = !((*res).d as i64) as f64;
            (*res).is_true = c_bool((*res).d != 0.0);
        }
    } else {
        err = simple_expr(filt, data, sym_func, str_, end, res);
    }
    if err != 0 {
        -1
    } else {
        0
    }
}

// original: mul_expr (htslib/hts_expr.c:423)
pub unsafe fn mul_expr(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    if unary_expr(filt, data, sym_func, str_, end, res) != 0 {
        return -1;
    }
    let mut str_ = *end;
    let mut val = expr_val_init();
    while *str_ != 0 {
        str_ = ws(str_);
        if *str_ == b'*' as c_char || *str_ == b'/' as c_char || *str_ == b'%' as c_char {
            if unary_expr(filt, data, sym_func, str_.add(1), end, &mut val) != 0 {
                return -1;
            }
            if !expr_val_exists(&val) || !expr_val_exists(&*res) {
                expr_val_undef(&mut *res);
            } else if val.is_str != 0 || (*res).is_str != 0 {
                expr_val_free(&mut val);
                return -1;
            }
        }
        if *str_ == b'*' as c_char {
            (*res).d *= val.d;
        } else if *str_ == b'/' as c_char {
            (*res).d /= val.d;
        } else if *str_ == b'%' as c_char {
            if val.d != 0.0 {
                (*res).d = ((*res).d as i64 % val.d as i64) as f64;
            } else {
                expr_val_undef(&mut *res);
            }
        } else {
            break;
        }
        (*res).is_true = c_bool(expr_val_exists(&*res) && (*res).d != 0.0);
        str_ = *end;
    }
    expr_val_free(&mut val);
    0
}

// original: add_expr (htslib/hts_expr.c:470)
pub unsafe fn add_expr(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    if mul_expr(filt, data, sym_func, str_, end, res) != 0 {
        return -1;
    }
    let mut str_ = *end;
    let mut val = expr_val_init();
    while *str_ != 0 {
        str_ = ws(str_);
        let mut undef = 0;
        if *str_ == b'+' as c_char || *str_ == b'-' as c_char {
            if mul_expr(filt, data, sym_func, str_.add(1), end, &mut val) != 0 {
                return -1;
            }
            if !expr_val_exists(&val) || !expr_val_exists(&*res) {
                undef = 1;
            } else if val.is_str != 0 || (*res).is_str != 0 {
                expr_val_free(&mut val);
                return -1;
            }
        }
        if *str_ == b'+' as c_char {
            (*res).d += val.d;
        } else if *str_ == b'-' as c_char {
            (*res).d -= val.d;
        } else {
            break;
        }
        if undef != 0 {
            expr_val_undef(&mut *res);
        } else {
            (*res).is_true = c_bool((*res).d != 0.0);
        }
        str_ = *end;
    }
    expr_val_free(&mut val);
    0
}

#[allow(clippy::too_many_arguments)]
unsafe fn bit_expr(
    next: unsafe fn(
        &mut hts_filter_t,
        *mut c_void,
        hts_expr_sym_func,
        *mut c_char,
        &mut *mut c_char,
        &mut hts_expr_val_t,
    ) -> c_int,
    op: u8,
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    if next(filt, data, sym_func, str_, end, res) != 0 {
        return -1;
    }
    let mut val = expr_val_init();
    let mut undef = 0;
    loop {
        let str_ = ws(*end);
        let is_op = match op {
            b'&' => *str_ == b'&' as c_char && *str_.add(1) != b'&' as c_char,
            b'|' => *str_ == b'|' as c_char && *str_.add(1) != b'|' as c_char,
            _ => *str_ == op as c_char,
        };
        if !is_op {
            break;
        }
        if next(filt, data, sym_func, str_.add(1), end, &mut val) != 0 {
            return -1;
        }
        if !expr_val_exists(&val) || !expr_val_exists(&*res) {
            undef = 1;
        } else if (*res).is_str != 0 || val.is_str != 0 {
            expr_val_free(&mut val);
            return -1;
        } else {
            let r = match op {
                b'&' => (*res).d as i64 & val.d as i64,
                b'^' => (*res).d as i64 ^ val.d as i64,
                _ => (*res).d as i64 | val.d as i64,
            };
            (*res).d = r as f64;
            (*res).is_true = c_bool(r != 0);
        }
    }
    expr_val_free(&mut val);
    if undef != 0 {
        expr_val_undef(&mut *res);
    }
    0
}

// original: bitand_expr (htslib/hts_expr.c:515)
pub unsafe fn bitand_expr(
    f: &mut hts_filter_t,
    d: *mut c_void,
    s: hts_expr_sym_func,
    st: *mut c_char,
    e: &mut *mut c_char,
    r: &mut hts_expr_val_t,
) -> c_int {
    bit_expr(add_expr, b'&', f, d, s, st, e, r)
}

// original: bitxor_expr (htslib/hts_expr.c:550)
pub unsafe fn bitxor_expr(
    f: &mut hts_filter_t,
    d: *mut c_void,
    s: hts_expr_sym_func,
    st: *mut c_char,
    e: &mut *mut c_char,
    r: &mut hts_expr_val_t,
) -> c_int {
    bit_expr(bitand_expr, b'^', f, d, s, st, e, r)
}

// original: bitor_expr (htslib/hts_expr.c:585)
pub unsafe fn bitor_expr(
    f: &mut hts_filter_t,
    d: *mut c_void,
    s: hts_expr_sym_func,
    st: *mut c_char,
    e: &mut *mut c_char,
    r: &mut hts_expr_val_t,
) -> c_int {
    bit_expr(bitxor_expr, b'|', f, d, s, st, e, r)
}

// original: cmp_expr (htslib/hts_expr.c:623)
pub unsafe fn cmp_expr(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    if bitor_expr(filt, data, sym_func, str_, end, res) != 0 {
        return -1;
    }
    let str_ = ws(*end);
    let mut val = expr_val_init();
    let mut err = 0;
    let mut cmp_done = 0;
    let op = if *str_ == b'>' as c_char && *str_.add(1) == b'=' as c_char {
        cmp_done = 1;
        err = cmp_expr(filt, data, sym_func, str_.add(2), end, &mut val);
        b'G'
    } else if *str_ == b'>' as c_char {
        cmp_done = 1;
        err = cmp_expr(filt, data, sym_func, str_.add(1), end, &mut val);
        b'>'
    } else if *str_ == b'<' as c_char && *str_.add(1) == b'=' as c_char {
        cmp_done = 1;
        err = cmp_expr(filt, data, sym_func, str_.add(2), end, &mut val);
        b'L'
    } else if *str_ == b'<' as c_char {
        cmp_done = 1;
        err = cmp_expr(filt, data, sym_func, str_.add(1), end, &mut val);
        b'<'
    } else {
        0
    };

    if cmp_done != 0 {
        if !expr_val_exists(&*res) || !expr_val_exists(&val) {
            expr_val_undef(&mut *res);
        } else {
            let r =
                if let (Some(lhs), Some(rhs)) = (expr_value_bytes(&*res), expr_value_bytes(&val)) {
                    let c = lhs.cmp(rhs);
                    match op {
                        b'G' => !c.is_lt(),
                        b'>' => c.is_gt(),
                        b'L' => !c.is_gt(),
                        _ => c.is_lt(),
                    }
                } else if (*res).is_str == 0 && val.is_str == 0 {
                    match op {
                        b'G' => (*res).d >= val.d,
                        b'>' => (*res).d > val.d,
                        b'L' => (*res).d <= val.d,
                        _ => (*res).d < val.d,
                    }
                } else {
                    false
                };
            (*res).is_true = c_bool(r);
            (*res).d = r as c_int as f64;
            (*res).is_str = 0;
        }
    }
    if cmp_done != 0 && (!expr_val_exists(&val) || !expr_val_exists(&*res)) {
        expr_val_undef(&mut *res);
    }
    expr_val_free(&mut val);
    if err != 0 {
        -1
    } else {
        0
    }
}

// original: eq_expr (htslib/hts_expr.c:696)
pub unsafe fn eq_expr(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    if cmp_expr(filt, data, sym_func, str_, end, res) != 0 {
        return -1;
    }
    let str_ = ws(*end);
    let mut val = expr_val_init();
    let mut err = 0;
    let mut eq_done = 0;
    if *str_ == b'=' as c_char && *str_.add(1) == b'=' as c_char {
        eq_done = 1;
        err = eq_expr(filt, data, sym_func, str_.add(2), end, &mut val);
        if err != 0 {
            (*res).is_true = 0;
            (*res).d = 0.0;
        } else if !expr_val_exists(&*res) || !expr_val_exists(&val) {
            expr_val_undef(&mut *res);
        } else {
            let r = if (*res).is_str != 0 {
                expr_value_bytes(&*res)
                    .zip(expr_value_bytes(&val))
                    .is_some_and(|(lhs, rhs)| lhs == rhs)
            } else {
                val.is_str == 0 && (*res).d == val.d
            };
            (*res).is_true = c_bool(r);
            (*res).d = r as c_int as f64;
        }
        (*res).is_str = 0;
    } else if *str_ == b'!' as c_char && *str_.add(1) == b'=' as c_char {
        eq_done = 1;
        err = eq_expr(filt, data, sym_func, str_.add(2), end, &mut val);
        if err != 0 {
            (*res).is_true = 0;
            (*res).d = 0.0;
        } else if !expr_val_exists(&*res) || !expr_val_exists(&val) {
            expr_val_undef(&mut *res);
        } else {
            let r = if (*res).is_str != 0 {
                expr_value_bytes(&*res)
                    .zip(expr_value_bytes(&val))
                    .is_none_or(|(lhs, rhs)| lhs != rhs)
            } else {
                val.is_str != 0 || (*res).d != val.d
            };
            (*res).is_true = c_bool(r);
            (*res).d = r as c_int as f64;
        }
        (*res).is_str = 0;
    } else if (*str_ == b'=' as c_char || *str_ == b'!' as c_char) && *str_.add(1) == b'~' as c_char
    {
        eq_done = 1;
        err = eq_expr(filt, data, sym_func, str_.add(2), end, &mut val);
        if val.is_str == 0 || (*res).is_str == 0 {
            expr_val_free(&mut val);
            return -1;
        }
        if !val.s.data.is_empty() && !(*res).s.data.is_empty() && val.is_true >= 0 && (*res).is_true >= 0 {
            let mut preg_tmp: crate::htslib_rs::c_compat::regex_t = std::mem::zeroed();
            let mut compile_regex = false;
            let preg_tmp_ptr = std::ptr::addr_of_mut!(preg_tmp);
            let preg = if filt.curr_regex >= filt.max_regex {
                if filt.curr_regex >= MAX_REGEX {
                    compile_regex = true;
                    preg_tmp_ptr
                } else {
                    compile_regex = true;
                    let idx = filt.curr_regex;
                    filt.max_regex += 1;
                    filter_regex_ptr(filt, idx)
                }
            } else {
                filter_regex_ptr(filt, filt.curr_regex)
            };
            if preg.is_null() {
                expr_val_free(&mut val);
                return -1;
            }
            let mut val_cstr = val.s.data.clone();
            val_cstr.push(0);
            if compile_regex {
                let ec = crate::htslib_rs::c_compat::regcomp(
                    preg,
                    val_cstr.as_ptr().cast::<c_char>(),
                    crate::htslib_rs::c_compat::REG_EXTENDED
                        | crate::htslib_rs::c_compat::REG_NOSUB,
                );
                if ec != 0 {
                    let mut errbuf = [0u8; 1024];
                    crate::htslib_rs::c_compat::regerror(
                        ec,
                        preg,
                        errbuf.as_mut_ptr().cast::<c_char>(),
                        errbuf.len(),
                    );
                    let msg_len = errbuf.iter().position(|&b| b == 0).unwrap_or(errbuf.len());
                    eprintln!(
                        "Failed regex: {}",
                        String::from_utf8_lossy(&errbuf[..msg_len])
                    );
                    expr_val_free(&mut val);
                    return -1;
                }
            }
            let mut res_cstr = (*res).s.data.clone();
            res_cstr.push(0);
            let matched = crate::htslib_rs::c_compat::regexec(
                preg,
                res_cstr.as_ptr().cast::<c_char>(),
                0,
                std::ptr::null_mut(),
                0,
            ) == 0;
            let r = if matched {
                *str_ == b'=' as c_char
            } else {
                *str_ == b'!' as c_char
            };
            (*res).is_true = c_bool(r);
            (*res).d = r as c_int as f64;
            if preg == preg_tmp_ptr {
                crate::htslib_rs::c_compat::regfree(preg);
            }
            filt.curr_regex += 1;
        } else {
            (*res).is_true = 0;
        }
        (*res).is_str = 0;
    }
    if eq_done != 0 && (!expr_val_exists(&val) || !expr_val_exists(&*res)) {
        expr_val_undef(&mut *res);
    }
    expr_val_free(&mut val);
    if err != 0 {
        -1
    } else {
        0
    }
}

fn expr_truth(v: &hts_expr_val_t) -> bool {
    // C truthiness uses `res->s.s` (NULL test), not emptiness: an allocated
    // (even empty) string is truthy. Use capacity as the `s.s != NULL` proxy.
    v.is_true != 0 || (v.is_str != 0 && v.s.data.capacity() != 0) || v.d != 0.0
}

// original: and_expr (htslib/hts_expr.c:795)
pub unsafe fn and_expr(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    if eq_expr(filt, data, sym_func, str_, end, res) != 0 {
        return -1;
    }
    loop {
        let mut val = expr_val_init();
        let str_ = ws(*end);
        if *str_ == b'&' as c_char && *str_.add(1) == b'&' as c_char {
            if eq_expr(filt, data, sym_func, str_.add(2), end, &mut val) != 0 {
                return -1;
            }
            if !expr_val_exists_true(&*res) || !expr_val_exists_true(&val) {
                expr_val_undef(&mut *res);
                (*res).d = 0.0;
            } else {
                let r = expr_truth(&*res) && expr_truth(&val);
                (*res).is_true = c_bool(r);
                (*res).d = r as c_int as f64;
                (*res).is_str = 0;
            }
        } else if *str_ == b'|' as c_char && *str_.add(1) == b'|' as c_char {
            if eq_expr(filt, data, sym_func, str_.add(2), end, &mut val) != 0 {
                return -1;
            }
            if (!expr_val_exists_true(&*res) && (!expr_val_exists_true(&val) || !expr_truth(&val)))
                || (!expr_val_exists_true(&val) && !expr_truth(&*res))
            {
                expr_val_undef(&mut *res);
                (*res).d = 0.0;
            } else {
                let r = expr_truth(&*res) || expr_truth(&val);
                (*res).is_true = c_bool(r);
                (*res).d = r as c_int as f64;
                (*res).is_str = 0;
            }
        } else {
            break;
        }
        expr_val_free(&mut val);
    }
    0
}

// original: expression (htslib/hts_expr.c:844)
pub unsafe fn expression(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    str_: *mut c_char,
    end: &mut *mut c_char,
    res: &mut hts_expr_val_t,
) -> c_int {
    and_expr(filt, data, sym_func, str_, end, res)
}

// original: hts_filter_init (htslib/hts_expr.c:849)
pub unsafe fn hts_expr_c_849_hts_filter_init(str_: *const c_char) -> *mut hts_filter_t {
    if str_.is_null() {
        return std::ptr::null_mut();
    }
    let len = {
        let mut len = 0usize;
        while *str_.add(len) != 0 {
            len += 1;
        }
        len
    };
    hts_filter_init_bytes(std::slice::from_raw_parts(str_.cast::<u8>(), len))
}

pub fn hts_filter_init_bytes(expr: &[u8]) -> *mut hts_filter_t {
    if expr.iter().any(|&byte| byte == 0) {
        return std::ptr::null_mut();
    }
    let mut expr_buf = Vec::with_capacity(expr.len() + 101);
    expr_buf.extend_from_slice(expr);
    expr_buf.push(0);
    let preg = (0..MAX_REGEX)
        .map(|_| unsafe { std::mem::zeroed() })
        .collect::<Vec<crate::htslib_rs::c_compat::regex_t>>();
    Box::into_raw(Box::new(hts_filter_t {
        expr: expr_buf,
        parsed: false,
        curr_regex: 0,
        max_regex: 0,
        preg,
    }))
}

unsafe fn filter_expr_ptr(filt: &mut hts_filter_t) -> *mut c_char {
    filt.expr.as_mut_ptr().cast::<c_char>()
}

unsafe fn filter_regex_ptr(
    filt: &mut hts_filter_t,
    idx: usize,
) -> *mut crate::htslib_rs::c_compat::regex_t {
    if idx >= filt.preg.len() {
        std::ptr::null_mut()
    } else {
        std::ptr::addr_of_mut!(filt.preg[idx])
    }
}

// original: hts_filter_free (htslib/hts_expr.c:863)
pub unsafe fn hts_expr_c_863_hts_filter_free(filt: *mut hts_filter_t) {
    if filt.is_null() {
        return;
    }
    let mut filt = Box::from_raw(filt);
    for i in 0..filt.max_regex {
        crate::htslib_rs::c_compat::regfree(&mut filt.preg[i]);
    }
}

unsafe fn hts_filter_eval_inner(
    filt: &mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    res: &mut hts_expr_val_t,
) -> c_int {
    let mut end: *mut c_char = std::ptr::null_mut();
    filt.curr_regex = 0;
    let expr = filter_expr_ptr(filt);
    if expression(filt, data, sym_func, expr, &mut end, res) != 0 {
        return -1;
    }
    if !end.is_null() && *ws(end) != 0 {
        let text_len = filt.expr.iter().position(|&b| b == 0).unwrap_or(filt.expr.len());
        eprintln!(
            "Unable to parse expression at {}",
            String::from_utf8_lossy(&filt.expr[..text_len])
        );
        return -1;
    }
    if (*res).is_str != 0 {
        // Upstream: `res->is_true |= res->s.s != NULL`. A present string is
        // true even when empty; only an *absent* (null) string is false. In
        // this owned port a null string is represented as `is_str == 0`
        // (hts_expr_val_undef clears is_str), so `is_str != 0` already implies
        // a present (non-null) string — hence an empty-but-present string is
        // true. (The old `!data.is_empty()` test wrongly made "" false.)
        (*res).is_true |= 1;
        (*res).d = (*res).is_true as f64;
    } else if expr_val_exists(&*res) {
        (*res).is_true |= ((*res).d != 0.0) as c_int as c_char;
    }
    0
}

// original: hts_filter_eval_ (htslib/hts_expr.c:875)
pub unsafe fn hts_filter_eval_(
    filt: *mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    res: *mut hts_expr_val_t,
) -> c_int {
    let Some(filt) = filt.as_mut() else {
        return -1;
    };
    let Some(res) = res.as_mut() else {
        return -1;
    };
    hts_filter_eval_inner(filt, data, sym_func, res)
}

// original: hts_filter_eval (htslib/hts_expr.c:903)
pub unsafe fn hts_expr_c_903_hts_filter_eval(
    filt: *mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    res: *mut hts_expr_val_t,
) -> c_int {
    let Some(filt) = filt.as_mut() else {
        return -1;
    };
    let Some(res) = res.as_mut() else {
        return -1;
    };
    if !res.s.data.is_empty() || res.s.data.capacity() != 0 {
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"hts_filter_eval".as_ptr(),
            c"Results structure must be cleared before calling this function".as_ptr(),
        );
        return -1;
    }
    *res = expr_val_init();
    hts_filter_eval_inner(filt, data, sym_func, res)
}

// original: hts_filter_eval2 (htslib/hts_expr.c:920)
pub unsafe fn hts_expr_c_920_hts_filter_eval2(
    filt: *mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    res: *mut hts_expr_val_t,
) -> c_int {
    let Some(filt) = filt.as_mut() else {
        return -1;
    };
    let Some(res) = res.as_mut() else {
        return -1;
    };
    ks_free(&mut res.s);
    *res = expr_val_init();
    hts_filter_eval_inner(filt, data, sym_func, res)
}

// Top-level public wrappers (mirrors of htslib's published API).

pub unsafe fn hts_filter_init(str_: *const c_char) -> *mut hts_filter_t {
    hts_expr_c_849_hts_filter_init(str_)
}

pub unsafe fn hts_filter_free(filt: *mut hts_filter_t) {
    hts_expr_c_863_hts_filter_free(filt)
}

pub unsafe fn hts_filter_eval(
    filt: *mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    res: *mut hts_expr_val_t,
) -> c_int {
    hts_expr_c_903_hts_filter_eval(filt, data, sym_func, res)
}

pub unsafe fn hts_filter_eval2(
    filt: *mut hts_filter_t,
    data: *mut c_void,
    sym_func: hts_expr_sym_func,
    res: *mut hts_expr_val_t,
) -> c_int {
    hts_expr_c_920_hts_filter_eval2(filt, data, sym_func, res)
}
