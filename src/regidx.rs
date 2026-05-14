use std::ffi::{c_char, c_int, c_void};

use crate::htslib_mini_rs::hts::{hts_parse_decimal, hts_pos_t, isspace_c};

pub const REGIDX_MAX: hts_pos_t = 1_i64 << 35;
pub const MAX_COOR_0: hts_pos_t = REGIDX_MAX;

#[repr(C)]
pub struct regidx_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct regitr_t {
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
    pub payload: *mut c_void,
    pub seq: *mut c_char,
    pub itr: *mut c_void,
}

#[repr(C)]
pub struct regidx_reg_t {
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
}

pub type regidx_parse_f = Option<
    unsafe extern "C" fn(
        *const c_char,
        *mut *mut c_char,
        *mut *mut c_char,
        *mut hts_pos_t,
        *mut hts_pos_t,
        *mut c_void,
        *mut c_void,
    ) -> c_int,
>;
pub type regidx_free_f = Option<unsafe extern "C" fn(*mut c_void)>;

unsafe extern "C" {
    #[link_name = "regidx_seq_nregs"]
    fn htslib_regidx_seq_nregs(idx: *mut regidx_t, seq: *const c_char) -> c_int;
    #[link_name = "regidx_nregs"]
    fn htslib_regidx_nregs(idx: *mut regidx_t) -> c_int;
    #[link_name = "regidx_seq_names"]
    fn htslib_regidx_seq_names(idx: *mut regidx_t, n: *mut c_int) -> *mut *mut c_char;
    #[link_name = "regidx_insert_list"]
    fn htslib_regidx_insert_list(idx: *mut regidx_t, line: *mut c_char, delim: c_char) -> c_int;
    #[link_name = "regidx_push"]
    fn htslib_regidx_push(
        idx: *mut regidx_t,
        chr_beg: *mut c_char,
        chr_end: *mut c_char,
        beg: hts_pos_t,
        end: hts_pos_t,
        payload: *mut c_void,
    ) -> c_int;
    #[link_name = "regidx_insert"]
    fn htslib_regidx_insert(idx: *mut regidx_t, line: *mut c_char) -> c_int;
    #[link_name = "regidx_init_string"]
    fn htslib_regidx_init_string(
        string: *const c_char,
        parsef: regidx_parse_f,
        freef: regidx_free_f,
        payload_size: usize,
        usr: *mut c_void,
    ) -> *mut regidx_t;
    #[link_name = "regidx_init"]
    fn htslib_regidx_init(
        fname: *const c_char,
        parsef: regidx_parse_f,
        freef: regidx_free_f,
        payload_size: usize,
        usr: *mut c_void,
    ) -> *mut regidx_t;
    #[link_name = "regidx_destroy"]
    fn htslib_regidx_destroy(idx: *mut regidx_t);
    #[link_name = "regidx_overlap"]
    fn htslib_regidx_overlap(
        idx: *mut regidx_t,
        chr: *const c_char,
        beg: hts_pos_t,
        end: hts_pos_t,
        itr: *mut regitr_t,
    ) -> c_int;
    #[link_name = "regitr_init"]
    fn htslib_regitr_init(idx: *mut regidx_t) -> *mut regitr_t;
    #[link_name = "regitr_reset"]
    fn htslib_regitr_reset(idx: *mut regidx_t, itr: *mut regitr_t);
    #[link_name = "regitr_destroy"]
    fn htslib_regitr_destroy(itr: *mut regitr_t);
    #[link_name = "regitr_overlap"]
    fn htslib_regitr_overlap(itr: *mut regitr_t) -> c_int;
    #[link_name = "regitr_loop"]
    fn htslib_regitr_loop(itr: *mut regitr_t) -> c_int;
    #[link_name = "regitr_copy"]
    fn htslib_regitr_copy(dst: *mut regitr_t, src: *mut regitr_t);
}

pub unsafe fn regidx_c_91_regidx_seq_nregs(idx: *mut regidx_t, seq: *const c_char) -> c_int {
    htslib_regidx_seq_nregs(idx, seq)
}

pub unsafe fn regidx_c_98_regidx_nregs(idx: *mut regidx_t) -> c_int {
    htslib_regidx_nregs(idx)
}

pub unsafe fn regidx_c_105_regidx_seq_names(idx: *mut regidx_t, n: *mut c_int) -> *mut *mut c_char {
    htslib_regidx_seq_names(idx, n)
}

pub unsafe fn regidx_c_111_regidx_insert_list(
    idx: *mut regidx_t,
    line: *mut c_char,
    delim: c_char,
) -> c_int {
    htslib_regidx_insert_list(idx, line, delim)
}

pub unsafe fn regidx_c_132_cmp_regs(a: *mut regidx_reg_t, b: *mut regidx_reg_t) -> c_int {
    if (*a).beg < (*b).beg {
        return -1;
    }
    if (*a).beg > (*b).beg {
        return 1;
    }
    if (*a).end < (*b).end {
        return 1;
    }
    if (*a).end > (*b).end {
        return -1;
    }
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    0
}

pub unsafe extern "C" fn regidx_c_142_cmp_reg_ptrs(a: *const c_void, b: *const c_void) -> c_int {
    regidx_c_132_cmp_regs(
        a.cast::<regidx_reg_t>() as *mut _,
        b.cast::<regidx_reg_t>() as *mut _,
    )
}

pub unsafe extern "C" fn regidx_c_146_cmp_reg_ptrs2(a: *const c_void, b: *const c_void) -> c_int {
    let ap = *(a.cast::<*mut regidx_reg_t>());
    let bp = *(b.cast::<*mut regidx_reg_t>());
    regidx_c_132_cmp_regs(ap, bp)
}

pub unsafe fn regidx_c_151_regidx_push(
    idx: *mut regidx_t,
    chr_beg: *mut c_char,
    chr_end: *mut c_char,
    beg: hts_pos_t,
    end: hts_pos_t,
    payload: *mut c_void,
) -> c_int {
    htslib_regidx_push(idx, chr_beg, chr_end, beg, end, payload)
}

pub unsafe fn regidx_c_198_regidx_insert(idx: *mut regidx_t, line: *mut c_char) -> c_int {
    htslib_regidx_insert(idx, line)
}

pub unsafe fn regidx_c_209_regidx_init_string(
    string: *const c_char,
    parsef: regidx_parse_f,
    freef: regidx_free_f,
    payload_size: usize,
    usr: *mut c_void,
) -> *mut regidx_t {
    htslib_regidx_init_string(string, parsef, freef, payload_size, usr)
}

pub unsafe fn regidx_c_246_regidx_init(
    fname: *const c_char,
    parsef: regidx_parse_f,
    freef: regidx_free_f,
    payload_size: usize,
    usr: *mut c_void,
) -> *mut regidx_t {
    htslib_regidx_init(fname, parsef, freef, payload_size, usr)
}

pub unsafe fn regidx_c_311_regidx_destroy(idx: *mut regidx_t) {
    htslib_regidx_destroy(idx);
}

pub unsafe fn regidx_c_401_regidx_overlap(
    idx: *mut regidx_t,
    chr: *const c_char,
    beg: hts_pos_t,
    end: hts_pos_t,
    itr: *mut regitr_t,
) -> c_int {
    htslib_regidx_overlap(idx, chr, beg, end, itr)
}

pub unsafe extern "C" fn regidx_c_466_regidx_parse_bed(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    _payload: *mut c_void,
    _usr: *mut c_void,
) -> c_int {
    let mut ss = line as *mut c_char;
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    if *ss == 0 {
        return -1;
    }
    if *ss == b'#' as c_char {
        return -1;
    }

    let mut se = ss;
    while *se != 0 && isspace_c(*se) == 0 {
        se = se.add(1);
    }

    *chr_beg = ss;
    *chr_end = se.sub(1);

    if *se == 0 {
        *beg = 0;
        *end = MAX_COOR_0;
        return 0;
    }

    ss = se.add(1);
    *beg = hts_parse_decimal(ss, &mut se, 0);
    if ss == se {
        return -2;
    }

    ss = se.add(1);
    *end = hts_parse_decimal(ss, &mut se, 0) - 1;
    if ss == se {
        return -2;
    }

    0
}

pub unsafe extern "C" fn regidx_c_498_regidx_parse_tab(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    _payload: *mut c_void,
    _usr: *mut c_void,
) -> c_int {
    let mut ss = line as *mut c_char;
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    if *ss == 0 {
        return -1;
    }
    if *ss == b'#' as c_char {
        return -1;
    }

    let mut se = ss;
    while *se != 0 && isspace_c(*se) == 0 {
        se = se.add(1);
    }

    *chr_beg = ss;
    *chr_end = se.sub(1);

    if *se == 0 {
        *beg = 0;
        *end = MAX_COOR_0;
        return 0;
    }

    ss = se.add(1);
    *beg = hts_parse_decimal(ss, &mut se, 0);
    if ss == se {
        return -2;
    }
    if *beg == 0 {
        return -2;
    }
    *beg -= 1;

    if *se == 0 || *se.add(1) == 0 {
        *end = *beg;
    } else {
        ss = se.add(1);
        *end = hts_parse_decimal(ss, &mut se, 0);
        if ss == se || (*se != 0 && isspace_c(*se) == 0) {
            *end = *beg;
        } else if *end == 0 {
            return -2;
        } else {
            *end -= 1;
        }
    }
    0
}

pub unsafe extern "C" fn regidx_c_538_regidx_parse_vcf(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    payload: *mut c_void,
    usr: *mut c_void,
) -> c_int {
    let ret = regidx_c_498_regidx_parse_tab(line, chr_beg, chr_end, beg, end, payload, usr);
    if ret == 0 {
        *end = *beg;
    }
    ret
}

pub unsafe extern "C" fn regidx_c_545_regidx_parse_reg(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    _payload: *mut c_void,
    _usr: *mut c_void,
) -> c_int {
    let mut ss = line as *mut c_char;
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    if *ss == 0 {
        return -1;
    }
    if *ss == b'#' as c_char {
        return -1;
    }

    let mut se = ss;
    while *se != 0 && *se != b':' as c_char {
        se = se.add(1);
    }

    *chr_beg = ss;
    *chr_end = se.sub(1);

    if *se == 0 {
        *beg = 0;
        *end = MAX_COOR_0;
        return 0;
    }

    ss = se.add(1);
    *beg = hts_parse_decimal(ss, &mut se, 0);
    if ss == se {
        return -2;
    }
    if *beg == 0 {
        return -2;
    }
    *beg -= 1;

    if *se == 0 || *se.add(1) == 0 {
        *end = if *se == b'-' as c_char {
            MAX_COOR_0
        } else {
            *beg
        };
    } else {
        ss = se.add(1);
        *end = hts_parse_decimal(ss, &mut se, 0);
        if ss == se {
            *end = *beg;
        } else if *end == 0 {
            return -2;
        } else {
            *end -= 1;
        }
    }
    0
}

pub unsafe fn regidx_c_584_regitr_init(regidx: *mut regidx_t) -> *mut regitr_t {
    htslib_regitr_init(regidx)
}

pub unsafe fn regidx_c_599_regitr_reset(regidx: *mut regidx_t, regitr: *mut regitr_t) {
    htslib_regitr_reset(regidx, regitr);
}

pub unsafe fn regidx_c_606_regitr_destroy(regitr: *mut regitr_t) {
    htslib_regitr_destroy(regitr);
}

pub unsafe fn regidx_c_612_regitr_overlap(regitr: *mut regitr_t) -> c_int {
    htslib_regitr_overlap(regitr)
}

pub unsafe fn regidx_c_646_regitr_loop(regitr: *mut regitr_t) -> c_int {
    htslib_regitr_loop(regitr)
}

pub unsafe fn regidx_c_681_regitr_copy(dst: *mut regitr_t, src: *mut regitr_t) {
    htslib_regitr_copy(dst, src);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn regidx_parse_bed_uses_zero_based_right_open_coordinates() {
        unsafe {
            let line = CString::new("chr1\t10\t20").unwrap();
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;
            assert_eq!(
                regidx_c_466_regidx_parse_bed(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!(beg, 10);
            assert_eq!(end, 19);
            assert_eq!(chr_end.offset_from(chr_beg), 3);
        }
    }

    #[test]
    fn regidx_parse_tab_and_vcf_use_one_based_positions() {
        unsafe {
            let line = CString::new("chr2 11 14").unwrap();
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;
            assert_eq!(
                regidx_c_498_regidx_parse_tab(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (10, 13));

            assert_eq!(
                regidx_c_538_regidx_parse_vcf(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (10, 10));
        }
    }

    #[test]
    fn regidx_parse_reg_handles_open_ended_regions() {
        unsafe {
            let line = CString::new("chr3:42-").unwrap();
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;
            assert_eq!(
                regidx_c_545_regidx_parse_reg(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (41, MAX_COOR_0));
        }
    }

    #[test]
    fn regidx_comparators_order_by_begin_then_longer_end_then_address() {
        unsafe {
            let mut a = regidx_reg_t { beg: 10, end: 20 };
            let mut b = regidx_reg_t { beg: 10, end: 25 };
            let mut c = regidx_reg_t { beg: 11, end: 12 };

            assert_eq!(regidx_c_132_cmp_regs(&mut a, &mut b), 1);
            assert_eq!(regidx_c_132_cmp_regs(&mut c, &mut b), 1);
            assert_eq!(
                regidx_c_142_cmp_reg_ptrs(
                    (&mut b as *mut regidx_reg_t).cast(),
                    (&mut a as *mut regidx_reg_t).cast()
                ),
                -1
            );

            let mut ap = &mut a as *mut regidx_reg_t;
            let mut bp = &mut b as *mut regidx_reg_t;
            assert_eq!(
                regidx_c_146_cmp_reg_ptrs2(
                    (&mut bp as *mut *mut regidx_reg_t).cast(),
                    (&mut ap as *mut *mut regidx_reg_t).cast()
                ),
                -1
            );
        }
    }

    #[test]
    fn regidx_public_api_builds_queries_and_iterates_regions() {
        unsafe {
            let data = CString::new("chr1\t5\t7\nchr1\t9\t10\nchr2\t3\n").unwrap();
            let idx = regidx_c_209_regidx_init_string(
                data.as_ptr(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let chr1 = CString::new("chr1").unwrap();
            let chr3 = CString::new("chr3").unwrap();
            assert_eq!(regidx_c_91_regidx_seq_nregs(idx, chr1.as_ptr()), 2);
            assert_eq!(regidx_c_91_regidx_seq_nregs(idx, chr3.as_ptr()), 0);
            assert_eq!(regidx_c_98_regidx_nregs(idx), 3);

            let mut n = 0;
            let names = regidx_c_105_regidx_seq_names(idx, &mut n);
            assert_eq!(n, 2);
            assert!(!names.is_null());

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, chr1.as_ptr(), 4, 4, itr),
                1
            );
            assert_eq!((*itr).beg, 4);
            assert_eq!((*itr).end, 6);
            assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
            assert_eq!(regidx_c_612_regitr_overlap(itr), 0);

            regidx_c_599_regitr_reset(idx, itr);
            assert_eq!(regidx_c_646_regitr_loop(itr), 1);
            assert_eq!((*itr).beg, 4);
            let copy = regidx_c_584_regitr_init(idx);
            assert!(!copy.is_null());
            regidx_c_681_regitr_copy(copy, itr);
            assert_eq!((*copy).beg, (*itr).beg);

            regidx_c_606_regitr_destroy(copy);
            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_init_insert_push_and_insert_list_match_public_api_edges() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_545_regidx_parse_reg),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut line = CString::new("chrX:2-4").unwrap().into_bytes_with_nul();
            assert_eq!(regidx_c_198_regidx_insert(idx, line.as_mut_ptr().cast()), 0);

            let mut list = CString::new("chrX:8-9,chrY:1")
                .unwrap()
                .into_bytes_with_nul();
            assert_eq!(
                regidx_c_111_regidx_insert_list(idx, list.as_mut_ptr().cast(), b',' as c_char),
                0
            );

            let mut chr = CString::new("chrZ").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            assert_eq!(
                regidx_c_151_regidx_push(idx, chr_beg, chr_end, 0, 2, std::ptr::null_mut()),
                0
            );

            assert_eq!(regidx_c_98_regidx_nregs(idx), 4);
            regidx_c_311_regidx_destroy(idx);
        }
    }
}
