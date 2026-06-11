use crate::htslib_rs::{
    hts::{hts_pos_t, isspace_c, kputsn, kstring_t},
    regidx::{
        regidx_c_198_regidx_insert, regidx_c_246_regidx_init, regidx_c_311_regidx_destroy,
        regidx_c_401_regidx_overlap, regidx_c_466_regidx_parse_bed, regidx_c_498_regidx_parse_tab,
        regidx_c_545_regidx_parse_reg, regidx_c_584_regitr_init, regidx_c_606_regitr_destroy,
        regidx_c_612_regitr_overlap, regidx_c_646_regitr_loop, regidx_parse_f,
    },
};
use std::ptr;

unsafe extern "C" {
    static mut optarg: *mut u8;
    static mut optind: i32;
}

static mut VERBOSE: i32 = 0;

// original: custom_parse (htslib/test/test-regidx.c:75)
//
// In the owned model the payload buffer is a fixed-size byte slice rather than
// a malloc'd `char*`; we copy the fourth whitespace-separated field directly
// into it as a NUL-terminated string. Callers size the payload buffer large
// enough to hold the field.
pub fn test_test_regidx_c_75_custom_parse(
    line: &[u8],
    out: &mut crate::htslib_rs::regidx::ParsedRegion,
    payload: &mut [u8],
    usr: Option<&mut Vec<u8>>,
) -> i32 {
    let ret = regidx_c_498_regidx_parse_tab(line, out, payload, usr);
    if ret != 0 {
        return ret;
    }

    let mut ss = 0usize;
    while ss < line.len() && isspace_c(line[ss] as i8) != 0 {
        ss += 1;
    }
    for _i in 0..3 {
        while ss < line.len() && isspace_c(line[ss] as i8) == 0 {
            ss += 1;
        }
        if ss >= line.len() {
            return -2;
        }
        while ss < line.len() && isspace_c(line[ss] as i8) != 0 {
            ss += 1;
        }
    }
    if ss >= line.len() {
        return -2;
    }

    let mut se = ss;
    while se < line.len() && isspace_c(line[se] as i8) == 0 {
        se += 1;
    }
    let field = &line[ss..se];
    payload[..field.len()].copy_from_slice(field);
    payload[field.len()] = 0;
    0
}

// original: custom_free (htslib/test/test-regidx.c:100)
//
// The owned payload buffer is plain bytes with no heap allocation to release,
// so freeing is a no-op.
pub fn test_test_regidx_c_100_custom_free(_payload: &mut [u8]) {}

// original: test_sequential_access (htslib/test/test-regidx.c:106)
pub unsafe fn test_test_regidx_c_106_test_sequential_access() {
    let idx = regidx_c_246_regidx_init(
        None,
        Some(test_test_regidx_c_75_custom_parse as regidx_parse_f),
        Some(test_test_regidx_c_100_custom_free as fn(&mut [u8])),
        64,
        None,
    );
    if idx.is_none() {
        eprintln!("init failed");
        libc::exit(-1);
    }
    let mut idx = idx.unwrap();

    let mut str_ = kstring_t::default();
    let n = 10;
    for i in 0..n {
        let beg = 10 * (i + 1);
        str_.data.clear();
        let line = format!("1\t{}\t{}\t{}", beg, beg, beg).into_bytes();
        kputsn(&line, line.len(), &mut str_);
        if regidx_c_198_regidx_insert(&mut idx, &str_.data) != 0 {
            eprintln!("insert failed: {}", String::from_utf8_lossy(&str_.data));
            libc::exit(-1);
        }
    }

    let mut itr = regidx_c_584_regitr_init(&mut idx);
    let mut i = 0;
    while regidx_c_646_regitr_loop(&mut itr) != 0 {
        if (*itr).beg != (*itr).end || (*itr).beg + 1 != 10 * (i + 1) as hts_pos_t {
            eprintln!(
                "listing failed, expected {}, found {}",
                10 * (i + 1),
                (*itr).beg + 1
            );
            libc::exit(-1);
        }
        str_.data.clear();
        let line = format!("{}", (*itr).beg + 1).into_bytes();
        kputsn(&line, line.len(), &mut str_);
        let payload_bytes = &(*itr).payload;
        let payload_str = &payload_bytes[..payload_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(payload_bytes.len())];
        if payload_str != str_.data.as_slice() {
            eprintln!(
                "listing failed, expected payload \"{}\", found \"{}\"",
                String::from_utf8_lossy(&str_.data),
                String::from_utf8_lossy(payload_str)
            );
            libc::exit(-1);
        }
        i += 1;
    }
    if i != n {
        eprintln!("Expected {} regions, listed {}", n, i);
        libc::exit(-1);
    }
    if VERBOSE >= 2 {
        eprintln!("ok: listed {} regions", n);
    }

    regidx_c_606_regitr_destroy(itr);
    regidx_c_311_regidx_destroy(idx);
    drop(str_);
}

// original: test_custom_payload (htslib/test/test-regidx.c:143)
pub unsafe fn test_test_regidx_c_143_test_custom_payload() {
    let idx = regidx_c_246_regidx_init(
        None,
        Some(test_test_regidx_c_75_custom_parse as regidx_parse_f),
        Some(test_test_regidx_c_100_custom_free as fn(&mut [u8])),
        64,
        None,
    );
    if idx.is_none() {
        eprintln!("init failed");
        libc::exit(-1);
    }
    let mut idx = idx.unwrap();

    let mut line: &[u8] = b"1 10000000 10000000 1:10000000-10000000";
    if regidx_c_198_regidx_insert(&mut idx, line) != 0 {
        eprintln!("insert failed: {}", String::from_utf8_lossy(line));
        libc::exit(-1);
    }
    line = b"1 20000000 20000001 1:20000000-20000001";
    if regidx_c_198_regidx_insert(&mut idx, line) != 0 {
        eprintln!("insert failed: {}", String::from_utf8_lossy(line));
        libc::exit(-1);
    }
    line = b"1 20000002 20000002 1:20000002-20000002";
    if regidx_c_198_regidx_insert(&mut idx, line) != 0 {
        eprintln!("insert failed: {}", String::from_utf8_lossy(line));
        libc::exit(-1);
    }
    line = b"1 30000000 30000000 1:30000000-30000000";
    if regidx_c_198_regidx_insert(&mut idx, line) != 0 {
        eprintln!("insert failed: {}", String::from_utf8_lossy(line));
        libc::exit(-1);
    }
    line = b"1 8000000000 8000000000 1:8000000000-8000000000";
    if regidx_c_198_regidx_insert(&mut idx, line) != 0 {
        eprintln!("insert failed: {}", String::from_utf8_lossy(line));
        libc::exit(-1);
    }

    let mut itr = regidx_c_584_regitr_init(&mut idx);
    let mut from: hts_pos_t = 10000000;
    let mut to: hts_pos_t = 10000000;

    if regidx_c_401_regidx_overlap(&mut idx, b"1", from - 1, to - 1, Some(&mut itr)) == 0 {
        eprintln!("query failed: 1:{}-{}", from, to);
        libc::exit(-1);
    }
    let payload_str = {
        let p = &(*itr).payload;
        p[..p.iter().position(|&b| b == 0).unwrap_or(p.len())].to_vec()
    };
    if payload_str.as_slice() != b"1:10000000-10000000" {
        eprintln!(
            "query failed: 1:{}-{} vs {}",
            from,
            to,
            String::from_utf8_lossy(&payload_str)
        );
        libc::exit(-1);
    }
    if regidx_c_401_regidx_overlap(&mut idx, b"1", from - 2, to - 1, Some(&mut itr)) == 0 {
        eprintln!("query failed: 1:{}-{}", from - 1, to);
        libc::exit(-1);
    }
    if regidx_c_401_regidx_overlap(&mut idx, b"1", from - 2, to + 3, Some(&mut itr)) == 0 {
        eprintln!("query failed: 1:{}-{}", from - 1, to + 2);
        libc::exit(-1);
    }
    if regidx_c_401_regidx_overlap(&mut idx, b"1", from - 2, to - 2, Some(&mut itr)) != 0 {
        eprintln!("query failed: 1:{}-{}", from - 1, to - 1);
        libc::exit(-1);
    }

    from = 20000000;
    to = 20000000;
    if regidx_c_401_regidx_overlap(&mut idx, b"1", from - 1, to - 1, Some(&mut itr)) == 0 {
        eprintln!("query failed: 1:{}-{}", from, to);
        libc::exit(-1);
    }

    from = 20000002;
    to = 20000002;
    if regidx_c_401_regidx_overlap(&mut idx, b"1", from - 1, to - 1, Some(&mut itr)) == 0 {
        eprintln!("query failed: 1:{}-{}", from, to);
        libc::exit(-1);
    }

    from = 30000000;
    to = 30000000;
    if regidx_c_401_regidx_overlap(&mut idx, b"1", from - 1, to - 1, Some(&mut itr)) == 0 {
        eprintln!("query failed: 1:{}-{}", from, to);
        libc::exit(-1);
    }

    from = 8000000000;
    to = 8000000000;
    if regidx_c_401_regidx_overlap(&mut idx, b"1", from - 1, to - 1, Some(&mut itr)) == 0 {
        eprintln!("query failed: 1:{}-{}", from, to);
        libc::exit(-1);
    }

    from &= 0xffffffffu32 as hts_pos_t;
    to &= 0xffffffffu32 as hts_pos_t;
    if regidx_c_401_regidx_overlap(&mut idx, b"1", from - 1, to - 1, Some(&mut itr)) != 0 {
        eprintln!("query should not succeed: 1:{}-{}", from, to);
        libc::exit(-1);
    }

    regidx_c_606_regitr_destroy(itr);
    regidx_c_311_regidx_destroy(idx);
}

// original: get_random_region (htslib/test/test-regidx.c:190)
pub unsafe fn test_test_regidx_c_190_get_random_region(
    min: u32,
    max: u32,
    beg: &mut u32,
    end: &mut u32,
) {
    let b = libc::rand() as u64;
    let e = libc::rand() as u64;
    *beg = min + ((b * (max - min) as u64) / libc::RAND_MAX as u64) as u32;
    *end = *beg + ((e * (max - *beg) as u64) / libc::RAND_MAX as u64) as u32;
}

// original: test_random (htslib/test/test-regidx.c:197)
pub unsafe fn test_test_regidx_c_197_test_random(nregs: i32, mut min: u32, mut max: u32) {
    min -= 1;
    max -= 1;

    let idx = regidx_c_246_regidx_init(
        None,
        Some(test_test_regidx_c_75_custom_parse as regidx_parse_f),
        Some(test_test_regidx_c_100_custom_free as fn(&mut [u8])),
        64,
        None,
    );
    if idx.is_none() {
        eprintln!("init failed");
        libc::exit(-1);
    }
    let mut idx = idx.unwrap();

    let mut beg: u32 = 0;
    let mut end: u32 = 0;
    test_test_regidx_c_190_get_random_region(min, max, &mut beg, &mut end);

    let mut nexp = 0;
    let mut str_ = kstring_t::default();
    for _i in 0..nregs {
        let mut b: u32 = 0;
        let mut e: u32 = 0;
        test_test_regidx_c_190_get_random_region(min, max, &mut b, &mut e);
        str_.data.clear();
        let line = format!("1\t{}\t{}\t1:{}-{}", b + 1, e + 1, b + 1, e + 1).into_bytes();
        kputsn(&line, line.len(), &mut str_);
        if regidx_c_198_regidx_insert(&mut idx, &str_.data) != 0 {
            eprintln!("insert failed: {}", String::from_utf8_lossy(&str_.data));
            libc::exit(-1);
        }
        if e >= beg && b <= end {
            nexp += 1;
        }
    }

    let mut itr = regidx_c_584_regitr_init(&mut idx);
    let mut nhit = 0;
    let ret =
        regidx_c_401_regidx_overlap(&mut idx, b"1", beg as hts_pos_t, end as hts_pos_t, Some(&mut itr));
    if nexp != 0 && ret == 0 {
        eprintln!(
            "query failed, expected {} overlap(s), found none: {}-{}",
            nexp,
            beg + 1,
            end + 1
        );
        libc::exit(-1);
    }
    if nexp == 0 && ret != 0 {
        eprintln!(
            "query failed, expected no overlaps, found some: {}-{}",
            beg + 1,
            end + 1
        );
        libc::exit(-1);
    }
    while ret != 0 && regidx_c_612_regitr_overlap(&mut itr) != 0 {
        str_.data.clear();
        let line = format!("1:{}-{}", (*itr).beg + 1, (*itr).end + 1).into_bytes();
        kputsn(&line, line.len(), &mut str_);
        let payload_str = {
            let p = &(*itr).payload;
            p[..p.iter().position(|&b| b == 0).unwrap_or(p.len())].to_vec()
        };
        if str_.data.as_slice() != payload_str.as_slice() {
            eprintln!(
                "query failed, incorrect payload: {} vs {} ({}-{})",
                String::from_utf8_lossy(&str_.data),
                String::from_utf8_lossy(&payload_str),
                beg + 1,
                end + 1
            );
            libc::exit(-1);
        }
        if (*itr).beg > end as hts_pos_t || (*itr).end < beg as hts_pos_t {
            eprintln!(
                "query failed, incorrect hit: {}-{} vs {}-{}, payload {}",
                beg + 1,
                end + 1,
                (*itr).beg + 1,
                (*itr).end + 1,
                String::from_utf8_lossy(&payload_str)
            );
            libc::exit(-1);
        }
        nhit += 1;
    }
    if nexp != nhit {
        eprintln!(
            "query failed, expected {} overlap(s), found {}: {}-{}",
            nexp,
            nhit,
            beg + 1,
            end + 1
        );
        libc::exit(-1);
    }
    if VERBOSE >= 2 {
        eprintln!("ok: found {} overlaps", nexp);
    }

    regidx_c_606_regitr_destroy(itr);
    regidx_c_311_regidx_destroy(idx);
    drop(str_);
}

// original: test_explicit (htslib/test/test-regidx.c:246)
pub unsafe fn test_test_regidx_c_246_test_explicit(tgt: &[u8], qry: &[u8], exp: &[u8]) {
    let mut idx = regidx_c_246_regidx_init(
        None,
        Some(regidx_c_545_regidx_parse_reg as regidx_parse_f),
        None,
        0,
        None,
    )
    .expect("regidx init");

    let mut beg_i = 0usize;
    let mut str_ = kstring_t::default();
    while beg_i < tgt.len() {
        // NOTE: mirrors the original, which restarts the scan from the start
        // of the buffer rather than from beg_i.
        let mut end_i = 0usize;
        while end_i < tgt.len() && tgt[end_i] != b';' {
            end_i += 1;
        }
        str_.data.clear();
        let field = &tgt[beg_i..end_i];
        kputsn(field, field.len(), &mut str_);
        if VERBOSE >= 2 {
            eprintln!("insert: {}", String::from_utf8_lossy(&str_.data));
        }
        if regidx_c_198_regidx_insert(&mut idx, &str_.data) != 0 {
            eprintln!("insert failed: {}", String::from_utf8_lossy(&str_.data));
            libc::exit(-1);
        }
        beg_i = if end_i < tgt.len() { end_i + 1 } else { end_i };
    }

    let mut exp_i = 0usize;
    beg_i = 0;
    while beg_i < qry.len() {
        let mut end_i = 0usize;
        while end_i < qry.len() && qry[end_i] != b';' {
            end_i += 1;
        }
        str_.data.clear();
        let field = &qry[beg_i..end_i];
        kputsn(field, field.len(), &mut str_);
        beg_i = if end_i < qry.len() { end_i + 1 } else { end_i };

        let mut out = crate::htslib_rs::regidx::ParsedRegion::default();
        let mut parse_payload: Vec<u8> = Vec::new();
        if regidx_c_545_regidx_parse_reg(&str_.data, &mut out, &mut parse_payload, None) != 0 {
            eprintln!(
                "could not parse: {} in {}",
                String::from_utf8_lossy(&str_.data),
                String::from_utf8_lossy(qry)
            );
            libc::exit(-1);
        }
        let reg_beg = out.beg;
        let reg_end = out.end;
        // ParsedRegion.chr is an inclusive byte range into the parsed line.
        let chr_range = out.chr.clone().expect("parsed chromosome");
        let chr_bytes = str_.data[chr_range].to_vec();
        let hit = regidx_c_401_regidx_overlap(&mut idx, &chr_bytes, reg_beg, reg_end, None);
        if exp[exp_i] == b'1' {
            if hit == 0 {
                eprintln!(
                    "query failed, there should be a hit .. {}:{}-{}",
                    String::from_utf8_lossy(&chr_bytes),
                    reg_beg + 1,
                    reg_end + 1
                );
                libc::exit(-1);
            } else if VERBOSE >= 2 {
                eprintln!(
                    "ok: overlap found for {}:{}-{}",
                    String::from_utf8_lossy(&chr_bytes),
                    reg_beg + 1,
                    reg_end + 1
                );
            }
        } else if exp[exp_i] == b'0' {
            if hit != 0 {
                eprintln!(
                    "query failed, there should be no hit .. {}:{}-{}",
                    String::from_utf8_lossy(&chr_bytes),
                    reg_beg + 1,
                    reg_end + 1
                );
                libc::exit(-1);
            } else if VERBOSE >= 2 {
                eprintln!(
                    "ok: no overlap found for {}:{}-{}",
                    String::from_utf8_lossy(&chr_bytes),
                    reg_beg + 1,
                    reg_end + 1
                );
            }
        } else {
            eprintln!("could not parse: {}", String::from_utf8_lossy(exp));
            libc::exit(-1);
        }
        exp_i += 1;
    }

    drop(str_);
    regidx_c_311_regidx_destroy(idx);
}

// original: create_line_bed (htslib/test/test-regidx.c:307)
pub unsafe fn test_test_regidx_c_307_create_line_bed(
    line: &mut [u8],
    _size: usize,
    chr: &[u8],
    start: i32,
    end: i32,
) {
    let s = format!("{}\t{}\t{}\n", String::from_utf8_lossy(chr), start - 1, end).into_bytes();
    line[..s.len()].copy_from_slice(&s);
    line[s.len()] = 0;
}

// original: create_line_tab (htslib/test/test-regidx.c:311)
pub unsafe fn test_test_regidx_c_311_create_line_tab(
    line: &mut [u8],
    _size: usize,
    chr: &[u8],
    start: i32,
    end: i32,
) {
    let s = format!("{}\t{}\t{}\n", String::from_utf8_lossy(chr), start, end).into_bytes();
    line[..s.len()].copy_from_slice(&s);
    line[s.len()] = 0;
}

// original: create_line_reg (htslib/test/test-regidx.c:315)
pub unsafe fn test_test_regidx_c_315_create_line_reg(
    line: &mut [u8],
    _size: usize,
    chr: &[u8],
    start: i32,
    end: i32,
) {
    let s = format!("{}:{}-{}\n", String::from_utf8_lossy(chr), start, end).into_bytes();
    line[..s.len()].copy_from_slice(&s);
    line[s.len()] = 0;
}

type set_line_f = unsafe fn(&mut [u8], usize, &[u8], i32, i32);

// original: test (htslib/test/test-regidx.c:322)
pub unsafe fn test_test_regidx_c_322_test(set_line: set_line_f, parse: regidx_parse_f) {
    let idx = regidx_c_246_regidx_init(None, Some(parse), None, 0, None);
    if idx.is_none() {
        eprintln!("init failed");
        libc::exit(-1);
    }
    let mut idx = idx.unwrap();

    let mut line = [0u8; 250];
    let chr: &[u8] = b"1";
    let n = 10;
    for i in 1..n {
        let mut start = 10 * i;
        let mut end = start;
        let line_len = line.len();
        set_line(&mut line, line_len, chr, start, end);
        let line_bytes = &line[..line.iter().position(|&b| b == 0).unwrap_or(line.len())];
        if VERBOSE >= 2 {
            eprint!("insert: {}", String::from_utf8_lossy(line_bytes));
        }
        if regidx_c_198_regidx_insert(&mut idx, line_bytes) != 0 {
            eprintln!("insert failed: {}", String::from_utf8_lossy(line_bytes));
            libc::exit(-1);
        }

        start = 10 * i + 1;
        end = start;
        let line_len = line.len();
        set_line(&mut line, line_len, chr, start, end);
        let line_bytes = &line[..line.iter().position(|&b| b == 0).unwrap_or(line.len())];
        if VERBOSE >= 2 {
            eprint!("insert: {}", String::from_utf8_lossy(line_bytes));
        }
        if regidx_c_198_regidx_insert(&mut idx, line_bytes) != 0 {
            eprintln!("insert failed: {}", String::from_utf8_lossy(line_bytes));
            libc::exit(-1);
        }

        start = 20000 * i;
        end = start + 2000;
        let line_len = line.len();
        set_line(&mut line, line_len, chr, start, end);
        let line_bytes = &line[..line.iter().position(|&b| b == 0).unwrap_or(line.len())];
        if VERBOSE >= 2 {
            eprint!("insert: {}", String::from_utf8_lossy(line_bytes));
        }
        if regidx_c_198_regidx_insert(&mut idx, line_bytes) != 0 {
            eprintln!("insert failed: {}", String::from_utf8_lossy(line_bytes));
            libc::exit(-1);
        }
    }

    let mut itr = regidx_c_584_regitr_init(&mut idx);
    for i in 1..n {
        let mut start = 10 * i - 1;
        let mut end = start;
        if regidx_c_401_regidx_overlap(
            &mut idx,
            b"1",
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            Some(&mut itr),
        ) != 0
        {
            eprintln!(
                "query failed, there should be no hit: {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            eprintln!(
                "ok: no overlap found for {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
        }

        start = 10 * i;
        end = start;
        if regidx_c_401_regidx_overlap(
            &mut idx,
            b"1",
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            Some(&mut itr),
        ) == 0
        {
            eprintln!(
                "query failed, there should be a hit: {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            eprintln!(
                "ok: overlap(s) found for {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
        }
        let mut nhit = 0;
        while regidx_c_612_regitr_overlap(&mut itr) != 0 {
            if (*itr).beg > (end - 1) as hts_pos_t || (*itr).end < (start - 1) as hts_pos_t {
                eprintln!(
                    "query failed, incorrect region: {}-{} for {}-{}",
                    (*itr).beg + 1,
                    (*itr).end + 1,
                    start,
                    end
                );
                libc::exit(-1);
            }
            if VERBOSE >= 2 {
                eprintln!("\t {}-{}", (*itr).beg + 1, (*itr).end + 1);
            }
            nhit += 1;
        }
        if nhit != 1 {
            eprintln!(
                "query failed, expected one hit, found {}: {}:{}-{}",
                nhit,
                String::from_utf8_lossy(chr),
                start,
                end
            );
            libc::exit(-1);
        }

        start = 10 * i + 1;
        end = start;
        if regidx_c_401_regidx_overlap(
            &mut idx,
            b"1",
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            Some(&mut itr),
        ) == 0
        {
            eprintln!(
                "query failed, there should be a hit: {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            eprintln!(
                "ok: overlap(s) found for {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
        }
        nhit = 0;
        while regidx_c_612_regitr_overlap(&mut itr) != 0 {
            if (*itr).beg > (end - 1) as hts_pos_t || (*itr).end < (start - 1) as hts_pos_t {
                eprintln!(
                    "query failed, incorrect region: {}-{} for {}-{}",
                    (*itr).beg + 1,
                    (*itr).end + 1,
                    start,
                    end
                );
                libc::exit(-1);
            }
            if VERBOSE >= 2 {
                eprintln!("\t {}-{}", (*itr).beg + 1, (*itr).end + 1);
            }
            nhit += 1;
        }
        if nhit != 1 {
            eprintln!(
                "query failed, expected one hit, found {}: {}:{}-{}",
                nhit,
                String::from_utf8_lossy(chr),
                start,
                end
            );
            libc::exit(-1);
        }

        start = 10 * i;
        end = start + 1;
        if regidx_c_401_regidx_overlap(
            &mut idx,
            b"1",
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            Some(&mut itr),
        ) == 0
        {
            eprintln!(
                "query failed, there should be a hit: {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            eprintln!(
                "ok: overlap(s) found for {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
        }
        nhit = 0;
        while regidx_c_612_regitr_overlap(&mut itr) != 0 {
            if (*itr).beg > (end - 1) as hts_pos_t || (*itr).end < (start - 1) as hts_pos_t {
                eprintln!(
                    "query failed, incorrect region: {}-{} for {}-{}",
                    (*itr).beg + 1,
                    (*itr).end + 1,
                    start,
                    end
                );
                libc::exit(-1);
            }
            if VERBOSE >= 2 {
                eprintln!("\t {}-{}", (*itr).beg + 1, (*itr).end + 1);
            }
            nhit += 1;
        }
        if nhit != 2 {
            eprintln!(
                "query failed, expected two hits, found {}: {}:{}-{}",
                nhit,
                String::from_utf8_lossy(chr),
                start,
                end
            );
            libc::exit(-1);
        }

        start = 20000 * i - 5000;
        end = 20000 * i + 3000;
        let line_len = line.len();
        set_line(&mut line, line_len, chr, start, end);
        if regidx_c_401_regidx_overlap(
            &mut idx,
            b"1",
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            Some(&mut itr),
        ) == 0
        {
            eprintln!(
                "query failed, there should be a hit: {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            eprintln!(
                "ok: overlap(s) found for {}:{}-{}",
                String::from_utf8_lossy(chr),
                start,
                end
            );
        }
        nhit = 0;
        while regidx_c_612_regitr_overlap(&mut itr) != 0 {
            if (*itr).beg > (end - 1) as hts_pos_t || (*itr).end < (start - 1) as hts_pos_t {
                eprintln!(
                    "query failed, incorrect region: {}-{} for {}-{}",
                    (*itr).beg + 1,
                    (*itr).end + 1,
                    start,
                    end
                );
                libc::exit(-1);
            }
            if VERBOSE >= 2 {
                eprintln!("\t {}-{}", (*itr).beg + 1, (*itr).end + 1);
            }
            nhit += 1;
        }
        if nhit != 1 {
            eprintln!(
                "query failed, expected one hit, found {}: {}:{}-{}",
                nhit,
                String::from_utf8_lossy(chr),
                start,
                end
            );
            libc::exit(-1);
        }
    }
    regidx_c_606_regitr_destroy(itr);
    regidx_c_311_regidx_destroy(idx);
}

// original: usage (htslib/test/test-regidx.c:415)
pub unsafe fn test_test_regidx_c_415_usage() -> ! {
    eprintln!("Usage: test-regidx [OPTIONS]");
    eprintln!("Options:");
    eprintln!("   -h, --help          this help message");
    eprintln!("   -s, --seed <int>    random seed");
    eprintln!("   -v, --verbose       increase verbosity by giving multiple times");
    libc::exit(1);
}

// original: main (htslib/test/test-regidx.c:426)
pub unsafe fn test_test_regidx_c_426_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut loptions = [
        libc::option {
            name: c"help".as_ptr(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: b'h' as i32,
        },
        libc::option {
            name: c"verbose".as_ptr(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: b'v' as i32,
        },
        libc::option {
            name: c"seed".as_ptr(),
            has_arg: 1,
            flag: ptr::null_mut(),
            val: b's' as i32,
        },
        libc::option {
            name: ptr::null(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: 0,
        },
    ];
    let mut seed = libc::time(ptr::null_mut()) as i32;
    loop {
        let c = libc::getopt_long(
            argc,
            argv.cast(),
            c"hvs:".as_ptr(),
            loptions.as_mut_ptr(),
            ptr::null_mut(),
        );
        if c < 0 {
            break;
        }
        match c {
            x if x == b's' as i32 => seed = libc::atoi(optarg.cast()),
            x if x == b'v' as i32 => VERBOSE += 1,
            _ => test_test_regidx_c_415_usage(),
        }
    }

    if VERBOSE >= 1 {
        eprintln!("Testing sequential access");
    }
    test_test_regidx_c_106_test_sequential_access();

    if VERBOSE >= 1 {
        eprintln!("Testing TAB");
    }
    test_test_regidx_c_322_test(
        test_test_regidx_c_311_create_line_tab,
        regidx_c_498_regidx_parse_tab as regidx_parse_f,
    );

    if VERBOSE >= 1 {
        eprintln!("Testing REG");
    }
    test_test_regidx_c_322_test(
        test_test_regidx_c_315_create_line_reg,
        regidx_c_545_regidx_parse_reg as regidx_parse_f,
    );

    if VERBOSE >= 1 {
        eprintln!("Testing BED");
    }
    test_test_regidx_c_322_test(
        test_test_regidx_c_307_create_line_bed,
        regidx_c_466_regidx_parse_bed as regidx_parse_f,
    );

    if VERBOSE >= 1 {
        eprintln!("Testing custom payload");
    }
    test_test_regidx_c_143_test_custom_payload();

    if VERBOSE >= 1 {
        eprintln!("Testing cases encountered in past");
    }
    test_test_regidx_c_246_test_explicit(
        b"12:2064519-2064763",
        b"12:2064488-2067434",
        b"1",
    );

    let ntest = 1000;
    let nreg = 50;
    libc::srand(seed as libc::c_uint);
    if VERBOSE >= 1 {
        eprintln!(
            "{} randomized tests, {} regions per test. Random seed is {}",
            ntest, nreg, seed
        );
    }
    for _i in 0..ntest {
        test_test_regidx_c_197_test_random(nreg, 1, 1000);
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static GETOPT_LOCK: Mutex<()> = Mutex::new(());

    // `args` are NUL-terminated argv byte strings; getopt_long is still a raw
    // libc call so we hand it `*mut u8` pointers into these owned buffers.
    unsafe fn run_main(args: &mut [Vec<u8>]) -> i32 {
        // NOTE: callers must already hold `ORIGINAL_MAIN_LOCK` (see
        // src/test/mod.rs). `GETOPT_LOCK` is retained for backward-compat
        // but is now effectively a no-op while the global lock is held.
        let _guard = GETOPT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        optarg = ptr::null_mut();
        // optind = 0 forces glibc getopt full reinit (shared-process tests).
        optind = 0;
        VERBOSE = 0;
        let mut argv = args
            .iter_mut()
            .map(|arg| arg.as_mut_ptr())
            .collect::<Vec<*mut u8>>();
        test_test_regidx_c_426_main(argv.len() as i32, argv.as_mut_ptr())
    }

    #[test]
    fn original_test_regidx_main_runs_full_harness_with_fixed_seed() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut args = vec![
            b"test-regidx\0".to_vec(),
            b"-s\0".to_vec(),
            b"17\0".to_vec(),
        ];

        unsafe {
            assert_eq!(run_main(&mut args), 0);
        }
    }

    #[test]
    fn original_test_regidx_create_line_helpers_match_expected_formats() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            let mut line = [0u8; 64];
            let line_len = line.len();
            let chr: &[u8] = b"chrA";

            test_test_regidx_c_311_create_line_tab(&mut line, line_len, chr, 11, 13);
            let line_bytes = &line[..line.iter().position(|&b| b == 0).unwrap_or(line.len())];
            assert_eq!(line_bytes, b"chrA\t11\t13\n");

            test_test_regidx_c_315_create_line_reg(&mut line, line_len, chr, 11, 13);
            let line_bytes = &line[..line.iter().position(|&b| b == 0).unwrap_or(line.len())];
            assert_eq!(line_bytes, b"chrA:11-13\n");

            test_test_regidx_c_307_create_line_bed(&mut line, line_len, chr, 11, 13);
            let line_bytes = &line[..line.iter().position(|&b| b == 0).unwrap_or(line.len())];
            assert_eq!(line_bytes, b"chrA\t10\t13\n");
        }
    }
}
