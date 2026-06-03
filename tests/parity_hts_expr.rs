// Parity tests for functions recently MOVED from src/hts.rs into src/hts_expr.rs.
//
// Public API covered (htslib/hts_expr.h):
//   - hts_filter_init
//   - hts_filter_free
//   - hts_filter_eval2  (the recommended modern entry)
//   - hts_filter_eval   (deprecated)
//
// The internal parser functions (simple_expr, unary_expr, mul_expr, add_expr,
// cmp_expr, eq_expr, and_expr, etc.) are not exported — they are exercised
// transitively through the eval entry points.
//
// hts-sys does NOT include htslib/hts_expr.h in its bindgen wrapper.h, so the
// extern symbols below are declared manually. Both sides link against the
// SAME libhts.a (hts-sys's static archive), so calling hts_filter_init/_eval
// directly via these externs is valid for the parity feature.
//
// Strategy: we treat hts_filter_t and hts_expr_val_t as opaque (void) on the
// C side. The expr_val_t result is read back via a parallel native struct
// after each eval and compared field-by-field against the native eval.

#![cfg(feature = "parity")]

// Pin hts-sys rlib into the link graph so libhts.a (which provides the
// `hts_filter_*` symbols extern-declared below) lands on the link line.
// bindgen doesn't expose hts_expr.h, so without this we'd see
// "undefined symbol: hts_filter_init" at link time.
use hts_sys as _;

use std::ffi::{c_char, c_int, c_void, CStr};
use std::path::Path;

// ---------------------------------------------------------------------------
// Externs into libhts for hts_filter_*. These are public symbols (HTSLIB_EXPORT
// in hts_expr.h), just not bindgen'd by hts-sys.
// ---------------------------------------------------------------------------

// C-side opaque structs. We never deref these — only pass pointers around.
#[repr(C)]
struct CHtsFilter {
    _opaque: [u8; 0],
}

// The C `hts_expr_val_t` layout (matches src/hts_expr.rs hts_expr_val_t and
// htslib's hts_expr.h definition):
//   char is_str; char is_true; kstring_t s; double d;
// kstring_t = { size_t l; size_t m; char *s; }
#[repr(C)]
#[derive(Default)]
struct CKstring {
    l: usize,
    m: usize,
    s: *mut c_char,
}

#[repr(C)]
struct CHtsExprVal {
    is_str: c_char,
    is_true: c_char,
    s: CKstring,
    d: f64,
}

// C-side hts_expr_sym_func signature — must match htslib's:
// int (*)(void *data, char *str, char **end, hts_expr_val_t *res)
type CExprSymFunc = Option<
    unsafe extern "C" fn(
        data: *mut c_void,
        str_: *mut c_char,
        end: *mut *mut c_char,
        res: *mut CHtsExprVal,
    ) -> c_int,
>;

extern "C" {
    fn hts_filter_init(str_: *const c_char) -> *mut CHtsFilter;
    fn hts_filter_free(filt: *mut CHtsFilter);
    fn hts_filter_eval(
        filt: *mut CHtsFilter,
        data: *mut c_void,
        sym_func: CExprSymFunc,
        res: *mut CHtsExprVal,
    ) -> c_int;
    fn hts_filter_eval2(
        filt: *mut CHtsFilter,
        data: *mut c_void,
        sym_func: CExprSymFunc,
        res: *mut CHtsExprVal,
    ) -> c_int;
}

// ---------------------------------------------------------------------------
// Symbol lookup callback used by both eval paths. Mirrors the canonical
// test_expr.c "lookup" function so the result is deterministic.
// ---------------------------------------------------------------------------

unsafe extern "C" fn lookup_native(
    _data: *mut c_void,
    str_: *mut c_char,
    end: *mut *mut c_char,
    res: *mut htslib_rs::hts_expr::hts_expr_val_t,
) -> c_int {
    (*res).is_str = 0;
    if libc::strncmp(str_, c"a".as_ptr(), 1) == 0 {
        *end = str_.add(1);
        (*res).d = 1.0;
    } else if libc::strncmp(str_, c"b".as_ptr(), 1) == 0 {
        *end = str_.add(1);
        (*res).d = 2.0;
    } else if libc::strncmp(str_, c"c".as_ptr(), 1) == 0 {
        *end = str_.add(1);
        (*res).d = 3.0;
    } else {
        return -1;
    }
    0
}

unsafe extern "C" fn lookup_c(
    _data: *mut c_void,
    str_: *mut c_char,
    end: *mut *mut c_char,
    res: *mut CHtsExprVal,
) -> c_int {
    (*res).is_str = 0;
    if libc::strncmp(str_, c"a".as_ptr(), 1) == 0 {
        *end = str_.add(1);
        (*res).d = 1.0;
    } else if libc::strncmp(str_, c"b".as_ptr(), 1) == 0 {
        *end = str_.add(1);
        (*res).d = 2.0;
    } else if libc::strncmp(str_, c"c".as_ptr(), 1) == 0 {
        *end = str_.add(1);
        (*res).d = 3.0;
    } else {
        return -1;
    }
    0
}

// ---------------------------------------------------------------------------
// Parity helpers
// ---------------------------------------------------------------------------

fn run_native(expr: &CStr) -> (c_int, c_int, c_int, f64, Vec<u8>) {
    unsafe {
        let filt = htslib_rs::hts_expr::hts_filter_init(expr.as_ptr());
        assert!(
            !filt.is_null(),
            "native hts_filter_init returned NULL for {expr:?}"
        );
        let mut res = htslib_rs::hts_expr::hts_expr_val_t {
            is_str: 0,
            is_true: 0,
            s: htslib_rs::kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
            d: 0.0,
        };
        let rc = htslib_rs::hts_expr::hts_filter_eval2(
            filt,
            std::ptr::null_mut(),
            Some(lookup_native),
            &mut res,
        );
        let s_bytes = if res.s.s.is_null() {
            Vec::new()
        } else {
            std::slice::from_raw_parts(res.s.s as *const u8, res.s.l).to_vec()
        };
        let out = (
            rc,
            res.is_str as c_int,
            res.is_true as c_int,
            res.d,
            s_bytes,
        );
        htslib_rs::hts_expr::hts_expr_val_free(&mut res);
        htslib_rs::hts_expr::hts_filter_free(filt);
        out
    }
}

fn run_c(expr: &CStr) -> (c_int, c_int, c_int, f64, Vec<u8>) {
    unsafe {
        let filt = hts_filter_init(expr.as_ptr());
        assert!(
            !filt.is_null(),
            "C hts_filter_init returned NULL for {expr:?}"
        );
        let mut res = CHtsExprVal {
            is_str: 0,
            is_true: 0,
            s: CKstring::default(),
            d: 0.0,
        };
        let rc = hts_filter_eval2(filt, std::ptr::null_mut(), Some(lookup_c), &mut res);
        let s_bytes = if res.s.s.is_null() {
            Vec::new()
        } else {
            std::slice::from_raw_parts(res.s.s as *const u8, res.s.l).to_vec()
        };
        let out = (
            rc,
            res.is_str as c_int,
            res.is_true as c_int,
            res.d,
            s_bytes,
        );
        // free res.s
        if !res.s.s.is_null() {
            libc::free(res.s.s.cast());
        }
        hts_filter_free(filt);
        out
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn parity_hts_filter_init_succeeds() {
    // hts_filter_init does no parsing — it just allocates and copies the
    // string. Both sides should succeed (non-NULL) for any non-NULL input.
    let exprs: &[&CStr] = &[
        c"a + b",
        c"1",
        c"a > b && c < 10",
        c"FLAG & 4",
        c"MAPQ >= 30",
        c"length(SEQ) > 50",
        c"min(MAPQ, NM) > 0",
        c"!(a == b)",
    ];
    for e in exprs {
        unsafe {
            let n = htslib_rs::hts_expr::hts_filter_init(e.as_ptr());
            let cc = hts_filter_init(e.as_ptr());
            assert!(!n.is_null(), "native init NULL for {e:?}");
            assert!(!cc.is_null(), "C init NULL for {e:?}");
            htslib_rs::hts_expr::hts_filter_free(n);
            hts_filter_free(cc);
        }
    }
}

#[test]
fn parity_hts_filter_eval2_arithmetic() {
    // Use only a/b/c which lookup_* resolves to 1.0/2.0/3.0.
    let exprs: &[&CStr] = &[
        c"a",
        c"b",
        c"c",
        c"a + b",
        c"c - a",
        c"a * b * c",
        c"c / b",
        c"a + b * c",
        c"(a + b) * c",
        c"-a",
        c"!a",
        c"!0",
        c"a == 1",
        c"b > a",
        c"a < b && b < c",
        c"a || c",
        c"a & 0",
        c"6 | 1",
        c"3 ^ 1",
        c"3 % 2",
        c"min(a, c)",
        c"max(a, c)",
        c"avg(a, c)",
        c"a == b ? a : c",
    ];
    for e in exprs {
        let n = run_native(e);
        let cc = run_c(e);
        assert_eq!(n.0, cc.0, "rc mismatch for {e:?}");
        assert_eq!(n.1, cc.1, "is_str mismatch for {e:?}");
        assert_eq!(n.2, cc.2, "is_true mismatch for {e:?}");
        // Compare d carefully, allowing NaN==NaN.
        let nd = n.3;
        let cd = cc.3;
        let ok = nd == cd || (nd.is_nan() && cd.is_nan());
        assert!(ok, "d mismatch for {e:?}: native={nd} c={cd}");
        assert_eq!(n.4, cc.4, "s mismatch for {e:?}");
    }
}

#[test]
fn parity_hts_filter_eval_deprecated_path() {
    // Exercise the deprecated `hts_filter_eval` entry too — it differs from
    // eval2 by demanding a pre-cleared result struct (which we ensure).
    let expr = c"a + b * c";
    unsafe {
        // Native
        let nfilt = htslib_rs::hts_expr::hts_filter_init(expr.as_ptr());
        let mut nres = htslib_rs::hts_expr::hts_expr_val_t {
            is_str: 0,
            is_true: 0,
            s: htslib_rs::kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
            d: 0.0,
        };
        let nrc = htslib_rs::hts_expr::hts_filter_eval(
            nfilt,
            std::ptr::null_mut(),
            Some(lookup_native),
            &mut nres,
        );

        // C
        let cfilt = hts_filter_init(expr.as_ptr());
        let mut cres = CHtsExprVal {
            is_str: 0,
            is_true: 0,
            s: CKstring::default(),
            d: 0.0,
        };
        let crc = hts_filter_eval(cfilt, std::ptr::null_mut(), Some(lookup_c), &mut cres);

        assert_eq!(nrc, crc);
        assert_eq!(nres.is_str, cres.is_str);
        assert_eq!(nres.is_true, cres.is_true);
        assert_eq!(nres.d, cres.d);

        htslib_rs::hts_expr::hts_expr_val_free(&mut nres);
        htslib_rs::hts_expr::hts_filter_free(nfilt);
        if !cres.s.s.is_null() {
            libc::free(cres.s.s.cast());
        }
        hts_filter_free(cfilt);
    }
}

#[test]
fn hts_filter_free_handles_null_safely() {
    // The C function and the native translation must both be safe to call
    // with NULL (htslib doc says "If filt is NULL, nothing happens").
    unsafe {
        htslib_rs::hts_expr::hts_filter_free(std::ptr::null_mut());
        hts_filter_free(std::ptr::null_mut());
    }
}

#[test]
fn hts_filter_init_and_free_basic() {
    // Sanity-check init+free across a few expression shapes, including the
    // ones requested in the parity-task prompt.
    let exprs: &[&CStr] = &[
        c"FLAG & 4",
        c"MAPQ >= 30",
        c"length(SEQ) > 50",
        c"min(MAPQ, NM) > 0",
    ];
    for e in exprs {
        unsafe {
            let n = htslib_rs::hts_expr::hts_filter_init(e.as_ptr());
            assert!(!n.is_null());
            htslib_rs::hts_expr::hts_filter_free(n);

            let cc = hts_filter_init(e.as_ptr());
            assert!(!cc.is_null());
            hts_filter_free(cc);
        }
    }
}

// ---------------------------------------------------------------------------
// Real-BAM eval comparison (uses sam_passes_filter under the hood would be the
// htslib way, but to compare *just* hts_filter_eval against records we instead
// build a sym_func that resolves SAM-record fields ourselves on both sides).
//
// This guards against the move + visibility changes breaking expression eval
// when the input drives more codepaths than the synthetic lookup above.
// ---------------------------------------------------------------------------

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

// Common record field resolver. The expressions we evaluate reference
// names that this resolves to numeric values from a captured bam1_t.
//
// SAFETY: `data` must point at a `RecordCtx`.

#[repr(C)]
struct RecordCtx {
    flag: i64,
    qual: i64,
    pos: i64,
}

type FieldGetter = fn(&RecordCtx) -> f64;

unsafe fn resolve_field(
    str_: *mut c_char,
    end: *mut *mut c_char,
    ctx: *const RecordCtx,
) -> Option<f64> {
    let known: &[(&[u8], FieldGetter)] = &[
        (b"FLAG", |c| c.flag as f64),
        (b"MAPQ", |c| c.qual as f64),
        (b"POS", |c| c.pos as f64),
    ];
    for (name, getter) in known {
        if libc::strncmp(str_, name.as_ptr().cast(), name.len()) == 0 {
            // Ensure the next char is not alnum/_ (so we don't match "FLAG2").
            let next = *str_.add(name.len()) as u8;
            if !(next.is_ascii_alphanumeric() || next == b'_') {
                *end = str_.add(name.len());
                return Some(getter(&*ctx));
            }
        }
    }
    None
}

unsafe extern "C" fn record_lookup_native(
    data: *mut c_void,
    str_: *mut c_char,
    end: *mut *mut c_char,
    res: *mut htslib_rs::hts_expr::hts_expr_val_t,
) -> c_int {
    (*res).is_str = 0;
    if let Some(v) = resolve_field(str_, end, data.cast::<RecordCtx>()) {
        (*res).d = v;
        0
    } else {
        -1
    }
}

unsafe extern "C" fn record_lookup_c(
    data: *mut c_void,
    str_: *mut c_char,
    end: *mut *mut c_char,
    res: *mut CHtsExprVal,
) -> c_int {
    (*res).is_str = 0;
    if let Some(v) = resolve_field(str_, end, data.cast::<RecordCtx>()) {
        (*res).d = v;
        0
    } else {
        -1
    }
}

#[test]
fn parity_hts_filter_eval_against_real_bam_records() {
    use htslib_rs::{
        bam_destroy1, bam_init1, hts_close, hts_open, sam_hdr_destroy, sam_hdr_read, sam_read1,
    };

    let fixture_path = fixture("htslib/test/range.bam");
    if !fixture_path.exists() {
        // Don't fail the suite if the fixture isn't checked out.
        return;
    }
    let cpath = std::ffi::CString::new(fixture_path.to_str().unwrap()).unwrap();

    let exprs: &[&CStr] = &[
        c"FLAG & 4",
        c"MAPQ >= 30",
        c"FLAG == 0",
        c"MAPQ > 0 && (FLAG & 256) == 0",
        c"POS > 0",
    ];

    unsafe {
        let fp = hts_open(cpath.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bam_init1();
        assert!(!rec.is_null());

        // Read up to N records and evaluate each expression on both sides.
        let mut nread = 0usize;
        loop {
            let r = sam_read1(fp, hdr, rec);
            if r < 0 || nread >= 8 {
                break;
            }
            let core = &(*rec).core;
            let ctx = RecordCtx {
                flag: core.flag as i64,
                qual: core.qual as i64,
                pos: core.pos as i64,
            };
            for e in exprs {
                // Native
                let nfilt = htslib_rs::hts_expr::hts_filter_init(e.as_ptr());
                let mut nres = htslib_rs::hts_expr::hts_expr_val_t {
                    is_str: 0,
                    is_true: 0,
                    s: htslib_rs::kstring_t {
                        l: 0,
                        m: 0,
                        s: std::ptr::null_mut(),
                    },
                    d: 0.0,
                };
                let n_rc = htslib_rs::hts_expr::hts_filter_eval2(
                    nfilt,
                    (&ctx as *const RecordCtx) as *mut c_void,
                    Some(record_lookup_native),
                    &mut nres,
                );

                // C
                let cfilt = hts_filter_init(e.as_ptr());
                let mut cres = CHtsExprVal {
                    is_str: 0,
                    is_true: 0,
                    s: CKstring::default(),
                    d: 0.0,
                };
                let c_rc = hts_filter_eval2(
                    cfilt,
                    (&ctx as *const RecordCtx) as *mut c_void,
                    Some(record_lookup_c),
                    &mut cres,
                );

                assert_eq!(n_rc, c_rc, "rc mismatch for {e:?} on record {nread}");
                assert_eq!(nres.is_str, cres.is_str, "is_str mismatch for {e:?}");
                assert_eq!(nres.is_true, cres.is_true, "is_true mismatch for {e:?}");
                assert_eq!(nres.d, cres.d, "d mismatch for {e:?}");

                htslib_rs::hts_expr::hts_expr_val_free(&mut nres);
                htslib_rs::hts_expr::hts_filter_free(nfilt);
                if !cres.s.s.is_null() {
                    libc::free(cres.s.s.cast());
                }
                hts_filter_free(cfilt);
            }
            nread += 1;
        }
        bam_destroy1(rec);
        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
        assert!(nread > 0, "no records read from {:?}", fixture_path);
    }
}
