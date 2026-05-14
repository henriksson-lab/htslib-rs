use std::ffi::c_int;

const KF_GAMMA_EPS: f64 = 1e-14;
const KF_TINY: f64 = 1e-290;

extern "C" {
    fn lgamma(x: f64) -> f64;
    fn erfc(x: f64) -> f64;
}

#[repr(C)]
pub struct hgacc_t {
    pub n11: c_int,
    pub n1_: c_int,
    pub n_1: c_int,
    pub n: c_int,
    pub p: f64,
}

pub fn kf_lgamma(z: f64) -> f64 {
    let mut x = 0.0;
    x += 0.1659470187408462e-06 / (z + 7.0);
    x += 0.9934937113930748e-05 / (z + 6.0);
    x -= 0.1385710331296526 / (z + 5.0);
    x += 12.50734324009056 / (z + 4.0);
    x -= 176.6150291498386 / (z + 3.0);
    x += 771.3234287757674 / (z + 2.0);
    x -= 1259.139216722289 / (z + 1.0);
    x += 676.5203681218835 / z;
    x += 0.9999999999995183;
    x.ln() - 5.58106146679532777 - z + (z - 0.5) * (z + 6.5).ln()
}

pub fn kf_erfc(x: f64) -> f64 {
    let p0 = 220.2068679123761;
    let p1 = 221.2135961699311;
    let p2 = 112.0792914978709;
    let p3 = 33.912866078383;
    let p4 = 6.37396220353165;
    let p5 = 0.7003830644436881;
    let p6 = 0.03526249659989109;
    let q0 = 440.4137358247522;
    let q1 = 793.8265125199484;
    let q2 = 637.3336333788311;
    let q3 = 296.5642487796737;
    let q4 = 86.78073220294608;
    let q5 = 16.06417757920695;
    let q6 = 1.755667163182642;
    let q7 = 0.08838834764831844;
    let z = x.abs() * std::f64::consts::SQRT_2;
    if z > 37.0 {
        return if x > 0.0 { 0.0 } else { 2.0 };
    }
    let expntl = (z * z * -0.5).exp();
    let p = if z < 10.0 / std::f64::consts::SQRT_2 {
        expntl * ((((((p6 * z + p5) * z + p4) * z + p3) * z + p2) * z + p1) * z + p0)
            / (((((((q7 * z + q6) * z + q5) * z + q4) * z + q3) * z + q2) * z + q1) * z + q0)
    } else {
        expntl / 2.506628274631001 / (z + 1.0 / (z + 2.0 / (z + 3.0 / (z + 4.0 / (z + 0.65)))))
    };
    if x > 0.0 {
        2.0 * p
    } else {
        2.0 * (1.0 - p)
    }
}

pub fn _kf_gammap(s: f64, z: f64) -> f64 {
    let mut sum = 1.0;
    let mut x = 1.0;
    let mut k = 1;
    while k < 100 {
        x *= z / (s + k as f64);
        sum += x;
        if x / sum < KF_GAMMA_EPS {
            break;
        }
        k += 1;
    }
    (s * z.ln() - z - kf_lgamma(s + 1.0) + sum.ln()).exp()
}

pub fn _kf_gammaq(s: f64, z: f64) -> f64 {
    let mut f = 1.0 + z - s;
    let mut c = f;
    let mut d = 0.0;
    let mut j = 1;
    while j < 100 {
        let a = j as f64 * (s - j as f64);
        let b = ((j << 1) + 1) as f64 + z - s;
        d = b + a * d;
        if d < KF_TINY {
            d = KF_TINY;
        }
        c = b + a / c;
        if c < KF_TINY {
            c = KF_TINY;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;
        if (delta - 1.0).abs() < KF_GAMMA_EPS {
            break;
        }
        j += 1;
    }
    (s * z.ln() - z - kf_lgamma(s) - f.ln()).exp()
}

pub fn kf_gammap(s: f64, z: f64) -> f64 {
    if z <= 1.0 || z < s {
        _kf_gammap(s, z)
    } else {
        1.0 - _kf_gammaq(s, z)
    }
}

pub fn kf_gammaq(s: f64, z: f64) -> f64 {
    if z <= 1.0 || z < s {
        1.0 - _kf_gammap(s, z)
    } else {
        _kf_gammaq(s, z)
    }
}

pub fn kf_betai_aux(a: f64, b: f64, x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    let mut f = 1.0;
    let mut c = f;
    let mut d = 0.0;
    let mut j = 1;
    while j < 200 {
        let m = j >> 1;
        let aa = if (j & 1) != 0 {
            -(a + m as f64) * (a + b + m as f64) * x
                / ((a + 2.0 * m as f64) * (a + 2.0 * m as f64 + 1.0))
        } else {
            m as f64 * (b - m as f64) * x / ((a + 2.0 * m as f64 - 1.0) * (a + 2.0 * m as f64))
        };
        d = 1.0 + aa * d;
        if d < KF_TINY {
            d = KF_TINY;
        }
        c = 1.0 + aa / c;
        if c < KF_TINY {
            c = KF_TINY;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;
        if (delta - 1.0).abs() < KF_GAMMA_EPS {
            break;
        }
        j += 1;
    }
    (kf_lgamma(a + b) - kf_lgamma(a) - kf_lgamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp() / a / f
}

pub fn kf_betai(a: f64, b: f64, x: f64) -> f64 {
    if x < (a + 1.0) / (a + b + 2.0) {
        kf_betai_aux(a, b, x)
    } else {
        1.0 - kf_betai_aux(b, a, 1.0 - x)
    }
}

pub fn lbinom(n: c_int, k: c_int) -> f64 {
    if k == 0 || n == k {
        return 0.0;
    }
    unsafe { lgamma((n + 1) as f64) - lgamma((k + 1) as f64) - lgamma((n - k + 1) as f64) }
}

pub fn hypergeo(n11: c_int, n1_: c_int, n_1: c_int, n: c_int) -> f64 {
    (lbinom(n1_, n11) + lbinom(n - n1_, n_1 - n11) - lbinom(n, n_1)).exp()
}

pub unsafe fn hypergeo_acc(n11: c_int, n1_: c_int, n_1: c_int, n: c_int, aux: *mut hgacc_t) -> f64 {
    if n1_ != 0 || n_1 != 0 || n != 0 {
        (*aux).n11 = n11;
        (*aux).n1_ = n1_;
        (*aux).n_1 = n_1;
        (*aux).n = n;
    } else {
        if n11 % 11 != 0 && n11 + (*aux).n - (*aux).n1_ - (*aux).n_1 != 0 {
            if n11 == (*aux).n11 + 1 {
                (*aux).p *= ((*aux).n1_ - (*aux).n11) as f64 / n11 as f64
                    * ((*aux).n_1 - (*aux).n11) as f64
                    / (n11 + (*aux).n - (*aux).n1_ - (*aux).n_1) as f64;
                (*aux).n11 = n11;
                return (*aux).p;
            }
            if n11 == (*aux).n11 - 1 {
                (*aux).p *= (*aux).n11 as f64 / ((*aux).n1_ - n11) as f64
                    * ((*aux).n11 + (*aux).n - (*aux).n1_ - (*aux).n_1) as f64
                    / ((*aux).n_1 - n11) as f64;
                (*aux).n11 = n11;
                return (*aux).p;
            }
        }
        (*aux).n11 = n11;
    }
    (*aux).p = hypergeo((*aux).n11, (*aux).n1_, (*aux).n_1, (*aux).n);
    (*aux).p
}

pub unsafe fn kt_fisher_exact(
    n11: c_int,
    n12: c_int,
    n21: c_int,
    n22: c_int,
    _left: *mut f64,
    _right: *mut f64,
    two: *mut f64,
) -> f64 {
    let n1_ = n11 + n12;
    let n_1 = n11 + n21;
    let n = n11 + n12 + n21 + n22;
    let max = if n_1 < n1_ { n_1 } else { n1_ };
    let mut min = n1_ + n_1 - n;
    if min < 0 {
        min = 0;
    }
    *two = 1.0;
    *_left = 1.0;
    *_right = 1.0;
    if min == max {
        return 1.0;
    }

    let mut aux = hgacc_t {
        n11: 0,
        n1_: 0,
        n_1: 0,
        n: 0,
        p: 0.0,
    };
    let q = hypergeo_acc(n11, n1_, n_1, n, &mut aux);
    if q == 0.0 {
        if (n11 as i64) * (n as i64 + 2) < (n_1 as i64 + 1) * (n1_ as i64 + 1) {
            *_left = 0.0;
            *_right = 1.0;
            *two = 0.0;
            return 0.0;
        } else {
            *_left = 1.0;
            *_right = 0.0;
            *two = 0.0;
            return 0.0;
        }
    }

    let mut p = hypergeo_acc(min, 0, 0, 0, &mut aux);
    let mut left = 0.0;
    let mut i = min + 1;
    while p < 0.99999999 * q && i <= max {
        left += p;
        p = hypergeo_acc(i, 0, 0, 0, &mut aux);
        i += 1;
    }
    i -= 1;
    if p < 1.00000001 * q {
        left += p;
    } else {
        i -= 1;
    }

    p = hypergeo_acc(max, 0, 0, 0, &mut aux);
    let mut right = 0.0;
    let mut j = max - 1;
    while p < 0.99999999 * q && j >= 0 {
        right += p;
        p = hypergeo_acc(j, 0, 0, 0, &mut aux);
        j -= 1;
    }
    j += 1;
    if p < 1.00000001 * q {
        right += p;
    } else {
        j += 1;
    }

    *two = left + right;
    if *two > 1.0 {
        *two = 1.0;
    }
    if (i - n11).abs() < (j - n11).abs() {
        right = 1.0 - left + q;
    } else {
        left = 1.0 - right + q;
    }
    *_left = left;
    *_right = right;
    q
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kfunc_special_functions_match_reference_identities() {
        assert!((kf_lgamma(5.5) - unsafe { lgamma(5.5) }).abs() < 1e-10);
        assert!((kf_erfc(1.25) - unsafe { erfc(1.25) }).abs() < 1e-10);

        let p = kf_gammap(3.0, 5.5);
        let q = kf_gammaq(3.0, 5.5);
        assert!((p + q - 1.0).abs() < 1e-12);

        assert!((kf_betai(2.0, 2.0, 0.5) - 0.5).abs() < 1e-12);
        assert_eq!(lbinom(10, 0), 0.0);
        assert_eq!(lbinom(10, 10), 0.0);
    }

    #[test]
    fn fisher_exact_returns_expected_tail_probabilities() {
        unsafe {
            let mut left = 0.0;
            let mut right = 0.0;
            let mut two = 0.0;
            let q = kt_fisher_exact(1, 9, 11, 3, &mut left, &mut right, &mut two);
            assert!((q - 0.001346076187912236).abs() < 1e-15);
            assert!((left - 0.0013797280926100418).abs() < 1e-15);
            assert!((right - 0.9999663480953022).abs() < 1e-15);
            assert!((two - 0.0027594561852200836).abs() < 1e-15);
        }
    }
}
