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

pub unsafe fn hts_os_c_35_hts_srand48(seed: c_long) {
    hts_srand48(seed);
}

pub unsafe fn hts_os_c_45_hts_erand48(xseed: *mut u16) -> f64 {
    hts_erand48(xseed)
}

pub unsafe fn hts_os_c_48_hts_drand48() -> f64 {
    hts_drand48()
}

pub unsafe fn hts_os_c_51_hts_lrand48() -> c_long {
    hts_lrand48()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rand48_sequence_matches_known_freebsd_algorithm_values() {
        unsafe {
            let mut seed = [RAND48_SEED_0, RAND48_SEED_1, RAND48_SEED_2];
            _dorand48(seed.as_mut_ptr());
            assert_eq!(seed, [0x5101, 0xb725, 0x657e]);

            let mut custom = [0x330e, 0x0000, 0x0000];
            let x = hts_erand48(custom.as_mut_ptr());
            assert_eq!(custom, [0x5101, 0x62dc, 0x2bbb]);
            assert!((x - 0.17082803610628972).abs() < 1e-18);

            hts_srand48(1);
            assert_eq!(hts_lrand48(), 89400484);
            hts_srand48(1);
            assert!((hts_drand48() - 0.041630344771878214).abs() < 1e-18);
        }
    }
}
