use std::ffi::c_int;

use crate::htslib_rs::{c_compat, kfunc::lbinom, os_rand::hts_drand48};

pub struct errmod_t {
    pub depcorr: f64,
    pub fk: Vec<f64>,
    pub beta: Vec<f64>,
    pub lhet: Vec<f64>,
}

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

pub unsafe fn cal_coef(em: &mut errmod_t, depcorr: f64, eta: f64) -> c_int {
    let mut fk = vec![0.0; 256];
    fk[0] = 1.0;
    let mut n = 1;
    while n < 256 {
        fk[n as usize] = (1.0 - depcorr).powi(n) * (1.0 - eta) + eta;
        n += 1;
    }

    let mut beta = vec![0.0; 256 * 256 * 64];

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
            let beta_row = ((q << 16) | (n << 8)) as usize;
            let mut sum1 = *lc.add(((n << 8) | n) as usize) + n as f64 * le;
            beta[beta_row + n as usize] = f64::INFINITY;
            let mut k = n - 1;
            loop {
                let sum = sum1
                    + (*lc.add(((n << 8) | k) as usize) + k as f64 * le + (n - k) as f64 * le1
                        - sum1)
                        .exp()
                        .ln_1p();
                beta[beta_row + k as usize] = -10.0 / std::f64::consts::LN_10 * (sum1 - sum);
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

    let mut lhet = vec![0.0; 256 * 256];
    n = 0;
    while n < 256 {
        let mut k = 0;
        while k < 256 {
            lhet[((n << 8) | k) as usize] =
                *lc.add(((n << 8) | k) as usize) - std::f64::consts::LN_2 * n as f64;
            k += 1;
        }
        n += 1;
    }
    c_compat::free(lc.cast());
    em.depcorr = depcorr;
    em.fk = fk;
    em.beta = beta;
    em.lhet = lhet;
    0
}

pub unsafe fn errmod_init(depcorr: f64) -> *mut errmod_t {
    let mut em = Box::new(errmod_t {
        depcorr,
        fk: Vec::new(),
        beta: Vec::new(),
        lhet: Vec::new(),
    });
    if cal_coef(&mut em, depcorr, 0.03) != 0 {
        return std::ptr::null_mut();
    }
    Box::into_raw(em)
}

pub unsafe fn errmod_destroy(em: *mut errmod_t) {
    if em.is_null() {
        return;
    }
    drop(Box::from_raw(em));
}

pub unsafe fn errmod_cal(
    em: *const errmod_t,
    mut n: c_int,
    m: c_int,
    bases: *mut u16,
    q: *mut f32,
) -> c_int {
    let em = &*em;
    std::ptr::write_bytes(q, 0, (m * m) as usize);
    if n == 0 {
        return 0;
    }
    if n > 255 {
        let mut i = n - 1;
        while i > 0 {
            let j = (hts_drand48() * (i + 1) as f64) as c_int;
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
        aux.fsum[base] += em.fk[w[basestrand] as usize];
        aux.bsum[base] += em.fk[w[basestrand] as usize]
            * em.beta[((qual << 16) | (n << 8) | aux.c[base] as c_int) as usize];
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
                tmp1 = (tmp1 as f64 + aux.bsum[k as usize]) as f32;
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
                    tmp1 = (tmp1 as f64 + aux.bsum[i as usize]) as f32;
                    tmp2 += aux.c[i as usize] as c_int;
                }
                i += 1;
            }
            let val = -4.343f64 * em.lhet[((cjk << 8) | aux.c[k as usize]) as usize];
            if tmp2 != 0 {
                let out = (val + tmp1 as f64) as f32;
                *q.add((j * m + k) as usize) = out;
                *q.add((k * m + j) as usize) = out;
            } else {
                let out = val as f32;
                *q.add((j * m + k) as usize) = out;
                *q.add((k * m + j) as usize) = out;
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

    fn rand48_next(seed: [u16; 3]) -> [u16; 3] {
        let state = seed[0] as u64 | ((seed[1] as u64) << 16) | ((seed[2] as u64) << 32);
        let next = state.wrapping_mul(0x5deece66d).wrapping_add(0x0b) & ((1u64 << 48) - 1);
        [next as u16, (next >> 16) as u16, (next >> 32) as u16]
    }

    fn rand48_lrand(seed: [u16; 3]) -> libc::c_long {
        ((seed[2] as libc::c_long) << 15) + ((seed[1] as libc::c_long) >> 1)
    }

    #[test]
    fn errmod_init_calculates_likelihoods_and_destroy_frees() {
        unsafe {
            let em = errmod_init(0.1);
            assert!(!em.is_null());
            assert_eq!((*em).fk.len(), 256);
            assert_eq!((*em).beta.len(), 256 * 256 * 64);
            assert_eq!((*em).lhet.len(), 256 * 256);
            assert_eq!((*em).depcorr, 0.1);

            let mut bases = [30u16 << 5, 25u16 << 5, (20u16 << 5) | 1, (35u16 << 5) | 17];
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
                [(20u16 << 5) | 1, 25u16 << 5, 30u16 << 5, (35u16 << 5) | 17,]
            );
            assert!(q.iter().all(|v| *v >= 0.0));
            assert!(q.iter().any(|v| *v > 0.0));
            errmod_destroy(em);
            errmod_destroy(std::ptr::null_mut());
        }
    }

    #[test]
    fn logbinomial_table_populates_internal_triangle_only() {
        unsafe {
            let table = logbinomial_table(256);
            assert!(!table.is_null());
            assert_eq!(*table.add(1 << 8), 0.0);
            assert_eq!(*table.add((1 << 8) | 1), 0.0);
            assert_eq!(*table.add((7 << 8) | 7), 0.0);
            assert!((*table.add((7 << 8) | 3) - lbinom(7, 3)).abs() < 1e-12);
            assert!((*table.add((7 << 8) | 4) - lbinom(7, 4)).abs() < 1e-12);
            c_compat::free(table.cast());
        }
    }

    #[test]
    fn errmod_cal_zero_depth_only_clears_output_matrix() {
        unsafe {
            let em = errmod_init(0.0);
            assert!(!em.is_null());

            let mut bases = [(30u16 << 5) | 3];
            let original_bases = bases;
            let mut q = [7.0f32; 16];
            assert_eq!(errmod_cal(em, 0, 4, bases.as_mut_ptr(), q.as_mut_ptr()), 0);
            assert_eq!(bases, original_bases);
            assert_eq!(q, [0.0; 16]);

            errmod_destroy(em);
        }
    }

    #[test]
    fn errmod_cal_single_base_leaves_homozygous_call_at_zero() {
        unsafe {
            let em = errmod_init(0.0);
            assert!(!em.is_null());

            let mut bases = [(30u16 << 5) | 2];
            let mut q = [9.0f32; 16];
            assert_eq!(errmod_cal(em, 1, 4, bases.as_mut_ptr(), q.as_mut_ptr()), 0);
            assert_eq!(bases, [(30u16 << 5) | 2]);
            assert_eq!(q[2 * 4 + 2], 0.0);
            assert!(q[0] > 0.0);
            assert!(q[5] > 0.0);
            assert!(q[3 * 4 + 3] > 0.0);

            errmod_destroy(em);
        }
    }

    #[test]
    fn errmod_cal_clamps_quality_scores_to_supported_boundaries() {
        unsafe {
            let em = errmod_init(0.0);
            assert!(!em.is_null());

            let mut low_clamped = [0, (20u16 << 5) | 1, (30u16 << 5) | 2];
            let mut low_boundary = [4u16 << 5, (20u16 << 5) | 1, (30u16 << 5) | 2];
            let mut q_low_clamped = [0.0f32; 16];
            let mut q_low_boundary = [0.0f32; 16];
            assert_eq!(
                errmod_cal(
                    em,
                    low_clamped.len() as c_int,
                    4,
                    low_clamped.as_mut_ptr(),
                    q_low_clamped.as_mut_ptr()
                ),
                0
            );
            assert_eq!(
                errmod_cal(
                    em,
                    low_boundary.len() as c_int,
                    4,
                    low_boundary.as_mut_ptr(),
                    q_low_boundary.as_mut_ptr()
                ),
                0
            );
            assert_eq!(q_low_clamped, q_low_boundary);

            let mut high_clamped = [63u16 << 5, (80u16 << 5) | 1, (30u16 << 5) | 2];
            let mut high_boundary = [63u16 << 5, (63u16 << 5) | 1, (30u16 << 5) | 2];
            let mut q_high_clamped = [0.0f32; 16];
            let mut q_high_boundary = [0.0f32; 16];
            assert_eq!(
                errmod_cal(
                    em,
                    high_clamped.len() as c_int,
                    4,
                    high_clamped.as_mut_ptr(),
                    q_high_clamped.as_mut_ptr()
                ),
                0
            );
            assert_eq!(
                errmod_cal(
                    em,
                    high_boundary.len() as c_int,
                    4,
                    high_boundary.as_mut_ptr(),
                    q_high_boundary.as_mut_ptr()
                ),
                0
            );
            assert_eq!(q_high_clamped, q_high_boundary);

            errmod_destroy(em);
        }
    }

    #[test]
    fn errmod_cal_strand_bit_does_not_change_base_index() {
        unsafe {
            let em = errmod_init(0.0);
            assert!(!em.is_null());

            let mut bases = [(30u16 << 5) | 17, (30u16 << 5) | 1];
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

            assert_eq!(bases, [(30u16 << 5) | 1, (30u16 << 5) | 17]);
            assert_eq!(q[5], 0.0);
            assert!(q[0] > 0.0);
            assert!(q[2 * 4 + 2] > 0.0);
            assert!(q[3 * 4 + 3] > 0.0);

            errmod_destroy(em);
        }
    }

    #[test]
    fn errmod_cal_only_clears_requested_square_matrix() {
        unsafe {
            let em = errmod_init(0.0);
            assert!(!em.is_null());

            let mut bases = [30u16 << 5, (30u16 << 5) | 1];
            let mut q = [7.0f32; 9];
            assert_eq!(errmod_cal(em, 2, 2, bases.as_mut_ptr(), q.as_mut_ptr()), 0);

            assert_eq!(q[4..], [7.0; 5]);
            assert_eq!(q[1], q[2]);
            assert!(q[..4].iter().all(|v| *v >= 0.0));

            errmod_destroy(em);
        }
    }

    #[test]
    fn errmod_cal_matches_htslib_outputs_bitwise_for_fixed_inputs() {
        unsafe {
            let cases: &[&[u16]] = &[
                &[30u16 << 5, 25u16 << 5, (20u16 << 5) | 1, (35u16 << 5) | 17],
                &[
                    2,
                    (4u16 << 5) | 18,
                    (63u16 << 5) | 3,
                    (80u16 << 5) | 19,
                    41u16 << 5,
                ],
            ];

            for &depcorr in &[0.0, 0.1, 0.25] {
                let rust_em = errmod_init(depcorr);
                let c_em = hts_sys::errmod_init(depcorr);
                assert!(!rust_em.is_null());
                assert!(!c_em.is_null());

                for input in cases {
                    let mut rust_bases = input.to_vec();
                    let mut c_bases = input.to_vec();
                    let mut rust_q = [0.0f32; 16];
                    let mut c_q = [0.0f32; 16];

                    assert_eq!(
                        errmod_cal(
                            rust_em,
                            rust_bases.len() as c_int,
                            4,
                            rust_bases.as_mut_ptr(),
                            rust_q.as_mut_ptr(),
                        ),
                        0
                    );
                    assert_eq!(
                        hts_sys::errmod_cal(
                            c_em,
                            c_bases.len() as c_int,
                            4,
                            c_bases.as_mut_ptr(),
                            c_q.as_mut_ptr(),
                        ),
                        0
                    );

                    assert_eq!(rust_bases, c_bases);
                    assert_eq!(
                        rust_q.map(f32::to_bits),
                        c_q.map(f32::to_bits),
                        "depcorr={depcorr} input={input:?}",
                    );
                }

                errmod_destroy(rust_em);
                hts_sys::errmod_destroy(c_em);
            }
        }
    }

    #[test]
    fn errmod_cal_downsampling_uses_htslib_rand48_state() {
        unsafe {
            let _guard = crate::htslib_rs::hts_os::rand48_test_lock();
            let em = errmod_init(0.0);
            assert!(!em.is_null());

            let mut bases = (0..260)
                .map(|i| (((4 + (i % 60)) as u16) << 5) | ((i % 4) as u16))
                .collect::<Vec<_>>();
            let mut q = [0.0f32; 16];

            crate::htslib_rs::os_rand::hts_srand48(1);
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

            let mut seed = [0x330e, 0x0001, 0x0000];
            for _ in 1..260 {
                seed = rand48_next(seed);
            }
            seed = rand48_next(seed);
            assert_eq!(crate::htslib_rs::os_rand::hts_lrand48(), rand48_lrand(seed));

            errmod_destroy(em);
        }
    }

    #[test]
    fn cal_coef_sets_dependency_and_heterozygous_boundary_tables() {
        unsafe {
            let mut em = errmod_t {
                depcorr: 0.0,
                fk: Vec::new(),
                beta: Vec::new(),
                lhet: Vec::new(),
            };
            assert_eq!(cal_coef(&mut em, 0.25, 0.03), 0);

            assert_eq!(em.depcorr, 0.25);
            assert_eq!(em.fk[0], 1.0);
            assert!((em.fk[1] - 0.7575).abs() < 1e-12);
            assert!((em.fk[2] - 0.575625).abs() < 1e-12);
            assert_eq!(em.lhet[0], 0.0);
            assert_eq!(em.lhet[1 << 8], -std::f64::consts::LN_2);
            assert_eq!(em.lhet[(1 << 8) | 1], -std::f64::consts::LN_2);
            assert_eq!(em.beta[(4 << 16) | (1 << 8) | 1], f64::INFINITY);
        }
    }
}
