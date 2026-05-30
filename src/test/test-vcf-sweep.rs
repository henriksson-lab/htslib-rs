use std::ffi::{c_char, c_int, c_void};

pub unsafe fn test_test_vcf_sweep_c_31_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    if argc != 2 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Usage: test-vcf-sweep <file.bcf|file.vcf>\n".as_ptr(),
        );
        return 1;
    }

    let sw = crate::htslib_rs::vcf::bcf_sweep_init(*argv.add(1));
    let hdr = crate::htslib_rs::vcf::bcf_sweep_hdr(sw);
    let mut chksum: c_int = 0;

    loop {
        let rec = crate::htslib_rs::vcf::bcf_sweep_fwd(sw);
        if rec.is_null() {
            break;
        }
        chksum += ((*rec).pos + 1) as c_int;
    }
    libc::printf(c"fwd position chksum: %d\n".as_ptr(), chksum);

    chksum = 0;
    loop {
        let rec = crate::htslib_rs::vcf::bcf_sweep_bwd(sw);
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
        let rec = crate::htslib_rs::vcf::bcf_sweep_fwd(sw);
        if rec.is_null() {
            break;
        }

        n_pls = crate::htslib_rs::vcf::bcf_get_format_values(
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
        let rec = crate::htslib_rs::vcf::bcf_sweep_bwd(sw);
        if rec.is_null() {
            break;
        }

        n_pls = crate::htslib_rs::vcf::bcf_get_format_values(
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

    crate::htslib_rs::vcf::bcf_sweep_destroy(sw);
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::path::{Path, PathBuf};

    fn fixture(path: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
    }

    fn c_path(path: &Path) -> CString {
        CString::new(path.to_string_lossy().as_bytes()).unwrap()
    }

    fn temp_output(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "htslib-rs-test-vcf-sweep-main-{label}-{}-{}.out",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ))
    }

    fn temp_vcf(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "htslib-rs-test-vcf-sweep-main-{label}-{}-{}.vcf",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ))
    }

    unsafe fn run_main_capture_stdout(args: &mut [CString], out_path: &Path) -> c_int {
        let _ = std::fs::remove_file(out_path);
        libc::fflush(std::ptr::null_mut());

        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");

        if pid == 0 {
            let out_c = c_path(out_path);
            let out_fd = libc::open(
                out_c.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o600,
            );
            if out_fd < 0 {
                libc::_exit(libc::EXIT_FAILURE);
            }
            if libc::dup2(out_fd, libc::STDOUT_FILENO) < 0 {
                libc::close(out_fd);
                libc::_exit(libc::EXIT_FAILURE);
            }
            libc::close(out_fd);

            let mut argv = args
                .iter_mut()
                .map(|arg| arg.as_ptr().cast_mut())
                .collect::<Vec<_>>();
            let ret = test_test_vcf_sweep_c_31_main(argv.len() as c_int, argv.as_mut_ptr());
            libc::fflush(std::ptr::null_mut());
            libc::_exit(ret);
        }

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status), "child did not exit normally");
        libc::WEXITSTATUS(status)
    }

    #[test]
    fn original_test_vcf_sweep_main_matches_fixture_output() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let input = temp_vcf("input");
        std::fs::write(
            &input,
            concat!(
                "##fileformat=VCFv4.2\n",
                "##contig=<ID=20>\n",
                "##INFO=<ID=NS,Number=1,Type=Integer,Description=\"Number of Samples With Data\">\n",
                "##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Total Depth\">\n",
                "##INFO=<ID=AF,Number=A,Type=Float,Description=\"Allele Frequency\">\n",
                "##INFO=<ID=DB,Number=0,Type=Flag,Description=\"dbSNP membership\">\n",
                "##INFO=<ID=H2,Number=0,Type=Flag,Description=\"HapMap2 membership\">\n",
                "##INFO=<ID=AA,Number=1,Type=String,Description=\"Ancestral Allele\">\n",
                "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n",
                "##FORMAT=<ID=GQ,Number=1,Type=Integer,Description=\"Genotype Quality\">\n",
                "##FORMAT=<ID=DP,Number=1,Type=Integer,Description=\"Read Depth\">\n",
                "##FORMAT=<ID=HQ,Number=2,Type=Integer,Description=\"Haplotype Quality\">\n",
                "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tNA00001\tNA00002\tNA00003\n",
                "20\t14370\trs6054257\tG\tA\t29\tPASS\tNS=3;DP=14;AF=0.5;DB;H2\tGT:GQ:DP:HQ\t0|0:48:1:51,51\t1|0:48:8:51,51\t1/1:43:5:.,.\n",
                "20\t1110696\t.\tA\tG,T\t67\t.\tNS=2;DP=10;AF=0.333,.;AA=T;DB\tGT\t2\t1\t./.\n",
            ),
        )
        .unwrap();

        let out = temp_output("input");
        let mut args = [CString::new("test-vcf-sweep").unwrap(), c_path(&input)];

        unsafe {
            assert_eq!(run_main_capture_stdout(&mut args, &out), libc::EXIT_SUCCESS);
        }

        let actual = std::fs::read_to_string(&out).unwrap();
        let expected = std::fs::read_to_string(fixture("htslib/test/test-vcf-sweep.out")).unwrap();
        assert_eq!(actual, expected);
        let _ = std::fs::remove_file(out);
        let _ = std::fs::remove_file(input);
    }
}
