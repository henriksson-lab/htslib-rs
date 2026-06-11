use crate::htslib_rs::{kfunc::lbinom, os_rand::hts_drand48};

const ERRMOD_TABLE_SIZE: usize = 256;
const ERRMOD_QUAL_SIZE: usize = 64;

pub struct errmod_t {
    pub depcorr: f64,
    pub fk: Vec<f64>,
    pub beta: Vec<f64>,
    pub lhet: Vec<f64>,
}

struct call_aux_t {
    fsum: [f64; 16],
    bsum: [f64; 16],
    c: [u32; 16],
}

pub fn logbinomial_table() -> Vec<f64> {
    let mut logbinom = vec![0.0; ERRMOD_TABLE_SIZE * ERRMOD_TABLE_SIZE];
    let mut n = 1usize;
    while n < ERRMOD_TABLE_SIZE {
        let mut k = 1usize;
        while k <= n {
            logbinom[(n << 8) | k] = lbinom(n as i32, k as i32);
            k += 1;
        }
        n += 1;
    }
    logbinom
}

pub fn cal_coef(em: &mut errmod_t, depcorr: f64, eta: f64) -> i32 {
    let mut fk = vec![0.0; ERRMOD_TABLE_SIZE];
    fk[0] = 1.0;
    let mut n = 1usize;
    while n < ERRMOD_TABLE_SIZE {
        fk[n] = (1.0 - depcorr).powi(n as i32) * (1.0 - eta) + eta;
        n += 1;
    }

    let mut beta = vec![0.0; ERRMOD_TABLE_SIZE * ERRMOD_TABLE_SIZE * ERRMOD_QUAL_SIZE];

    let lc = logbinomial_table();

    let mut q = 1usize;
    while q < ERRMOD_QUAL_SIZE {
        let e = 10.0_f64.powf(-(q as f64) / 10.0);
        let le = e.ln();
        let le1 = (1.0 - e).ln();
        n = 1;
        while n < ERRMOD_TABLE_SIZE {
            let beta_row = (q << 16) | (n << 8);
            let mut sum1 = lc[(n << 8) | n] + n as f64 * le;
            beta[beta_row + n] = f64::INFINITY;
            let mut k = n - 1;
            loop {
                let sum = sum1
                    + (lc[(n << 8) | k] + k as f64 * le + (n - k) as f64 * le1 - sum1)
                        .exp()
                        .ln_1p();
                beta[beta_row + k] = -10.0 / std::f64::consts::LN_10 * (sum1 - sum);
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

    let mut lhet = vec![0.0; ERRMOD_TABLE_SIZE * ERRMOD_TABLE_SIZE];
    n = 0;
    while n < ERRMOD_TABLE_SIZE {
        let mut k = 0usize;
        while k < ERRMOD_TABLE_SIZE {
            lhet[(n << 8) | k] = lc[(n << 8) | k] - std::f64::consts::LN_2 * n as f64;
            k += 1;
        }
        n += 1;
    }
    em.depcorr = depcorr;
    em.fk = fk;
    em.beta = beta;
    em.lhet = lhet;
    0
}

pub fn errmod_new(depcorr: f64) -> Option<Box<errmod_t>> {
    let mut em = Box::new(errmod_t {
        depcorr,
        fk: Vec::new(),
        beta: Vec::new(),
        lhet: Vec::new(),
    });
    if cal_coef(&mut em, depcorr, 0.03) != 0 {
        return None;
    }
    Some(em)
}

pub fn errmod_init(depcorr: f64) -> Option<Box<errmod_t>> {
    errmod_new(depcorr)
}

pub fn errmod_destroy(em: Option<Box<errmod_t>>) {
    drop(em);
}

pub fn errmod_cal_ref(em: &errmod_t, bases: &mut [u16], m: usize, q: &mut [f32]) -> i32 {
    if q.len() < m.saturating_mul(m) {
        return -1;
    }
    q[..m * m].fill(0.0);
    if bases.is_empty() {
        return 0;
    }

    let n = bases.len().min(ERRMOD_TABLE_SIZE - 1);
    if bases.len() > ERRMOD_TABLE_SIZE - 1 {
        let mut i = bases.len() - 1;
        while i > 0 {
            let j = (hts_drand48() * (i + 1) as f64) as i32;
            bases.swap(i, j as usize);
            i -= 1;
        }
    }

    bases[..n].sort_unstable();

    let mut w = [0i32; 32];
    let mut aux = call_aux_t {
        fsum: [0.0; 16],
        bsum: [0.0; 16],
        c: [0; 16],
    };

    let mut j = n - 1;
    loop {
        let b = bases[j];
        let mut qual = if (b >> 5) < 4 { 4 } else { (b >> 5) as usize };
        if qual >= ERRMOD_QUAL_SIZE {
            qual = ERRMOD_QUAL_SIZE - 1;
        }
        let basestrand = (b & 0x1f) as usize;
        let base = (b & 0xf) as usize;
        aux.fsum[base] += em.fk[w[basestrand] as usize];
        aux.bsum[base] +=
            em.fk[w[basestrand] as usize] * em.beta[(qual << 16) | (n << 8) | aux.c[base] as usize];
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
                tmp1 = (tmp1 as f64 + aux.bsum[k]) as f32;
                tmp2 += aux.c[k] as i32;
            }
            k += 1;
        }
        if tmp2 != 0 {
            q[j * m + j] = tmp1;
        }

        k = j + 1;
        while k < m {
            let cjk = aux.c[j] + aux.c[k];
            let mut i = 0;
            tmp1 = 0.0;
            tmp2 = 0;
            while i < m {
                if i != j && i != k {
                    tmp1 = (tmp1 as f64 + aux.bsum[i]) as f32;
                    tmp2 += aux.c[i] as i32;
                }
                i += 1;
            }
            let val = -4.343f64 * em.lhet[((cjk << 8) | aux.c[k]) as usize];
            if tmp2 != 0 {
                let out = (val + tmp1 as f64) as f32;
                q[j * m + k] = out;
                q[k * m + j] = out;
            } else {
                let out = val as f32;
                q[j * m + k] = out;
                q[k * m + j] = out;
            }
            k += 1;
        }

        k = 0;
        while k < m {
            let v = &mut q[j * m + k];
            if *v < 0.0 {
                *v = 0.0;
            }
            k += 1;
        }
        j += 1;
    }
    0
}

pub fn errmod_cal(em: &errmod_t, bases: &mut [u16], m: usize, q: &mut [f32]) -> i32 {
    errmod_cal_ref(em, bases, m, q)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rand48_next(seed: [u16; 3]) -> [u16; 3] {
        let state = seed[0] as u64 | ((seed[1] as u64) << 16) | ((seed[2] as u64) << 32);
        let next = state.wrapping_mul(0x5deece66d).wrapping_add(0x0b) & ((1u64 << 48) - 1);
        [next as u16, (next >> 16) as u16, (next >> 32) as u16]
    }

    fn rand48_lrand(seed: [u16; 3]) -> i64 {
        ((seed[2] as i64) << 15) + ((seed[1] as i64) >> 1)
    }

    #[test]
    fn errmod_init_calculates_likelihoods_and_destroy_frees() {
        let em = errmod_init(0.1).expect("errmod init");
        assert_eq!(em.fk.len(), 256);
        assert_eq!(em.beta.len(), 256 * 256 * 64);
        assert_eq!(em.lhet.len(), 256 * 256);
        assert_eq!(em.depcorr, 0.1);

        let mut bases = [30u16 << 5, 25u16 << 5, (20u16 << 5) | 1, (35u16 << 5) | 17];
        let mut q = [0.0f32; 16];
        assert_eq!(errmod_cal(&em, &mut bases, 4, &mut q), 0);
        assert_eq!(
            bases,
            [(20u16 << 5) | 1, 25u16 << 5, 30u16 << 5, (35u16 << 5) | 17,]
        );
        assert!(q.iter().all(|v| *v >= 0.0));
        assert!(q.iter().any(|v| *v > 0.0));
        errmod_destroy(Some(em));
        errmod_destroy(None);
    }

    #[test]
    fn logbinomial_table_populates_internal_triangle_only() {
        let table = logbinomial_table();
        assert_eq!(table.len(), 256 * 256);
        assert_eq!(table[1 << 8], 0.0);
        assert_eq!(table[(1 << 8) | 1], 0.0);
        assert_eq!(table[(7 << 8) | 7], 0.0);
        assert!((table[(7 << 8) | 3] - lbinom(7, 3)).abs() < 1e-12);
        assert!((table[(7 << 8) | 4] - lbinom(7, 4)).abs() < 1e-12);
    }

    #[test]
    fn errmod_cal_zero_depth_only_clears_output_matrix() {
        let em = errmod_init(0.0).expect("errmod init");

        let mut bases = [(30u16 << 5) | 3];
        let original_bases = bases;
        let mut q = [7.0f32; 16];
        assert_eq!(errmod_cal(&em, &mut bases[..0], 4, &mut q), 0);
        assert_eq!(bases, original_bases);
        assert_eq!(q, [0.0; 16]);
    }

    #[test]
    fn errmod_cal_single_base_leaves_homozygous_call_at_zero() {
        let em = errmod_init(0.0).expect("errmod init");

        let mut bases = [(30u16 << 5) | 2];
        let mut q = [9.0f32; 16];
        assert_eq!(errmod_cal(&em, &mut bases, 4, &mut q), 0);
        assert_eq!(bases, [(30u16 << 5) | 2]);
        assert_eq!(q[2 * 4 + 2], 0.0);
        assert!(q[0] > 0.0);
        assert!(q[5] > 0.0);
        assert!(q[3 * 4 + 3] > 0.0);
    }

    #[test]
    fn errmod_cal_clamps_quality_scores_to_supported_boundaries() {
        let em = errmod_init(0.0).expect("errmod init");

        let mut low_clamped = [0, (20u16 << 5) | 1, (30u16 << 5) | 2];
        let mut low_boundary = [4u16 << 5, (20u16 << 5) | 1, (30u16 << 5) | 2];
        let mut q_low_clamped = [0.0f32; 16];
        let mut q_low_boundary = [0.0f32; 16];
        assert_eq!(errmod_cal(&em, &mut low_clamped, 4, &mut q_low_clamped), 0);
        assert_eq!(
            errmod_cal(&em, &mut low_boundary, 4, &mut q_low_boundary),
            0
        );
        assert_eq!(q_low_clamped, q_low_boundary);

        let mut high_clamped = [63u16 << 5, (80u16 << 5) | 1, (30u16 << 5) | 2];
        let mut high_boundary = [63u16 << 5, (63u16 << 5) | 1, (30u16 << 5) | 2];
        let mut q_high_clamped = [0.0f32; 16];
        let mut q_high_boundary = [0.0f32; 16];
        assert_eq!(
            errmod_cal(&em, &mut high_clamped, 4, &mut q_high_clamped),
            0
        );
        assert_eq!(
            errmod_cal(&em, &mut high_boundary, 4, &mut q_high_boundary),
            0
        );
        assert_eq!(q_high_clamped, q_high_boundary);
    }

    #[test]
    fn errmod_cal_strand_bit_does_not_change_base_index() {
        let em = errmod_init(0.0).expect("errmod init");

        let mut bases = [(30u16 << 5) | 17, (30u16 << 5) | 1];
        let mut q = [0.0f32; 16];
        assert_eq!(errmod_cal(&em, &mut bases, 4, &mut q), 0);

        assert_eq!(bases, [(30u16 << 5) | 1, (30u16 << 5) | 17]);
        assert_eq!(q[5], 0.0);
        assert!(q[0] > 0.0);
        assert!(q[2 * 4 + 2] > 0.0);
        assert!(q[3 * 4 + 3] > 0.0);
    }

    #[test]
    fn errmod_cal_only_clears_requested_square_matrix() {
        let em = errmod_init(0.0).expect("errmod init");

        let mut bases = [30u16 << 5, (30u16 << 5) | 1];
        let mut q = [7.0f32; 9];
        assert_eq!(errmod_cal(&em, &mut bases, 2, &mut q[..4]), 0);

        assert_eq!(q[4..], [7.0; 5]);
        assert_eq!(q[1], q[2]);
        assert!(q[..4].iter().all(|v| *v >= 0.0));
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
                let rust_em = errmod_init(depcorr).expect("errmod init");
                let c_em = hts_sys::errmod_init(depcorr);
                assert!(!c_em.is_null());

                for input in cases {
                    let mut rust_bases = input.to_vec();
                    let mut c_bases = input.to_vec();
                    let mut rust_q = [0.0f32; 16];
                    let mut c_q = [0.0f32; 16];

                    assert_eq!(
                        errmod_cal(&rust_em, rust_bases.as_mut_slice(), 4, &mut rust_q),
                        0
                    );
                    assert_eq!(
                        hts_sys::errmod_cal(
                            c_em,
                            c_bases.len() as i32,
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

                hts_sys::errmod_destroy(c_em);
            }
        }
    }

    #[test]
    fn errmod_cal_downsampling_uses_htslib_rand48_state() {
        let _guard = crate::htslib_rs::hts_os::rand48_test_lock();
        let em = errmod_init(0.0).expect("errmod init");

        let mut bases = (0..260)
            .map(|i| (((4 + (i % 60)) as u16) << 5) | ((i % 4) as u16))
            .collect::<Vec<_>>();
        let mut q = [0.0f32; 16];

        crate::htslib_rs::os_rand::hts_srand48(1);
        assert_eq!(errmod_cal(&em, bases.as_mut_slice(), 4, &mut q), 0);

        let mut seed = [0x330e, 0x0001, 0x0000];
        for _ in 1..260 {
            seed = rand48_next(seed);
        }
        seed = rand48_next(seed);
        assert_eq!(crate::htslib_rs::os_rand::hts_lrand48(), rand48_lrand(seed));
    }

    #[test]
    fn cal_coef_sets_dependency_and_heterozygous_boundary_tables() {
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
