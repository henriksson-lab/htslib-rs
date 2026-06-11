use crate::htslib_rs::kfunc;

static mut TEST_KFUNC_NFAILED: i32 = 0;

// original: differ (htslib/test/test_kfunc.c:33)
pub unsafe fn test_test_kfunc_c_33_differ(obs: f64, expected: f64) -> i32 {
    ((obs - expected).abs() > 1e-8) as i32
}

// original: fail (htslib/test/test_kfunc.c:40)
pub unsafe fn test_test_kfunc_c_40_fail(
    test: &[u8],
    obs: f64,
    expected: f64,
    n11: i32,
    n12: i32,
    n21: i32,
    n22: i32,
) {
    eprintln!(
        "[{} {} | {} {}] {}: {} (expected {})",
        n11,
        n12,
        n21,
        n22,
        String::from_utf8_lossy(test),
        obs,
        expected,
    );
    TEST_KFUNC_NFAILED += 1;
}

// original: test_fisher (htslib/test/test_kfunc.c:48)
#[allow(clippy::too_many_arguments)]
pub unsafe fn test_test_kfunc_c_48_test_fisher(
    n11: i32,
    n12: i32,
    n21: i32,
    n22: i32,
    eleft: f64,
    eright: f64,
    etwo: f64,
    eprob: f64,
) {
    let mut left = 0.0;
    let mut right = 0.0;
    let mut two = 0.0;
    let prob = kfunc::kt_fisher_exact(n11, n12, n21, n22, &mut left, &mut right, &mut two);
    if test_test_kfunc_c_33_differ(left, eleft) != 0 {
        test_test_kfunc_c_40_fail(b"LEFT", left, eleft, n11, n12, n21, n22);
    }
    if test_test_kfunc_c_33_differ(right, eright) != 0 {
        test_test_kfunc_c_40_fail(b"RIGHT", right, eright, n11, n12, n21, n22);
    }
    if test_test_kfunc_c_33_differ(two, etwo) != 0 {
        test_test_kfunc_c_40_fail(b"TWO-TAIL", two, etwo, n11, n12, n21, n22);
    }
    if test_test_kfunc_c_33_differ(prob, eprob) != 0 {
        test_test_kfunc_c_40_fail(b"RESULT", prob, eprob, n11, n12, n21, n22);
    }
}

// original: main (htslib/test/test_kfunc.c:59)
pub unsafe fn test_test_kfunc_c_59_main(_argc: i32, _argv: *mut *mut u8) -> i32 {
    test_test_kfunc_c_48_test_fisher(
        2,
        1,
        0,
        31,
        1.0,
        0.005347593583,
        0.005347593583,
        0.005347593583,
    );
    test_test_kfunc_c_48_test_fisher(2, 1, 0, 1, 1.0, 0.5, 1.0, 0.5);
    test_test_kfunc_c_48_test_fisher(3, 1, 0, 0, 1.0, 1.0, 1.0, 1.0);
    test_test_kfunc_c_48_test_fisher(
        3,
        15,
        37,
        45,
        0.021479750169,
        0.995659202564,
        0.033161943699,
        0.017138952733,
    );
    test_test_kfunc_c_48_test_fisher(
        12,
        5,
        29,
        2,
        0.044554737835,
        0.994525206022,
        0.080268552074,
        0.039079943857,
    );

    test_test_kfunc_c_48_test_fisher(781, 23171, 4963, 2455001, 1.0, 0.0, 0.0, 0.0);
    test_test_kfunc_c_48_test_fisher(333, 381, 801722, 7664285, 1.0, 0.0, 0.0, 0.0);
    test_test_kfunc_c_48_test_fisher(4155, 4903, 805463, 8507517, 1.0, 0.0, 0.0, 0.0);
    test_test_kfunc_c_48_test_fisher(4455, 4903, 805463, 8507517, 1.0, 0.0, 0.0, 0.0);
    test_test_kfunc_c_48_test_fisher(5455, 4903, 805463, 8507517, 1.0, 0.0, 0.0, 0.0);

    test_test_kfunc_c_48_test_fisher(
        1,
        1,
        100000,
        1000000,
        0.991735477166,
        0.173555146661,
        0.173555146661,
        0.165290623827,
    );
    test_test_kfunc_c_48_test_fisher(1000, 1000, 100000, 1000000, 1.0, 0.0, 0.0, 0.0);
    test_test_kfunc_c_48_test_fisher(1000, 1000, 1000000, 100000, 0.0, 1.0, 0.0, 0.0);

    test_test_kfunc_c_48_test_fisher(49999, 10001, 90001, 49999, 1.0, 0.0, 0.0, 0.0);
    test_test_kfunc_c_48_test_fisher(50000, 10000, 90000, 50000, 1.0, 0.0, 0.0, 0.0);
    test_test_kfunc_c_48_test_fisher(50001, 9999, 89999, 50001, 1.0, 0.0, 0.0, 0.0);
    test_test_kfunc_c_48_test_fisher(10000, 50000, 130000, 10000, 0.0, 1.0, 0.0, 0.0);

    if TEST_KFUNC_NFAILED > 0 {
        let plural = if TEST_KFUNC_NFAILED == 1 { "" } else { "s" };
        eprintln!(
            "Failed {} test case{}",
            TEST_KFUNC_NFAILED,
            plural,
        );
        return libc::EXIT_FAILURE;
    }
    libc::EXIT_SUCCESS
}
