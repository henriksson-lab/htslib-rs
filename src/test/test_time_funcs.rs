// Pure UTC civil-date decomposition (relocated out of c_compat; used only here,
// as the reference inverse of `hts_time_gm` in the round-trip test below).
fn unix_time_utc_parts(now: libc::time_t) -> (i32, u32, u32, u32, u32, u32, usize) {
    let secs = i64::from(now);
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let hour = (sod / 3_600) as u32;
    let minute = ((sod % 3_600) / 60) as u32;
    let second = (sod % 60) as u32;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = (mp + if mp < 10 { 3 } else { -9 }) as u32;
    if month <= 2 {
        year += 1;
    }
    let weekday = (days + 4).rem_euclid(7) as usize;
    (year, month, day, hour, minute, second, weekday)
}

// original: test_normalised (htslib/test/test_time_funcs.c:36)
pub unsafe fn test_test_time_funcs_c_36_test_normalised(
    start: libc::time_t,
    end: libc::time_t,
    incr: libc::time_t,
) -> i32 {
    let mut i = start;
    while i < end {
        let (year, month, day, hour, min, sec, wday) = unix_time_utc_parts(i);
        let mut utc: libc::tm = std::mem::zeroed();
        utc.tm_sec = sec as i32;
        utc.tm_min = min as i32;
        utc.tm_hour = hour as i32;
        utc.tm_mday = day as i32;
        utc.tm_mon = month as i32 - 1;
        utc.tm_year = year - 1900;
        utc.tm_wday = wday as i32;
        let j = crate::htslib_rs::hts::hts_time_gm(&mut utc);
        if i != j {
            eprintln!("hts_time_gm() failed, got {} expected {}", j as i64, i as i64);
            return 1;
        }
        i += incr;
    }
    0
}

// original: test_specific (htslib/test/test_time_funcs.c:53)
pub unsafe fn test_test_time_funcs_c_53_test_specific(
    year: i32,
    mon: i32,
    mday: i32,
    hour: i32,
    min: i32,
    sec: i32,
    expected: libc::time_t,
) -> i32 {
    let mut utc: libc::tm = std::mem::zeroed();
    utc.tm_sec = sec;
    utc.tm_min = min;
    utc.tm_hour = hour;
    utc.tm_mday = mday;
    utc.tm_mon = mon - 1;
    utc.tm_year = year - 1900;
    utc.tm_wday = 0;
    utc.tm_yday = 0;
    utc.tm_isdst = 0;

    let res = crate::htslib_rs::hts::hts_time_gm(&mut utc);
    if res != expected {
        eprintln!(
            "hts_time_gm() failed for {:4}/{:02}/{:02} {:02}:{:02}:{:02} : got {} expected {}",
            year, mon, mday, hour, min, sec, res as i64, expected as i64
        );
        return 1;
    }
    0
}

// original: main (htslib/test/test_time_funcs.c:68)
pub unsafe fn test_test_time_funcs_c_68_main(_argc: i32, _argv: *mut *mut u8) -> i32 {
    let mut res = 0;
    let int_max = i32::MAX as libc::time_t;

    if test_test_time_funcs_c_36_test_normalised(0, int_max - 1000, 1000) != 0 {
        return libc::EXIT_FAILURE;
    }
    if std::mem::size_of::<libc::time_t>() >= 8
        && test_test_time_funcs_c_36_test_normalised(
            int_max - 1000,
            ((i32::MAX as i64) * 2) as libc::time_t,
            1000,
        ) != 0
    {
        return libc::EXIT_FAILURE;
    }

    // 2022-06-14 12:32:10
    res |= test_test_time_funcs_c_53_test_specific(2022, 6, 14, 12, 32, 10, 1655209930);
    // 2022-06-14 12:32:10
    res |= test_test_time_funcs_c_53_test_specific(1993, 9, 10514, 12, 32, 10, 1655209930);
    // 2022-02-28 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2020, 2, 28, 12, 0, 0, 1582891200);
    // 2022-02-29 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2020, 2, 29, 12, 0, 0, 1582977600);
    // 2022-03-01 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2020, 2, 30, 12, 0, 0, 1583064000);
    // 2022-02-29 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2020, 3, 0, 12, 0, 0, 1582977600);
    // 2020-02-01 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2019, 14, 1, 12, 0, 0, 1580558400);
    // 2020-03-01 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2019, 15, 1, 12, 0, 0, 1583064000);
    // 2021-03-01 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2019, 27, 1, 12, 0, 0, 1614600000);
    // 2024-02-01 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2019, 62, 1, 12, 0, 0, 1706788800);
    // 2024-03-01 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2019, 63, 1, 12, 0, 0, 1709294400);
    // 2020-12-31 23:59:59
    res |= test_test_time_funcs_c_53_test_specific(2021, 0, 31, 23, 59, 59, 1609459199);
    // 2020-03-01 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2021, -9, 1, 12, 0, 0, 1583064000);
    // 2020-02-01 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2021, -10, 1, 12, 0, 0, 1580558400);
    // 2019-02-01 12:00:00
    res |= test_test_time_funcs_c_53_test_specific(2021, -22, 1, 12, 0, 0, 1549022400);
    // 1970-01-01 00:00:00
    res |= test_test_time_funcs_c_53_test_specific(1970, 1, 1, 0, 0, 0, 0);
    // 2038-01-19 03:14:07
    res |= test_test_time_funcs_c_53_test_specific(1970, 1, 1, 0, 0, i32::MAX, int_max);
    // 2038-01-19 03:14:07
    res |= test_test_time_funcs_c_53_test_specific(2038, 1, 19, 3, 14, 7, int_max);
    if std::mem::size_of::<libc::time_t>() < 8 {
        // 2038-01-19 03:14:08
        res |= test_test_time_funcs_c_53_test_specific(2038, 1, 19, 3, 14, 8, -1);
    } else {
        // 2038-01-19 03:14:08
        res |= test_test_time_funcs_c_53_test_specific(2038, 1, 19, 3, 14, 8, int_max + 1);
    }

    if res == 0 {
        libc::EXIT_SUCCESS
    } else {
        libc::EXIT_FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::unix_time_utc_parts;

    #[test]
    fn unix_time_utc_parts_matches_known_dates() {
        assert_eq!(unix_time_utc_parts(0), (1970, 1, 1, 0, 0, 0, 4));
        assert_eq!(unix_time_utc_parts(1_748_868_896), (2025, 6, 2, 12, 54, 56, 1));
    }
}
