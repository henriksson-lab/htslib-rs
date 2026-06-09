pub use crate::htslib_rs::hts_os::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_long;

    fn reference_next(seed: [u16; 3]) -> [u16; 3] {
        let state = seed[0] as u64 | ((seed[1] as u64) << 16) | ((seed[2] as u64) << 32);
        let next = state.wrapping_mul(0x5deece66d).wrapping_add(0x0b) & ((1u64 << 48) - 1);
        [next as u16, (next >> 16) as u16, (next >> 32) as u16]
    }

    fn reference_erand(seed: [u16; 3]) -> f64 {
        (seed[0] as f64) * 2f64.powi(-48)
            + (seed[1] as f64) * 2f64.powi(-32)
            + (seed[2] as f64) * 2f64.powi(-16)
    }

    #[test]
    fn os_rand_reexports_explicit_seed_rand48_entrypoints() {
        let mut seed = [0x330e, 0x5678, 0x1234];
        let value = hts_erand48(&mut seed);

        assert_eq!(seed, [0x5101, 0x03f4, 0xb854]);
        assert_eq!(value.to_bits(), reference_erand(seed).to_bits());
    }

    #[test]
    fn os_rand_reexports_slice_seed_rand48_entrypoint() {
        let mut seed = [0x330e, 0x5678, 0x1234];
        let value = hts_erand48_slice(&mut seed).expect("three-word rand48 seed");

        assert_eq!(seed, [0x5101, 0x03f4, 0xb854]);
        assert_eq!(value.to_bits(), reference_erand(seed).to_bits());

        let mut short_seed = [0x330e, 0x5678];
        assert!(hts_erand48_slice(&mut short_seed).is_none());
    }

    #[test]
    fn os_rand_reexports_original_hts_os_aliases() {
        let mut seed = [0xffff, 0xffff, 0xffff];
        let value = hts_os_c_45_hts_erand48(&mut seed);

        assert_eq!(seed, [0x199e, 0x2113, 0xfffa]);
        assert_eq!(value.to_bits(), reference_erand(seed).to_bits());
    }

    #[test]
    fn os_rand_erand48_uses_caller_seed_without_advancing_global_state() {
        let _guard = crate::htslib_rs::hts_os::rand48_test_lock();
        hts_srand48(0x0000_0001);
        let expected_global_seed = reference_next([0x330e, 0x0001, 0x0000]);
        let expected_global_value = reference_erand(expected_global_seed);

        let mut explicit_seed = [0x330e, 0x5678, 0x1234];
        let expected_explicit_seed = reference_next(explicit_seed);
        let explicit_value = hts_erand48(&mut explicit_seed);
        assert_eq!(explicit_seed, expected_explicit_seed);
        assert_eq!(
            explicit_value.to_bits(),
            reference_erand(expected_explicit_seed).to_bits()
        );

        assert_eq!(hts_drand48().to_bits(), expected_global_value.to_bits());
    }

    #[test]
    fn os_rand_drand48_and_lrand48_advance_same_global_sequence() {
        let _guard = crate::htslib_rs::hts_os::rand48_test_lock();
        hts_srand48(0);
        let first_seed = reference_next([0x330e, 0x0000, 0x0000]);
        let second_seed = reference_next(first_seed);
        let second_lrand = ((second_seed[2] as c_long) << 15) + ((second_seed[1] as c_long) >> 1);

        assert_eq!(
            hts_drand48().to_bits(),
            reference_erand(first_seed).to_bits()
        );
        assert_eq!(hts_lrand48(), second_lrand);
    }

    #[test]
    fn os_rand_srand48_reinitializes_signed_seed_words() {
        let _guard = crate::htslib_rs::hts_os::rand48_test_lock();
        let first_seed = reference_next([0x330e, 0xffff, 0xffff]);
        let first_lrand = ((first_seed[2] as c_long) << 15) + ((first_seed[1] as c_long) >> 1);

        hts_os_c_35_hts_srand48(-1);
        assert_eq!(hts_os_c_51_hts_lrand48(), first_lrand);
        assert_ne!(hts_os_c_51_hts_lrand48(), first_lrand);

        hts_srand48(-1);
        assert_eq!(hts_lrand48(), first_lrand);
    }

    #[test]
    fn os_rand_srand48_uses_low_32_seed_bits() {
        let _guard = crate::htslib_rs::hts_os::rand48_test_lock();
        hts_srand48(1);
        let low_seed_lrand = hts_lrand48();

        hts_srand48(0x1_0000_0001 as c_long);
        assert_eq!(hts_lrand48(), low_seed_lrand);
    }

    #[test]
    fn os_rand_reexported_aliases_share_global_rand48_lifecycle() {
        let _guard = crate::htslib_rs::hts_os::rand48_test_lock();
        hts_os_c_35_hts_srand48(1);
        assert_eq!(hts_lrand48(), 89400484);

        hts_srand48(1);
        assert_eq!(hts_os_c_51_hts_lrand48(), 89400484);

        hts_os_c_35_hts_srand48(1);
        assert_eq!(
            hts_drand48().to_bits(),
            reference_erand(reference_next([0x330e, 0x0001, 0x0000])).to_bits()
        );

        hts_srand48(1);
        assert_eq!(
            hts_os_c_48_hts_drand48().to_bits(),
            reference_erand(reference_next([0x330e, 0x0001, 0x0000])).to_bits()
        );
    }
}
