use std::ffi::c_long;

const RAND48_SEED_0: u16 = 0x330e;
const RAND48_SEED_1: u16 = 0xabcd;
const RAND48_SEED_2: u16 = 0x1234;
const RAND48_MULT_0: u16 = 0xe66d;
const RAND48_MULT_1: u16 = 0xdeec;
const RAND48_MULT_2: u16 = 0x0005;
const RAND48_ADD: u16 = 0x000b;

static mut RAND48_SEED: [u16; 3] = [RAND48_SEED_0, RAND48_SEED_1, RAND48_SEED_2];
static mut RAND48_MULT: [u16; 3] = [RAND48_MULT_0, RAND48_MULT_1, RAND48_MULT_2];
static mut RAND48_ADD_STATE: u16 = RAND48_ADD;

pub unsafe fn _dorand48(xseed: *mut u16) {
    let mut accu = RAND48_MULT[0] as u64 * *xseed.add(0) as u64 + RAND48_ADD_STATE as u64;
    let temp0 = accu as u16;
    accu >>= std::mem::size_of::<u16>() * 8;
    accu +=
        RAND48_MULT[0] as u64 * *xseed.add(1) as u64 + RAND48_MULT[1] as u64 * *xseed.add(0) as u64;
    let temp1 = accu as u16;
    accu >>= std::mem::size_of::<u16>() * 8;
    accu += RAND48_MULT[0] as u64 * *xseed.add(2) as u64
        + RAND48_MULT[1] as u64 * *xseed.add(1) as u64
        + RAND48_MULT[2] as u64 * *xseed.add(0) as u64;
    *xseed.add(0) = temp0;
    *xseed.add(1) = temp1;
    *xseed.add(2) = accu as u16;
}

pub unsafe fn hts_srand48(seed: c_long) {
    RAND48_SEED[0] = RAND48_SEED_0;
    RAND48_SEED[1] = seed as u16;
    RAND48_SEED[2] = (seed >> 16) as u16;
    RAND48_MULT[0] = RAND48_MULT_0;
    RAND48_MULT[1] = RAND48_MULT_1;
    RAND48_MULT[2] = RAND48_MULT_2;
    RAND48_ADD_STATE = RAND48_ADD;
}

pub unsafe fn hts_erand48(xseed: *mut u16) -> f64 {
    _dorand48(xseed);
    (*xseed.add(0) as f64) * 2f64.powi(-48)
        + (*xseed.add(1) as f64) * 2f64.powi(-32)
        + (*xseed.add(2) as f64) * 2f64.powi(-16)
}

pub unsafe fn hts_drand48() -> f64 {
    let seed = std::ptr::addr_of_mut!(RAND48_SEED).cast::<u16>();
    hts_erand48(seed)
}

pub unsafe fn hts_lrand48() -> c_long {
    let seed = std::ptr::addr_of_mut!(RAND48_SEED).cast::<u16>();
    _dorand48(seed);
    ((*seed.add(2) as c_long) << 15) + ((*seed.add(1) as c_long) >> 1)
}

// original: hts_srand48 (htslib/hts_os.c:35)
pub unsafe fn hts_os_c_35_hts_srand48(seed: c_long) {
    hts_srand48(seed);
}

// original: hts_erand48 (htslib/hts_os.c:45)
pub unsafe fn hts_os_c_45_hts_erand48(xseed: *mut u16) -> f64 {
    hts_erand48(xseed)
}

// original: hts_drand48 (htslib/hts_os.c:48)
pub unsafe fn hts_os_c_48_hts_drand48() -> f64 {
    hts_drand48()
}

// original: hts_lrand48 (htslib/hts_os.c:51)
pub unsafe fn hts_os_c_51_hts_lrand48() -> c_long {
    hts_lrand48()
}

#[cfg(test)]
pub(crate) fn rand48_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn rand48_sequence_matches_known_freebsd_algorithm_values() {
        unsafe {
            let _guard = rand48_test_lock();
            let mut seed = [RAND48_SEED_0, RAND48_SEED_1, RAND48_SEED_2];
            _dorand48(seed.as_mut_ptr());
            assert_eq!(seed, [0x5101, 0xb725, 0x657e]);

            let mut custom = [0x330e, 0x0000, 0x0000];
            let x = hts_erand48(custom.as_mut_ptr());
            assert_eq!(custom, [0x5101, 0x62dc, 0x2bbb]);
            assert_eq!(x.to_bits(), reference_erand(custom).to_bits());

            hts_srand48(1);
            assert_eq!(hts_lrand48(), 89400484);
            hts_srand48(1);
            assert_eq!(
                hts_drand48().to_bits(),
                reference_erand(reference_next([RAND48_SEED_0, 0x0001, 0x0000])).to_bits()
            );
        }
    }

    #[test]
    fn srand48_initializes_seed_words_and_multiplier_state() {
        unsafe {
            let _guard = rand48_test_lock();
            hts_srand48(0x1234_5678);

            let seed = std::ptr::addr_of!(RAND48_SEED).cast::<u16>();
            assert_eq!(*seed.add(0), RAND48_SEED_0);
            assert_eq!(*seed.add(1), 0x5678);
            assert_eq!(*seed.add(2), 0x1234);

            let mult = std::ptr::addr_of!(RAND48_MULT).cast::<u16>();
            assert_eq!(*mult.add(0), RAND48_MULT_0);
            assert_eq!(*mult.add(1), RAND48_MULT_1);
            assert_eq!(*mult.add(2), RAND48_MULT_2);
            assert_eq!(std::ptr::addr_of!(RAND48_ADD_STATE).read(), RAND48_ADD);
        }
    }

    #[test]
    fn erand48_updates_all_seed_words_at_upper_boundary() {
        unsafe {
            let _guard = rand48_test_lock();
            let mut seed = [0xffff, 0xffff, 0xffff];
            let value = hts_os_c_45_hts_erand48(seed.as_mut_ptr());

            assert_eq!(seed, [0x199e, 0x2113, 0xfffa]);
            assert_eq!(value.to_bits(), reference_erand(seed).to_bits());
        }
    }

    #[test]
    fn rand48_entrypoint_aliases_share_deterministic_state_progression() {
        unsafe {
            let _guard = rand48_test_lock();
            hts_os_c_35_hts_srand48(0x1234_5678);
            let first_seed = reference_next([RAND48_SEED_0, 0x5678, 0x1234]);
            let first_value = reference_erand(first_seed);
            assert_eq!(hts_os_c_51_hts_lrand48(), 0x5c2a01fa);

            hts_os_c_35_hts_srand48(0x1234_5678);
            assert_eq!(hts_os_c_48_hts_drand48().to_bits(), first_value.to_bits());

            let second_seed = reference_next(first_seed);
            let second_value = reference_erand(second_seed);
            assert_eq!(hts_os_c_48_hts_drand48().to_bits(), second_value.to_bits());
        }
    }

    #[test]
    fn erand48_uses_caller_seed_without_advancing_global_state() {
        unsafe {
            let _guard = rand48_test_lock();
            hts_srand48(0x0000_0001);
            let expected_global_seed = reference_next([RAND48_SEED_0, 0x0001, 0x0000]);
            let expected_global_value = reference_erand(expected_global_seed);

            let mut explicit_seed = [0x330e, 0x5678, 0x1234];
            let expected_explicit_seed = reference_next(explicit_seed);
            let explicit_value = hts_erand48(explicit_seed.as_mut_ptr());
            assert_eq!(explicit_seed, expected_explicit_seed);
            assert_eq!(
                explicit_value.to_bits(),
                reference_erand(expected_explicit_seed).to_bits()
            );

            assert_eq!(hts_drand48().to_bits(), expected_global_value.to_bits());
        }
    }

    #[test]
    fn srand48_reinitializes_after_alias_state_advancement() {
        unsafe {
            let _guard = rand48_test_lock();
            let first_seed = reference_next([RAND48_SEED_0, 0xffff, 0xffff]);
            let first_lrand = ((first_seed[2] as c_long) << 15) + ((first_seed[1] as c_long) >> 1);

            hts_os_c_35_hts_srand48(-1);
            assert_eq!(hts_os_c_51_hts_lrand48(), first_lrand);
            assert_ne!(hts_os_c_51_hts_lrand48(), first_lrand);

            hts_srand48(-1);
            let seed = std::ptr::addr_of!(RAND48_SEED).cast::<u16>();
            assert_eq!(*seed.add(0), RAND48_SEED_0);
            assert_eq!(*seed.add(1), 0xffff);
            assert_eq!(*seed.add(2), 0xffff);
            assert_eq!(hts_lrand48(), first_lrand);
        }
    }
}
