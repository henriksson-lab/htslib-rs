use std::ffi::{c_char, c_int, c_void};

pub unsafe fn test_test_vcf_sweep_c_31_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    if argc != 2 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Usage: test-vcf-sweep <file.bcf|file.vcf>\n".as_ptr(),
        );
        return 1;
    }

    let sw = crate::htslib_mini_rs::vcf::bcf_sweep_init(*argv.add(1));
    let hdr = crate::htslib_mini_rs::vcf::bcf_sweep_hdr(sw);
    let mut chksum: c_int = 0;

    loop {
        let rec = crate::htslib_mini_rs::vcf::bcf_sweep_fwd(sw);
        if rec.is_null() {
            break;
        }
        chksum += ((*rec).pos + 1) as c_int;
    }
    libc::printf(c"fwd position chksum: %d\n".as_ptr(), chksum);

    chksum = 0;
    loop {
        let rec = crate::htslib_mini_rs::vcf::bcf_sweep_bwd(sw);
        if rec.is_null() {
            break;
        }
        chksum += ((*rec).pos + 1) as c_int;
    }
    libc::printf(c"bwd position chksum: %d\n".as_ptr(), chksum);

    let mut m_pls: c_int = 0;
    let mut n_pls: c_int;
    let mut pls: *mut i32 = std::ptr::null_mut();
    chksum = 0;
    loop {
        let rec = crate::htslib_mini_rs::vcf::bcf_sweep_fwd(sw);
        if rec.is_null() {
            break;
        }

        n_pls = crate::htslib_mini_rs::vcf::bcf_get_format_values(
            hdr,
            rec,
            c"PL".as_ptr(),
            (&mut pls as *mut *mut i32).cast::<*mut c_void>(),
            &mut m_pls,
            hts_sys::BCF_HT_INT as c_int,
        );
        if n_pls <= 0 {
            continue;
        }

        let nsamples = (*hdr).n[hts_sys::BCF_DT_SAMPLE as usize];
        let nvals = n_pls / nsamples;
        let mut ptr = pls;
        for _ in 0..nsamples {
            for j in 0..nvals {
                let val = *ptr.add(j as usize);
                if val == hts_sys::bcf_int32_vector_end {
                    break;
                }
                if val == hts_sys::bcf_int32_missing {
                    continue;
                }
                chksum += val;
            }
            ptr = ptr.add(nvals as usize);
        }
    }
    libc::printf(c"fwd PL chksum: %d\n".as_ptr(), chksum);

    chksum = 0;
    loop {
        let rec = crate::htslib_mini_rs::vcf::bcf_sweep_bwd(sw);
        if rec.is_null() {
            break;
        }

        n_pls = crate::htslib_mini_rs::vcf::bcf_get_format_values(
            hdr,
            rec,
            c"PL".as_ptr(),
            (&mut pls as *mut *mut i32).cast::<*mut c_void>(),
            &mut m_pls,
            hts_sys::BCF_HT_INT as c_int,
        );
        if n_pls <= 0 {
            continue;
        }

        let nsamples = (*hdr).n[hts_sys::BCF_DT_SAMPLE as usize];
        let nvals = n_pls / nsamples;
        let mut ptr = pls;
        for _ in 0..nsamples {
            for j in 0..nvals {
                let val = *ptr.add(j as usize);
                if val == hts_sys::bcf_int32_vector_end {
                    break;
                }
                if val == hts_sys::bcf_int32_missing {
                    continue;
                }
                chksum += val;
            }
            ptr = ptr.add(nvals as usize);
        }
    }
    libc::printf(c"bwd PL chksum: %d\n".as_ptr(), chksum);

    crate::htslib_mini_rs::vcf::bcf_sweep_destroy(sw);
    0
}
