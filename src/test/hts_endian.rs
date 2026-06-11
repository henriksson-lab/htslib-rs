use crate::htslib_rs::hts::{
    double_to_le, float_to_le, i16_to_le, i32_to_le, i64_to_le, le_to_double, le_to_float,
    le_to_i16, le_to_i32, le_to_i64, le_to_u16, le_to_u32, le_to_u64, u16_to_le, u32_to_le,
    u64_to_le,
};

// original: Test16 (htslib/test/hts_endian.c:34)
#[derive(Clone, Copy)]
struct Test16 {
    u8: [u8; 2],
    u8_unaligned: [u8; 3],
    i16_: i16,
    u16_: u16,
}

// original: Test32 (htslib/test/hts_endian.c:41)
#[derive(Clone, Copy)]
struct Test32 {
    u8: [u8; 4],
    u8_unaligned: [u8; 5],
    i32_: i32,
    u32_: u32,
}

// original: Test64 (htslib/test/hts_endian.c:48)
#[derive(Clone, Copy)]
struct Test64 {
    u8: [u8; 8],
    u8_unaligned: [u8; 9],
    i64_: i64,
    u64_: u64,
}

// original: Test_float (htslib/test/hts_endian.c:55)
#[derive(Clone, Copy)]
struct TestFloat {
    u8: [u8; 4],
    u8_unaligned: [u8; 5],
    f: f32,
}

// original: Test_double (htslib/test/hts_endian.c:61)
#[derive(Clone, Copy)]
struct TestDouble {
    u8: [u8; 8],
    u8_unaligned: [u8; 9],
    d: f64,
}

// original: tests_16_bit (htslib/test/hts_endian.c:70)
const TESTS_16_BIT: [Test16; 6] = [
    Test16 {
        u8: [0x00, 0x00],
        u8_unaligned: [0x00, 0x00, 0x00],
        i16_: 0,
        u16_: 0,
    },
    Test16 {
        u8: [0x01, 0x00],
        u8_unaligned: [0x00, 0x01, 0x00],
        i16_: 1,
        u16_: 1,
    },
    Test16 {
        u8: [0x00, 0x01],
        u8_unaligned: [0x00, 0x00, 0x01],
        i16_: 256,
        u16_: 256,
    },
    Test16 {
        u8: [0xff, 0x7f],
        u8_unaligned: [0x00, 0xff, 0x7f],
        i16_: 32767,
        u16_: 32767,
    },
    Test16 {
        u8: [0x00, 0x80],
        u8_unaligned: [0x00, 0x00, 0x80],
        i16_: -32768,
        u16_: 32768,
    },
    Test16 {
        u8: [0xff, 0xff],
        u8_unaligned: [0x00, 0xff, 0xff],
        i16_: -1,
        u16_: 65535,
    },
];

// original: tests_32_bit (htslib/test/hts_endian.c:83)
const TESTS_32_BIT: [Test32; 7] = [
    Test32 {
        u8: [0x00, 0x00, 0x00, 0x00],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x00],
        i32_: 0,
        u32_: 0,
    },
    Test32 {
        u8: [0x01, 0x00, 0x00, 0x00],
        u8_unaligned: [0x00, 0x01, 0x00, 0x00, 0x00],
        i32_: 1,
        u32_: 1,
    },
    Test32 {
        u8: [0x00, 0x01, 0x00, 0x00],
        u8_unaligned: [0x00, 0x00, 0x01, 0x00, 0x00],
        i32_: 256,
        u32_: 256,
    },
    Test32 {
        u8: [0x00, 0x00, 0x01, 0x00],
        u8_unaligned: [0x00, 0x00, 0x00, 0x01, 0x00],
        i32_: 65536,
        u32_: 65536,
    },
    Test32 {
        u8: [0xff, 0xff, 0xff, 0x7f],
        u8_unaligned: [0x00, 0xff, 0xff, 0xff, 0x7f],
        i32_: 2147483647,
        u32_: 2147483647,
    },
    // Odd coding of signed result below avoids a compiler warning
    // as 2147483648 can't fit in a signed 32-bit number
    Test32 {
        u8: [0x00, 0x00, 0x00, 0x80],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x80],
        i32_: -2147483647 - 1,
        u32_: 2147483648,
    },
    Test32 {
        u8: [0xff, 0xff, 0xff, 0xff],
        u8_unaligned: [0x00, 0xff, 0xff, 0xff, 0xff],
        i32_: -1,
        u32_: 4294967295,
    },
];

// original: tests_64_bit (htslib/test/hts_endian.c:105)
const TESTS_64_BIT: [Test64; 8] = [
    Test64 {
        u8: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        i64_: 0,
        u64_: 0,
    },
    Test64 {
        u8: [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        u8_unaligned: [0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        i64_: 1,
        u64_: 1,
    },
    Test64 {
        u8: [0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        u8_unaligned: [0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        i64_: 256,
        u64_: 256,
    },
    Test64 {
        u8: [0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
        u8_unaligned: [0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
        i64_: 65536,
        u64_: 65536,
    },
    Test64 {
        u8: [0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00],
        i64_: 4294967296,
        u64_: 4294967296,
    },
    Test64 {
        u8: [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        u8_unaligned: [0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        i64_: 9223372036854775807,
        u64_: 9223372036854775807,
    },
    // Odd coding of signed result below avoids a compiler warning
    // as 9223372036854775808 can't fit in a signed 64-bit number
    Test64 {
        u8: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80],
        i64_: -9223372036854775807 - 1,
        u64_: 9223372036854775808,
    },
    Test64 {
        u8: [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        u8_unaligned: [0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        i64_: -1,
        u64_: 18446744073709551615,
    },
];

// original: tests_float (htslib/test/hts_endian.c:130)
const TESTS_FLOAT: [TestFloat; 7] = [
    TestFloat {
        u8: [0x00, 0x00, 0x00, 0x00],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x00],
        f: 0.0,
    },
    TestFloat {
        u8: [0x00, 0x00, 0x80, 0x3f],
        u8_unaligned: [0x00, 0x00, 0x00, 0x80, 0x3f],
        f: 1.0,
    },
    TestFloat {
        u8: [0x00, 0x00, 0x80, 0xbf],
        u8_unaligned: [0x00, 0x00, 0x00, 0x80, 0xbf],
        f: -1.0,
    },
    TestFloat {
        u8: [0x00, 0x00, 0x20, 0x41],
        u8_unaligned: [0x00, 0x00, 0x00, 0x20, 0x41],
        f: 10.0,
    },
    TestFloat {
        u8: [0xd0, 0x0f, 0x49, 0x40],
        u8_unaligned: [0x00, 0xd0, 0x0f, 0x49, 0x40],
        f: f32::from_bits(0x4049_0fd0),
    },
    TestFloat {
        u8: [0xa8, 0x0a, 0xff, 0x66],
        u8_unaligned: [0x00, 0xa8, 0x0a, 0xff, 0x66],
        f: 6.022e23,
    },
    TestFloat {
        u8: [0xcd, 0x84, 0x03, 0x13],
        u8_unaligned: [0x00, 0xcd, 0x84, 0x03, 0x13],
        f: 1.66e-27,
    },
];

// original: tests_double (htslib/test/hts_endian.c:144)
const TESTS_DOUBLE: [TestDouble; 7] = [
    TestDouble {
        u8: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        d: 0.0,
    },
    TestDouble {
        u8: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f],
        d: 1.0,
    },
    TestDouble {
        u8: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0xbf],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0xbf],
        d: -1.0,
    },
    TestDouble {
        u8: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x40],
        u8_unaligned: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x40],
        d: 10.0,
    },
    TestDouble {
        u8: [0x18, 0x2d, 0x44, 0x54, 0xfb, 0x21, 0x09, 0x40],
        u8_unaligned: [0x00, 0x18, 0x2d, 0x44, 0x54, 0xfb, 0x21, 0x09, 0x40],
        d: std::f64::consts::PI,
    },
    TestDouble {
        u8: [0x2b, 0x08, 0x0c, 0xd3, 0x85, 0xe1, 0xdf, 0x44],
        u8_unaligned: [0x00, 0x2b, 0x08, 0x0c, 0xd3, 0x85, 0xe1, 0xdf, 0x44],
        d: 6.022140858e23,
    },
    TestDouble {
        u8: [0x55, 0xfa, 0x81, 0x74, 0xf7, 0x71, 0x60, 0x3a],
        u8_unaligned: [0x00, 0x55, 0xfa, 0x81, 0x74, 0xf7, 0x71, 0x60, 0x3a],
        d: 1.66053904e-27,
    },
];

// original: to_hex (htslib/test/hts_endian.c:149)
pub fn test_hts_endian_c_149_to_hex(buf: &[u8], len: i32) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < len {
        out.extend_from_slice(format!("{:02x} ", buf[i as usize]).as_bytes());
        i += 1;
    }
    out
}

// original: t16_bit (htslib/test/hts_endian.c:159)
pub unsafe fn test_hts_endian_c_159_t16_bit(verbose: i32) -> i32 {
    let mut buf = [0u8; 9];
    let mut errors = 0;

    for test in TESTS_16_BIT.iter() {
        let mut u16_: u16;
        let mut i16_: i16;

        if verbose != 0 {
            eprintln!(
                "{} {:6} {:6}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 2)),
                test.i16_,
                test.u16_,
            );
        }

        u16_ = le_to_u16(test.u8.as_ptr());
        if u16_ != test.u16_ {
            eprintln!(
                "Failed {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 2)),
                u16_,
                test.u16_,
            );
            errors += 1;
        }

        i16_ = le_to_i16(test.u8.as_ptr());
        if i16_ != test.i16_ {
            eprintln!(
                "Failed {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 2)),
                i16_,
                test.i16_,
            );
            errors += 1;
        }

        u16_ = le_to_u16(test.u8_unaligned.as_ptr().add(1));
        if u16_ != test.u16_ {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8_unaligned[1..], 2)),
                u16_,
                test.u16_,
            );
            errors += 1;
        }

        i16_ = le_to_i16(test.u8_unaligned.as_ptr().add(1));
        if i16_ != test.i16_ {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8_unaligned[1..], 2)),
                i16_,
                test.i16_,
            );
            errors += 1;
        }

        u16_to_le(test.u16_, buf.as_mut_ptr());
        if buf[..2] != test.u8[..2] {
            eprintln!(
                "Failed {} => {}; expected {}",
                test.u16_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf, 2)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 2)),
            );
            errors += 1;
        }

        i16_to_le(test.i16_, buf.as_mut_ptr());
        if buf[..2] != test.u8[..2] {
            eprintln!(
                "Failed {} => {}; expected {}",
                test.i16_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf, 2)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 2)),
            );
            errors += 1;
        }

        u16_to_le(test.u16_, buf.as_mut_ptr().add(1));
        if buf[1..3] != test.u8[..2] {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                test.u16_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf[1..], 2)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 2)),
            );
            errors += 1;
        }

        i16_to_le(test.i16_, buf.as_mut_ptr().add(1));
        if buf[1..3] != test.u8[..2] {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                test.i16_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf[1..], 2)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 2)),
            );
            errors += 1;
        }
    }

    errors
}

// original: t32_bit (htslib/test/hts_endian.c:242)
pub unsafe fn test_hts_endian_c_242_t32_bit(verbose: i32) -> i32 {
    let mut buf = [0u8; 9];
    let mut errors = 0;

    for test in TESTS_32_BIT.iter() {
        let mut u32_: u32;
        let mut i32_: i32;

        if verbose != 0 {
            eprintln!(
                "{} {:11} {:11}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
                test.i32_,
                test.u32_,
            );
        }

        u32_ = le_to_u32(test.u8.as_ptr());
        if u32_ != test.u32_ {
            eprintln!(
                "Failed {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
                u32_,
                test.u32_,
            );
            errors += 1;
        }

        i32_ = le_to_i32(test.u8.as_ptr());
        if i32_ != test.i32_ {
            eprintln!(
                "Failed {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
                i32_,
                test.i32_,
            );
            errors += 1;
        }

        u32_ = le_to_u32(test.u8_unaligned.as_ptr().add(1));
        if u32_ != test.u32_ {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8_unaligned[1..], 4)),
                u32_,
                test.u32_,
            );
            errors += 1;
        }

        i32_ = le_to_i32(test.u8_unaligned.as_ptr().add(1));
        if i32_ != test.i32_ {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8_unaligned[1..], 4)),
                i32_,
                test.i32_,
            );
            errors += 1;
        }

        u32_to_le(test.u32_, buf.as_mut_ptr());
        if buf[..4] != test.u8[..4] {
            eprintln!(
                "Failed {} => {}; expected {}",
                test.u32_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf, 4)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
            );
            errors += 1;
        }

        i32_to_le(test.i32_, buf.as_mut_ptr());
        if buf[..4] != test.u8[..4] {
            eprintln!(
                "Failed {} => {}; expected {}",
                test.i32_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf, 4)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
            );
            errors += 1;
        }

        u32_to_le(test.u32_, buf.as_mut_ptr().add(1));
        if buf[1..5] != test.u8[..4] {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                test.u32_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf[1..], 4)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
            );
            errors += 1;
        }

        i32_to_le(test.i32_, buf.as_mut_ptr().add(1));
        if buf[1..5] != test.u8[..4] {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                test.i32_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf[1..], 4)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
            );
            errors += 1;
        }
    }

    errors
}

// original: t64_bit (htslib/test/hts_endian.c:323)
pub unsafe fn test_hts_endian_c_323_t64_bit(verbose: i32) -> i32 {
    let mut buf = [0u8; 9];
    let mut errors = 0;

    for test in TESTS_64_BIT.iter() {
        let mut u64_: u64;
        let mut i64_: i64;

        if verbose != 0 {
            eprintln!(
                "{} {:20} {:20}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
                test.i64_,
                test.u64_,
            );
        }

        u64_ = le_to_u64(test.u8.as_ptr());
        if u64_ != test.u64_ {
            eprintln!(
                "Failed {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
                u64_,
                test.u64_,
            );
            errors += 1;
        }

        i64_ = le_to_i64(test.u8.as_ptr());
        if i64_ != test.i64_ {
            eprintln!(
                "Failed {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
                i64_,
                test.i64_,
            );
            errors += 1;
        }

        u64_ = le_to_u64(test.u8_unaligned.as_ptr().add(1));
        if u64_ != test.u64_ {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8_unaligned[1..], 8)),
                u64_,
                test.u64_,
            );
            errors += 1;
        }

        i64_ = le_to_i64(test.u8_unaligned.as_ptr().add(1));
        if i64_ != test.i64_ {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8_unaligned[1..], 8)),
                i64_,
                test.i64_,
            );
            errors += 1;
        }

        u64_to_le(test.u64_, buf.as_mut_ptr());
        if buf[..8] != test.u8[..8] {
            eprintln!(
                "Failed {} => {}; expected {}",
                test.u64_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf, 8)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
            );
            errors += 1;
        }

        i64_to_le(test.i64_, buf.as_mut_ptr());
        if buf[..8] != test.u8[..8] {
            eprintln!(
                "Failed {} => {}; expected {}",
                test.i64_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf, 8)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
            );
            errors += 1;
        }

        u64_to_le(test.u64_, buf.as_mut_ptr().add(1));
        if buf[1..9] != test.u8[..8] {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                test.u64_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf[1..], 8)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
            );
            errors += 1;
        }

        i64_to_le(test.i64_, buf.as_mut_ptr().add(1));
        if buf[1..9] != test.u8[..8] {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                test.i64_,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf[1..], 8)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
            );
            errors += 1;
        }
    }

    errors
}

// original: t_float (htslib/test/hts_endian.c:406)
pub unsafe fn test_hts_endian_c_406_t_float(verbose: i32) -> i32 {
    let mut buf = [0u8; 9];
    let mut errors = 0;

    for test in TESTS_FLOAT.iter() {
        let mut f: f32;

        if verbose != 0 {
            eprintln!(
                "{} {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
                test.f as f64,
            );
        }

        f = le_to_float(test.u8.as_ptr());
        if f != test.f {
            eprintln!(
                "Failed {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
                f as f64,
                test.f as f64,
            );
            errors += 1;
        }

        f = le_to_float(test.u8_unaligned.as_ptr().add(1));
        if f != test.f {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8_unaligned[1..], 4)),
                f as f64,
                test.f as f64,
            );
            errors += 1;
        }

        float_to_le(test.f, buf.as_mut_ptr());
        if test.u8[..4] != buf[..4] {
            eprintln!(
                "Failed {} => {}; expected {}",
                test.f as f64,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf, 4)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
            );
        }

        float_to_le(test.f, buf.as_mut_ptr().add(1));
        if test.u8[..4] != buf[1..5] {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                test.f as f64,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf[1..], 4)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 4)),
            );
        }
    }

    errors
}

// original: t_double (htslib/test/hts_endian.c:451)
pub unsafe fn test_hts_endian_c_451_t_double(verbose: i32) -> i32 {
    let mut buf = [0u8; 9];
    let mut errors = 0;

    for test in TESTS_DOUBLE.iter() {
        let mut f: f64;

        if verbose != 0 {
            eprintln!(
                "{} {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
                test.d,
            );
        }

        f = le_to_double(test.u8.as_ptr());
        if f != test.d {
            eprintln!(
                "Failed {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
                f,
                test.d,
            );
            errors += 1;
        }

        f = le_to_double(test.u8_unaligned.as_ptr().add(1));
        if f != test.d {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8_unaligned[1..], 8)),
                f,
                test.d,
            );
            errors += 1;
        }

        double_to_le(test.d, buf.as_mut_ptr());
        if test.u8[..8] != buf[..8] {
            eprintln!(
                "Failed {} => {}; expected {}",
                test.d,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf, 8)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
            );
        }

        double_to_le(test.d, buf.as_mut_ptr().add(1));
        if test.u8[..8] != buf[1..9] {
            eprintln!(
                "Failed unaligned {} => {}; expected {}",
                test.d,
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&buf[1..], 8)),
                String::from_utf8_lossy(&test_hts_endian_c_149_to_hex(&test.u8, 8)),
            );
        }
    }

    errors
}

// original: main (htslib/test/hts_endian.c:496)
pub unsafe fn test_hts_endian_c_496_main(argc: i32, argv: &[&[u8]]) -> i32 {
    let mut verbose = 0;
    let mut errors = 0;

    if argc > 1 && argv[1] == b"-v" {
        verbose = 1;
    }

    errors += test_hts_endian_c_159_t16_bit(verbose);
    errors += test_hts_endian_c_242_t32_bit(verbose);
    errors += test_hts_endian_c_323_t64_bit(verbose);
    errors += test_hts_endian_c_406_t_float(verbose);
    errors += test_hts_endian_c_451_t_double(verbose);

    if errors != 0 {
        eprintln!("{} errors", errors);
        return libc::EXIT_FAILURE;
    }

    libc::EXIT_SUCCESS
}
