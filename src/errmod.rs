use std::ffi::c_int;

use crate::htslib_mini_rs::{c_compat, kfunc::lbinom};

#[repr(C)]
pub struct errmod_t {
    pub depcorr: f64,
    pub fk: *mut f64,
    pub beta: *mut f64,
    pub lhet: *mut f64,
}

pub fn errmod_t() {}

#[repr(C)]
struct call_aux_t {
    fsum: [f64; 16],
    bsum: [f64; 16],
    c: [u32; 16],
}

pub unsafe fn logbinomial_table(n_size: c_int) -> *mut f64 {
    let logbinom =
        c_compat::calloc((n_size * n_size) as u64, std::mem::size_of::<f64>() as u64).cast::<f64>();
    if logbinom.is_null() {
        return std::ptr::null_mut();
    }
    let mut n = 1;
    while n < n_size {
        let mut k = 1;
        while k <= n {
            *logbinom.add(((n << 8) | k) as usize) = lbinom(n, k);
            k += 1;
        }
        n += 1;
    }
    logbinom
}

pub unsafe fn cal_coef(em: *mut errmod_t, depcorr: f64, eta: f64) -> c_int {
    (*em).fk = c_compat::calloc(256, std::mem::size_of::<f64>() as u64).cast::<f64>();
    if (*em).fk.is_null() {
        return -1;
    }
    *(*em).fk.add(0) = 1.0;
    let mut n = 1;
    while n < 256 {
        *(*em).fk.add(n as usize) = (1.0 - depcorr).powi(n) * (1.0 - eta) + eta;
        n += 1;
    }

    (*em).beta = c_compat::calloc(256 * 256 * 64, std::mem::size_of::<f64>() as u64).cast::<f64>();
    if (*em).beta.is_null() {
        return -1;
    }

    let lc = logbinomial_table(256);
    if lc.is_null() {
        return -1;
    }

    let mut q = 1;
    while q < 64 {
        let e = 10.0_f64.powf(-(q as f64) / 10.0);
        let le = e.ln();
        let le1 = (1.0 - e).ln();
        n = 1;
        while n <= 255 {
            let beta = (*em).beta.add(((q << 16) | (n << 8)) as usize);
            let mut sum1 = *lc.add(((n << 8) | n) as usize) + n as f64 * le;
            *beta.add(n as usize) = f64::INFINITY;
            let mut k = n - 1;
            loop {
                let sum = sum1
                    + (*lc.add(((n << 8) | k) as usize) + k as f64 * le + (n - k) as f64 * le1
                        - sum1)
                        .exp()
                        .ln_1p();
                *beta.add(k as usize) = -10.0 / std::f64::consts::LN_10 * (sum1 - sum);
                sum1 = sum;
                if k == 0 {
                    break;
                }
                k -= 1;
            }
            n += 1;
        }
        q += 1;
    }

    (*em).lhet = c_compat::calloc(256 * 256, std::mem::size_of::<f64>() as u64).cast::<f64>();
    if (*em).lhet.is_null() {
        c_compat::free(lc.cast());
        return -1;
    }
    n = 0;
    while n < 256 {
        let mut k = 0;
        while k < 256 {
            *(*em).lhet.add(((n << 8) | k) as usize) =
                *lc.add(((n << 8) | k) as usize) - std::f64::consts::LN_2 * n as f64;
            k += 1;
        }
        n += 1;
    }
    c_compat::free(lc.cast());
    0
}

pub unsafe fn errmod_init(depcorr: f64) -> *mut errmod_t {
    let em = c_compat::calloc(1, std::mem::size_of::<errmod_t>() as u64).cast::<errmod_t>();
    if em.is_null() {
        return std::ptr::null_mut();
    }
    (*em).depcorr = depcorr;
    cal_coef(em, depcorr, 0.03);
    em
}

pub unsafe fn errmod_destroy(em: *mut errmod_t) {
    if em.is_null() {
        return;
    }
    c_compat::free((*em).lhet.cast());
    c_compat::free((*em).fk.cast());
    c_compat::free((*em).beta.cast());
    c_compat::free(em.cast());
}

pub unsafe fn errmod_cal(
    em: *const errmod_t,
    mut n: c_int,
    m: c_int,
    bases: *mut u16,
    q: *mut f32,
) -> c_int {
    std::ptr::write_bytes(q, 0, (m * m) as usize);
    if n == 0 {
        return 0;
    }
    if n > 255 {
        let mut i = n - 1;
        while i > 0 {
            let j = (libc::drand48() * (i + 1) as f64) as c_int;
            let tmp = *bases.add(i as usize);
            *bases.add(i as usize) = *bases.add(j as usize);
            *bases.add(j as usize) = tmp;
            i -= 1;
        }
        n = 255;
    }

    let slice = std::slice::from_raw_parts_mut(bases, n as usize);
    slice.sort_unstable();

    let mut w = [0i32; 32];
    let mut aux = call_aux_t {
        fsum: [0.0; 16],
        bsum: [0.0; 16],
        c: [0; 16],
    };

    let mut j = n - 1;
    loop {
        let b = *bases.add(j as usize);
        let mut qual = if (b >> 5) < 4 { 4 } else { (b >> 5) as c_int };
        if qual > 63 {
            qual = 63;
        }
        let basestrand = (b & 0x1f) as usize;
        let base = (b & 0xf) as usize;
        aux.fsum[base] += *(*em).fk.add(w[basestrand] as usize);
        aux.bsum[base] += *(*em).fk.add(w[basestrand] as usize)
            * *(*em)
                .beta
                .add(((qual << 16) | (n << 8) | aux.c[base] as c_int) as usize);
        aux.c[base] += 1;
        w[basestrand] += 1;
        if j == 0 {
            break;
        }
        j -= 1;
    }

    j = 0;
    while j < m {
        let mut k = 0;
        let mut tmp1 = 0.0f32;
        let mut tmp2 = 0;
        while k < m {
            if k != j {
                tmp1 += aux.bsum[k as usize] as f32;
                tmp2 += aux.c[k as usize] as c_int;
            }
            k += 1;
        }
        if tmp2 != 0 {
            *q.add((j * m + j) as usize) = tmp1;
        }

        k = j + 1;
        while k < m {
            let cjk = aux.c[j as usize] + aux.c[k as usize];
            let mut i = 0;
            tmp1 = 0.0;
            tmp2 = 0;
            while i < m {
                if i != j && i != k {
                    tmp1 += aux.bsum[i as usize] as f32;
                    tmp2 += aux.c[i as usize] as c_int;
                }
                i += 1;
            }
            let val = -4.343f32 * *(*em).lhet.add(((cjk << 8) | aux.c[k as usize]) as usize) as f32;
            if tmp2 != 0 {
                *q.add((j * m + k) as usize) = val + tmp1;
                *q.add((k * m + j) as usize) = val + tmp1;
            } else {
                *q.add((j * m + k) as usize) = val;
                *q.add((k * m + j) as usize) = val;
            }
            k += 1;
        }

        k = 0;
        while k < m {
            let v = q.add((j * m + k) as usize);
            if *v < 0.0 {
                *v = 0.0;
            }
            k += 1;
        }
        j += 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errmod_init_calculates_likelihoods_and_destroy_frees() {
        unsafe {
            let em = errmod_init(0.1);
            assert!(!em.is_null());
            assert!(!(*em).fk.is_null());
            assert!(!(*em).beta.is_null());
            assert!(!(*em).lhet.is_null());
            assert_eq!((*em).depcorr, 0.1);

            let mut bases = [
                (30u16 << 5) | 0,
                (25u16 << 5) | 0,
                (20u16 << 5) | 1,
                (35u16 << 5) | 17,
            ];
            let mut q = [0.0f32; 16];
            assert_eq!(
                errmod_cal(
                    em,
                    bases.len() as c_int,
                    4,
                    bases.as_mut_ptr(),
                    q.as_mut_ptr()
                ),
                0
            );
            assert_eq!(
                bases,
                [
                    (20u16 << 5) | 1,
                    (25u16 << 5) | 0,
                    (30u16 << 5) | 0,
                    (35u16 << 5) | 17,
                ]
            );
            assert!(q.iter().all(|v| *v >= 0.0));
            assert!(q.iter().any(|v| *v > 0.0));
            errmod_destroy(em);
            errmod_destroy(std::ptr::null_mut());
        }
    }
}
