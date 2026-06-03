use std::ffi::{c_char, c_int};

// original: probaln_par_t (htslib/htslib/hts.h:1435)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct probaln_par_t {
    pub d: f32,
    pub e: f32,
    pub bw: c_int,
}

unsafe extern "C" {
    static mut optarg: *mut c_char;
    static mut optind: c_int;
}

const EI: f64 = 0.25;
const EM: f64 = 0.33333333333;

/// Quality (Phred) → probability lookup table.
///
/// Lazy-initialized via `LazyLock` for thread-safe one-time computation; a
/// previous `static mut G_QUAL2PROB: [f32; 256] = [0.0; 256];` with an
/// in-band `if G_QUAL2PROB[0] == 0.0 { …populate… }` guard had a classic
/// lazy-init data race — thread B could observe the partially-populated table
/// (low indices set, high indices still 0.0) while thread A was still in the
/// init loop, producing 0.0 instead of `10.0^(-q/10)` for high-quality reads
/// and corrupting BAQ output. Surfaced by `realn01_baq_apply/extend_*` at
/// `--test-threads=8`.
static G_QUAL2PROB: std::sync::LazyLock<[f32; 256]> = std::sync::LazyLock::new(|| {
    let mut t = [0.0f32; 256];
    let mut i = 0usize;
    while i < 256 {
        t[i] = 10.0_f64.powf(-(i as f64) / 10.0) as f32;
        i += 1;
    }
    t
});

#[allow(clippy::too_many_arguments)]
pub unsafe fn probaln_glocal(
    ref_: *const u8,
    l_ref: c_int,
    query: *const u8,
    l_query: c_int,
    iqual: *const u8,
    c: *const probaln_par_t,
    state: *mut c_int,
    q: *mut u8,
) -> c_int {
    let mut m = [0.0_f64; 9];
    let mut i: c_int;
    let mut k: c_int;
    let Pr: c_int;

    macro_rules! set_u {
        ($b:expr, $i:expr, $k:expr) => {{
            let mut x: c_int = ($i) - ($b);
            x = if x > 0 { x } else { 0 };
            (($k) - x + 1) * 3
        }};
    }

    if l_ref < 0 || !(0..c_int::MAX - 2).contains(&l_query) {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return c_int::MIN;
    }
    if l_ref == 0 || l_query == 0 {
        return 0;
    }

    let is_backward = !state.is_null() && !q.is_null();
    let mut bw = if l_ref > l_query { l_ref } else { l_query };
    if bw > (*c).bw {
        bw = (*c).bw;
    }
    let len_diff = (l_ref - l_query).abs();
    if bw < len_diff {
        bw = len_diff;
    }
    let bw2 = bw * 2 + 1;
    let i_dim: usize = if bw2 < l_ref {
        bw2 as usize * 3 + 6
    } else {
        l_ref as usize * 3 + 6
    };
    let rows = (l_query as usize) + 1;
    let f_len = match rows.checked_mul(i_dim) {
        Some(n) => n,
        None => {
            *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
            return c_int::MIN;
        }
    };
    if usize::MAX / ((l_query as usize) + 1) / i_dim < std::mem::size_of::<f64>() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
        return c_int::MIN;
    }

    let mut f = Vec::new();
    if f.try_reserve_exact(f_len).is_err() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
        return c_int::MIN;
    }
    f.resize(f_len, 0.0_f64);

    let mut b = Vec::new();
    if is_backward {
        if b.try_reserve_exact(f_len).is_err() {
            *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
            return c_int::MIN;
        }
        b.resize(f_len, 0.0_f64);
    }

    let s_len = (l_query as usize) + 2;
    let mut s = Vec::new();
    if s.try_reserve_exact(s_len).is_err() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
        return c_int::MIN;
    }
    s.resize(s_len, 0.0_f64);

    let mut qual = Vec::new();
    if qual.try_reserve_exact(l_query as usize).is_err() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
        return c_int::MIN;
    }
    qual.resize(l_query as usize, 0.0_f32);

    // Table is lazy-initialized exactly once across all threads — see G_QUAL2PROB.
    qual[0] = 0.0;
    i = 0;
    while i < l_query {
        let qi = if !iqual.is_null() {
            *iqual.add(i as usize)
        } else {
            30
        };
        qual[i as usize] = G_QUAL2PROB[qi as usize];
        i += 1;
    }

    let sM = 1.0 / (2 * l_query + 2) as f64;
    let sI = sM;
    m[0] = (1.0 - (*c).d as f64 - (*c).d as f64) * (1.0 - sM);
    m[1] = (*c).d as f64 * (1.0 - sM);
    m[2] = m[1];
    m[3] = (1.0 - (*c).e as f64) * (1.0 - sI);
    m[4] = (*c).e as f64 * (1.0 - sI);
    m[5] = 0.0;
    m[6] = 1.0 - (*c).e as f64;
    m[2 * 3 + 1] = 0.0;
    m[2 * 3 + 2] = (*c).e as f64;
    let bM = (1.0 - (*c).d as f64) / l_ref as f64;
    let bI = (*c).d as f64 / l_ref as f64;

    k = set_u!(bw, 0, 0);
    f[k as usize] = 1.0;
    s[0] = 1.0;

    {
        let fi_off = i_dim;
        let beg = 1;
        let end = if l_ref < bw + 1 { l_ref } else { bw + 1 };
        let mut sum = 0.0;
        k = beg;
        while k <= end {
            let e = if *ref_.add((k - 1) as usize) > 3 || *query > 3 {
                1.0
            } else if *ref_.add((k - 1) as usize) == *query {
                1.0 - qual[0] as f64
            } else {
                qual[0] as f64 * EM
            };
            let u = set_u!(bw, 1, k);
            f[fi_off + u as usize] = e * bM;
            f[fi_off + u as usize + 1] = EI * bI;
            sum += f[fi_off + u as usize] + f[fi_off + u as usize + 1];
            k += 1;
        }
        s[1] = sum;
    }

    i = 2;
    while i <= l_query {
        let fi_off = i as usize * i_dim;
        let fi1_off = (i as usize - 1) * i_dim;
        let mut beg = 1;
        let mut end = l_ref;
        let x = i - bw;
        if beg < x {
            beg = x;
        }
        let x = i + bw;
        if end > x {
            end = x;
        }
        let qli = qual[(i - 1) as usize] as f64;
        let qyi = *query.add((i - 1) as usize);
        let e_arr = [qli * EM, 1.0 - qli, 1.0, 1.0];
        let mm = 1.0 / s[(i - 1) as usize];
        let xm = [
            mm * m[0],
            mm * m[3],
            mm * m[6],
            EI * mm * m[1],
            EI * mm * m[4],
        ];
        let u = set_u!(bw, i, beg);
        let v11 = set_u!(bw, i - 1, beg - 1);
        let mut xi = fi_off + u as usize;
        let mut yi = fi1_off + v11 as usize;
        let mut l_x0 = m[2] * f[xi];
        let mut l_x2 = m[8] * f[xi + 2];
        let mut sum = 0.0;
        k = beg;
        while k <= end {
            let cond = ((*ref_.add((k - 1) as usize) > 3 || qyi > 3) as usize) * 2
                + (*ref_.add((k - 1) as usize) == qyi) as usize;
            let z0 = xm[0] * f[yi];
            let z1 = xm[1] * f[yi + 1];
            let z2 = xm[2] * f[yi + 2];
            let z3 = xm[3] * f[yi + 3];
            let z4 = xm[4] * f[yi + 4];
            f[xi] = e_arr[cond] * (z0 + z1 + z2);
            f[xi + 1] = z3 + z4;
            f[xi + 2] = l_x0 + l_x2;
            sum += f[xi] + f[xi + 1] + f[xi + 2];
            l_x0 = m[2] * f[xi];
            l_x2 = m[8] * f[xi + 2];
            k += 1;
            xi += 3;
            yi += 3;
        }
        s[i as usize] = sum;
        i += 1;
    }

    {
        let mut sum = 0.0;
        let mm = 1.0 / s[l_query as usize];
        k = 1;
        while k <= l_ref {
            let u = set_u!(bw, l_query, k);
            if !(u < 3 || u as usize >= i_dim) {
                let off = l_query as usize * i_dim + u as usize;
                sum += mm * f[off] * sM + mm * f[off + 1] * sI;
            }
            k += 1;
        }
        s[l_query as usize + 1] = sum;
    }

    {
        let mut p = 1.0;
        let mut Pr1 = 0.0;
        i = 0;
        while i <= l_query + 1 {
            p *= s[i as usize];
            if p < 1e-100 {
                Pr1 += -4.343 * p.ln();
                p = 1.0;
            }
            i += 1;
        }
        Pr1 += -4.343 * (p * l_ref as f64 * l_query as f64).ln();
        Pr = (Pr1 + 0.499) as c_int;
        if !is_backward {
            return Pr;
        }
    }

    k = 1;
    while k <= l_ref {
        let bi = l_query as usize * i_dim;
        let u = set_u!(bw, l_query, k);
        if !(u < 3 || u as usize >= i_dim) {
            b[bi + u as usize] = sM / s[l_query as usize] / s[l_query as usize + 1];
            b[bi + u as usize + 1] = sI / s[l_query as usize] / s[l_query as usize + 1];
        }
        k += 1;
    }

    i = l_query - 1;
    while i >= 1 {
        let bi_off = i as usize * i_dim;
        let bi1_off = (i as usize + 1) * i_dim;
        let y = if i > 1 { 1.0 } else { 0.0 };
        let qli1 = qual[i as usize] as f64;
        let qyi1 = *query.add(i as usize);
        let mut beg = 1;
        let mut end = l_ref;
        let x = i - bw;
        if beg < x {
            beg = x;
        }
        let x = i + bw;
        if end > x {
            end = x;
        }
        let e_arr = [qli1 * EM, 1.0 - qli1, 1.0, 1.0];
        let u = set_u!(bw, i, end);
        let v10 = set_u!(bw, i + 1, end);
        let mut xi = bi_off + u as usize;
        let mut yi = bi1_off + v10 as usize;
        let mut xi_5 = b[xi + 5];
        let e1 = EI * m[1];
        let e4 = EI * m[4];
        let n = 1.0 / s[i as usize];
        k = end;
        while k >= beg {
            let e = if k >= l_ref {
                0.0
            } else {
                let cond = ((*ref_.add(k as usize) > 3 || qyi1 > 3) as usize) * 2
                    + (*ref_.add(k as usize) == qyi1) as usize;
                e_arr[cond] * b[yi + 3]
            };
            b[xi + 1] = e * m[3] + e4 * b[yi + 1];
            b[xi] = e * m[0] + e1 * b[yi + 1] + m[2] * xi_5;
            b[xi + 2] = (e * m[6] + m[8] * xi_5) * y;
            xi_5 = b[xi + 2];
            b[xi + 1] *= n;
            b[xi] *= n;
            b[xi + 2] *= n;
            if k == beg {
                break;
            }
            k -= 1;
            xi -= 3;
            yi -= 3;
        }
        i -= 1;
    }

    {
        let beg = 1;
        let end = if l_ref < bw + 1 { l_ref } else { bw + 1 };
        let mut sum = 0.0;
        k = end;
        while k >= beg {
            let e = if *ref_.add((k - 1) as usize) > 3 || *query > 3 {
                1.0
            } else if *ref_.add((k - 1) as usize) == *query {
                1.0 - qual[0] as f64
            } else {
                qual[0] as f64 * EM
            };
            let u = set_u!(bw, 1, k);
            if !(u < 3 || u as usize >= i_dim) {
                sum += e * b[i_dim + u as usize] * bM + EI * b[i_dim + u as usize + 1] * bI;
            }
            if k == beg {
                break;
            }
            k -= 1;
        }
        k = set_u!(bw, 0, 0);
        b[k as usize] = sum / s[0];
    }

    i = 1;
    while i <= l_query {
        let fi = i as usize * i_dim;
        let bi = i as usize * i_dim;
        let mut sum = 0.0;
        let mut max = 0.0;
        let mut beg = 1;
        let mut end = l_ref;
        let mut max_k = -1;
        let x = i - bw;
        if beg < x {
            beg = x;
        }
        let x = i + bw;
        if end > x {
            end = x;
        }
        let mm = 1.0 / s[i as usize];
        let mut u = set_u!(bw, i, beg);
        k = beg;
        while k <= end {
            let uu = u as usize;
            let z1 = mm * f[fi + uu] * b[bi + uu];
            let z2 = mm * f[fi + uu + 1] * b[bi + uu + 1];
            let which = (z2 > z1) as c_int;
            let zm = if which != 0 { z2 } else { z1 };
            if zm > max {
                max = zm;
                max_k = ((k - 1) << 2) | which;
            }
            sum += z1 + z2;
            k += 1;
            u += 3;
        }
        max /= sum;
        sum *= s[i as usize];
        if !state.is_null() {
            *state.add((i - 1) as usize) = max_k;
        }
        if !q.is_null() {
            k = (-4.343 * (1.0 - max).ln() + 0.499) as c_int;
            *q.add((i - 1) as usize) = if k > 100 { 99 } else { k as u8 };
        }
        let _ = sum;
        i += 1;
    }

    Pr
}

#[cfg(test)]
mod tests {
    use super::*;

    type ProbalnCase<'a> = (&'a [u8], &'a [u8], &'a [u8], c_int);

    fn encode_bases(seq: &[u8]) -> Vec<u8> {
        seq.iter()
            .map(|b| match *b {
                b'a' | b'A' => 0,
                b'c' | b'C' => 1,
                b'g' | b'G' => 2,
                b't' | b'T' => 3,
                _ => 4,
            })
            .collect()
    }

    #[test]
    fn probaln_parameter_layout_matches_public_abi() {
        assert_eq!(std::mem::size_of::<probaln_par_t>(), 12);
        assert_eq!(std::mem::align_of::<probaln_par_t>(), 4);
        assert_eq!(std::mem::offset_of!(probaln_par_t, d), 0);
        assert_eq!(std::mem::offset_of!(probaln_par_t, e), 4);
        assert_eq!(std::mem::offset_of!(probaln_par_t, bw), 8);
    }

    #[test]
    fn probaln_glocal_matches_htslib_likelihood_only() {
        unsafe {
            let cases: &[ProbalnCase<'_>] = &[
                (b"ACTTC", b"ATTC", &[30, 30, 30, 30], 10),
                (b"ACGTACGTNNACGT", b"ACGTCGTNACGT", &[20; 12], 7),
                (
                    b"TTTTACGTACGTAAAA",
                    b"ACGTTCGT",
                    &[10, 15, 20, 25, 30, 35, 40, 30],
                    5,
                ),
            ];
            for (ref_s, query_s, qual, bw) in cases {
                let ref_ = encode_bases(ref_s);
                let query = encode_bases(query_s);
                let par = probaln_par_t {
                    d: 0.001,
                    e: 0.1,
                    bw: *bw,
                };
                let rust_score = probaln_glocal(
                    ref_.as_ptr(),
                    ref_.len() as c_int,
                    query.as_ptr(),
                    query.len() as c_int,
                    qual.as_ptr(),
                    &par,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                let c_score = hts_sys::probaln_glocal(
                    ref_.as_ptr(),
                    ref_.len() as c_int,
                    query.as_ptr(),
                    query.len() as c_int,
                    qual.as_ptr(),
                    (&par as *const probaln_par_t).cast(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                assert_eq!(rust_score, c_score, "case {ref_s:?} {query_s:?}");
            }
        }
    }

    #[test]
    fn probaln_glocal_rejects_negative_lengths_before_dereferencing() {
        unsafe {
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 10,
            };
            *crate::htslib_rs::c_compat::__errno_location() = 0;
            assert_eq!(
                probaln_glocal(
                    std::ptr::null(),
                    -1,
                    std::ptr::null(),
                    1,
                    std::ptr::null(),
                    &par,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                c_int::MIN
            );
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );

            *crate::htslib_rs::c_compat::__errno_location() = 0;
            assert_eq!(
                probaln_glocal(
                    std::ptr::null(),
                    1,
                    std::ptr::null(),
                    -1,
                    std::ptr::null(),
                    &par,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                c_int::MIN
            );
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );
        }
    }

    #[test]
    fn probaln_glocal_rejects_overflow_prone_query_length() {
        unsafe {
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 10,
            };
            *crate::htslib_rs::c_compat::__errno_location() = 0;
            assert_eq!(
                probaln_glocal(
                    std::ptr::null(),
                    1,
                    std::ptr::null(),
                    c_int::MAX - 2,
                    std::ptr::null(),
                    &par,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                c_int::MIN
            );
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );
        }
    }

    #[test]
    fn probaln_glocal_returns_zero_for_empty_dimensions() {
        unsafe {
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 10,
            };
            assert_eq!(
                probaln_glocal(
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    4,
                    std::ptr::null(),
                    &par,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                0
            );
            assert_eq!(
                probaln_glocal(
                    std::ptr::null(),
                    4,
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    &par,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                0
            );
        }
    }

    #[test]
    fn probaln_glocal_empty_dimensions_leave_outputs_untouched() {
        unsafe {
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 10,
            };
            let mut state = [123];
            let mut q = [45];
            assert_eq!(
                probaln_glocal(
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    &par,
                    state.as_mut_ptr(),
                    q.as_mut_ptr(),
                ),
                0
            );
            assert_eq!(state, [123]);
            assert_eq!(q, [45]);
        }
    }

    #[test]
    fn probaln_glocal_widens_band_to_length_difference() {
        unsafe {
            let ref_ = encode_bases(b"ACGTACGTACGT");
            let query = encode_bases(b"ACGT");
            let par_narrow = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 0,
            };
            let par_wide = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: (ref_.len() - query.len()) as c_int,
            };

            assert_eq!(
                probaln_glocal(
                    ref_.as_ptr(),
                    ref_.len() as c_int,
                    query.as_ptr(),
                    query.len() as c_int,
                    std::ptr::null(),
                    &par_narrow,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                probaln_glocal(
                    ref_.as_ptr(),
                    ref_.len() as c_int,
                    query.as_ptr(),
                    query.len() as c_int,
                    std::ptr::null(),
                    &par_wide,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            );
        }
    }

    #[test]
    fn probaln_glocal_does_not_enter_backward_pass_with_one_output_buffer() {
        unsafe {
            let ref_ = encode_bases(b"ACGTACGT");
            let query = encode_bases(b"ACGT");
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 6,
            };
            let mut state = [-7; 4];
            let score_with_state_only = probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                std::ptr::null(),
                &par,
                state.as_mut_ptr(),
                std::ptr::null_mut(),
            );
            let score_without_outputs = probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                std::ptr::null(),
                &par,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            assert_eq!(score_with_state_only, score_without_outputs);
            assert_eq!(state, [-7; 4]);
        }
    }

    #[test]
    fn probaln_glocal_does_not_enter_backward_pass_with_q_only() {
        unsafe {
            let ref_ = encode_bases(b"ACGTACGT");
            let query = encode_bases(b"ACGT");
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 6,
            };
            let mut q = [77; 4];
            let score_with_q_only = probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                std::ptr::null(),
                &par,
                std::ptr::null_mut(),
                q.as_mut_ptr(),
            );
            let score_without_outputs = probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                std::ptr::null(),
                &par,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            assert_eq!(score_with_q_only, score_without_outputs);
            assert_eq!(q, [77; 4]);
        }
    }

    #[test]
    fn probaln_glocal_null_quality_matches_explicit_q30() {
        unsafe {
            let ref_ = encode_bases(b"ACGTNNACGT");
            let query = encode_bases(b"ACGTTACGT");
            let qual = [30; 9];
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 7,
            };

            assert_eq!(
                probaln_glocal(
                    ref_.as_ptr(),
                    ref_.len() as c_int,
                    query.as_ptr(),
                    query.len() as c_int,
                    std::ptr::null(),
                    &par,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                probaln_glocal(
                    ref_.as_ptr(),
                    ref_.len() as c_int,
                    query.as_ptr(),
                    query.len() as c_int,
                    qual.as_ptr(),
                    &par,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            );
        }
    }

    #[test]
    fn probaln_glocal_ambiguous_bases_match_htslib_outputs() {
        unsafe {
            let ref_ = encode_bases(b"NNNN");
            let query = encode_bases(b"NN");
            let qual = [0, 255];
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 2,
            };
            let mut rust_state = [-1; 2];
            let mut rust_q = [255; 2];
            let mut c_state = [-1; 2];
            let mut c_q = [255; 2];

            let rust_score = probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                qual.as_ptr(),
                &par,
                rust_state.as_mut_ptr(),
                rust_q.as_mut_ptr(),
            );
            let c_score = hts_sys::probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                qual.as_ptr(),
                (&par as *const probaln_par_t).cast(),
                c_state.as_mut_ptr(),
                c_q.as_mut_ptr(),
            );

            assert_eq!(rust_score, c_score);
            assert_eq!(rust_state, c_state);
            assert_eq!(rust_q, c_q);
        }
    }

    #[test]
    fn probaln_glocal_accepts_highest_quality_table_index() {
        unsafe {
            let ref_ = encode_bases(b"ACGTACGT");
            let query = encode_bases(b"ACGT");
            let qual = [255; 4];
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 6,
            };

            assert_ne!(
                probaln_glocal(
                    ref_.as_ptr(),
                    ref_.len() as c_int,
                    query.as_ptr(),
                    query.len() as c_int,
                    qual.as_ptr(),
                    &par,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                c_int::MIN
            );
        }
    }

    #[test]
    fn probaln_glocal_populates_backward_outputs_when_both_buffers_are_present() {
        unsafe {
            let ref_ = encode_bases(b"ACGTACGT");
            let query = encode_bases(b"ACGT");
            let qual = [30; 4];
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 6,
            };
            let mut state = [-1; 4];
            let mut q = [255; 4];

            let score = probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                qual.as_ptr(),
                &par,
                state.as_mut_ptr(),
                q.as_mut_ptr(),
            );

            assert_ne!(score, c_int::MIN);
            assert!(state.iter().all(|&s| s >= 0));
            assert!(q.iter().all(|&posterior| posterior <= 99));
        }
    }

    #[test]
    fn probaln_glocal_matches_htslib_state_and_posterior() {
        unsafe {
            let ref_ = encode_bases(b"ACGTACGTNNACGTACGT");
            let query = encode_bases(b"ACGTTGTNACGT");
            let qual = [18, 22, 25, 30, 30, 35, 35, 20, 25, 30, 32, 34];
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 8,
            };
            let mut rust_state = vec![0; query.len()];
            let mut rust_q = vec![0; query.len()];
            let mut c_state = vec![0; query.len()];
            let mut c_q = vec![0; query.len()];
            let rust_score = probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                qual.as_ptr(),
                &par,
                rust_state.as_mut_ptr(),
                rust_q.as_mut_ptr(),
            );
            let c_score = hts_sys::probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                qual.as_ptr(),
                (&par as *const probaln_par_t).cast(),
                c_state.as_mut_ptr(),
                c_q.as_mut_ptr(),
            );
            assert_eq!(rust_score, c_score);
            assert_eq!(rust_state, c_state);
            assert_eq!(rust_q, c_q);
        }
    }

    #[test]
    fn probaln_glocal_matches_htslib_high_quality_table_path() {
        unsafe {
            let ref_ = encode_bases(b"ACGTACGTACGTACGTNNNNACGTACGT");
            let query = encode_bases(b"ACGTTCGTACGTNNACGT");
            let qual = [
                255, 254, 253, 252, 251, 250, 45, 44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34,
            ];
            let par = probaln_par_t {
                d: 0.001,
                e: 0.1,
                bw: 10,
            };
            let mut rust_state = vec![0; query.len()];
            let mut rust_q = vec![0; query.len()];
            let mut c_state = vec![0; query.len()];
            let mut c_q = vec![0; query.len()];
            let rust_score = probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                qual.as_ptr(),
                &par,
                rust_state.as_mut_ptr(),
                rust_q.as_mut_ptr(),
            );
            let c_score = hts_sys::probaln_glocal(
                ref_.as_ptr(),
                ref_.len() as c_int,
                query.as_ptr(),
                query.len() as c_int,
                qual.as_ptr(),
                (&par as *const probaln_par_t).cast(),
                c_state.as_mut_ptr(),
                c_q.as_mut_ptr(),
            );
            assert_eq!(rust_score, c_score);
            assert_eq!(rust_state, c_state);
            assert_eq!(rust_q, c_q);
        }
    }
}

// original: probaln_glocal (htslib/probaln.c:77)
#[allow(clippy::items_after_test_module, clippy::too_many_arguments)]
pub unsafe fn probaln_c_77_probaln_glocal(
    ref_: *const u8,
    l_ref: c_int,
    query: *const u8,
    l_query: c_int,
    iqual: *const u8,
    c: *const probaln_par_t,
    state: *mut c_int,
    q: *mut u8,
) -> c_int {
    probaln_glocal(ref_, l_ref, query, l_query, iqual, c, state, q)
}

// original: main (htslib/probaln.c:438)
#[cfg(unix)]
pub unsafe fn probaln_c_438_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut conv = [4u8; 256];
    let mut par = probaln_par_t {
        d: 0.001,
        e: 0.1,
        bw: 10,
    };
    let mut q = 30;
    let mut b = 10;

    loop {
        let c = libc::getopt(argc, argv, c"b:q:".as_ptr());
        if c < 0 {
            break;
        }
        match c {
            c if c == b'b' as c_int => b = libc::atoi(optarg),
            c if c == b'q' as c_int => q = libc::atoi(optarg),
            _ => {}
        }
    }
    if optind + 2 > argc {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Usage: %s [-q %d] [-b %d] <ref> <query>\n".as_ptr(),
            *argv,
            q,
            b,
        );
        return 1;
    }

    conv[b'a' as usize] = 0;
    conv[b'A' as usize] = 0;
    conv[b'c' as usize] = 1;
    conv[b'C' as usize] = 1;
    conv[b'g' as usize] = 2;
    conv[b'G' as usize] = 2;
    conv[b't' as usize] = 3;
    conv[b'T' as usize] = 3;

    let ref_ = *argv.add(optind as usize) as *mut u8;
    let query = *argv.add(optind as usize + 1) as *mut u8;
    let l_ref = libc::strlen(ref_.cast()) as c_int;
    let l_query = libc::strlen(query.cast()) as c_int;
    let mut i = 0;
    while i < l_ref {
        *ref_.add(i as usize) = conv[*ref_.add(i as usize) as usize];
        i += 1;
    }
    i = 0;
    while i < l_query {
        *query.add(i as usize) = conv[*query.add(i as usize) as usize];
        i += 1;
    }

    let iqual = libc::malloc(l_query as usize).cast::<u8>();
    libc::memset(iqual.cast(), q, l_query as usize);
    par.bw = b;
    let p = probaln_glocal(
        ref_,
        l_ref,
        query,
        l_query,
        iqual,
        &par,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    libc::fprintf(
        crate::htslib_rs::c_compat::stderr.cast(),
        c"%d\n".as_ptr(),
        p,
    );
    libc::free(iqual.cast());
    0
}
