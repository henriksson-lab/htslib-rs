pub unsafe fn test_test_vcf_sweep_c_31_main(argc: i32, argv: *mut *mut u8) -> i32 {
    use std::io::Write;
    if argc != 2 {
        eprint!("Usage: test-vcf-sweep <file.bcf|file.vcf>\n");
        return 1;
    }
    let mut __out = std::io::stdout();

    let fname = std::ffi::CStr::from_ptr((*argv.add(1)).cast()).to_bytes();
    let mut sw = crate::htslib_rs::vcf::bcf_sweep_init(fname);
    let Some(sw_ref) = sw.as_mut() else {
        return 1;
    };
    let Some(hdr) = crate::htslib_rs::vcf::bcf_sweep_hdr(sw_ref.as_mut()) else {
        crate::htslib_rs::vcf::bcf_sweep_destroy(sw);
        return 1;
    };
    let hdr = hdr as *mut crate::htslib_rs::vcf::bcf_hdr_t;
    let mut chksum: i32 = 0;

    while let Some(rec) = crate::htslib_rs::vcf::bcf_sweep_fwd(sw.as_mut().unwrap().as_mut()) {
        chksum += (rec.pos + 1) as i32;
    }
    write!(__out, "fwd position chksum: {}\n", chksum).unwrap();

    chksum = 0;
    while let Some(rec) = crate::htslib_rs::vcf::bcf_sweep_bwd(sw.as_mut().unwrap().as_mut()) {
        chksum += (rec.pos + 1) as i32;
    }
    write!(__out, "bwd position chksum: {}\n", chksum).unwrap();

    let mut m_pls: i32 = 0;
    let mut n_pls: i32;
    let mut pls: *mut i32 = std::ptr::null_mut();
    chksum = 0;
    while let Some(rec) = crate::htslib_rs::vcf::bcf_sweep_fwd(sw.as_mut().unwrap().as_mut()) {
        n_pls = crate::htslib_rs::vcf::bcf_get_format_values(
            hdr,
            rec as *mut crate::htslib_rs::vcf::bcf1_t,
            b"PL\0".as_ptr().cast(),
            (&mut pls as *mut *mut i32).cast::<*mut std::os::raw::c_void>(),
            &mut m_pls,
            crate::htslib_rs::vcf::BCF_HT_INT as i32,
        );
        if n_pls <= 0 {
            continue;
        }

        let nsamples = (*hdr).n[crate::htslib_rs::vcf::BCF_DT_SAMPLE as usize];
        let nvals = n_pls / nsamples;
        let mut ptr = pls;
        for _ in 0..nsamples {
            for j in 0..nvals {
                let val = *ptr.add(j as usize);
                if val == crate::htslib_rs::vcf::bcf_int32_vector_end {
                    break;
                }
                if val == crate::htslib_rs::vcf::bcf_int32_missing {
                    continue;
                }
                chksum += val;
            }
            ptr = ptr.add(nvals as usize);
        }
    }
    write!(__out, "fwd PL chksum: {}\n", chksum).unwrap();

    chksum = 0;
    while let Some(rec) = crate::htslib_rs::vcf::bcf_sweep_bwd(sw.as_mut().unwrap().as_mut()) {
        n_pls = crate::htslib_rs::vcf::bcf_get_format_values(
            hdr,
            rec as *mut crate::htslib_rs::vcf::bcf1_t,
            b"PL\0".as_ptr().cast(),
            (&mut pls as *mut *mut i32).cast::<*mut std::os::raw::c_void>(),
            &mut m_pls,
            crate::htslib_rs::vcf::BCF_HT_INT as i32,
        );
        if n_pls <= 0 {
            continue;
        }

        let nsamples = (*hdr).n[crate::htslib_rs::vcf::BCF_DT_SAMPLE as usize];
        let nvals = n_pls / nsamples;
        let mut ptr = pls;
        for _ in 0..nsamples {
            for j in 0..nvals {
                let val = *ptr.add(j as usize);
                if val == crate::htslib_rs::vcf::bcf_int32_vector_end {
                    break;
                }
                if val == crate::htslib_rs::vcf::bcf_int32_missing {
                    continue;
                }
                chksum += val;
            }
            ptr = ptr.add(nvals as usize);
        }
    }
    write!(__out, "bwd PL chksum: {}\n", chksum).unwrap();

    crate::htslib_rs::vcf::bcf_sweep_destroy(sw);
    __out.flush().unwrap();
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn fixture(path: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
    }

    fn c_path(path: &Path) -> Vec<u8> {
        let mut v = path.to_string_lossy().as_bytes().to_vec();
        v.push(0);
        v
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

    unsafe fn run_main_capture_stdout(args: &mut [Vec<u8>], out_path: &Path) -> i32 {
        let _ = std::fs::remove_file(out_path);
        libc::fflush(std::ptr::null_mut());

        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");

        if pid == 0 {
            let out_c = c_path(out_path);
            let out_fd = libc::open(
                out_c.as_ptr().cast(),
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
                .map(|arg| arg.as_mut_ptr())
                .collect::<Vec<_>>();
            let ret = test_test_vcf_sweep_c_31_main(argv.len() as i32, argv.as_mut_ptr());
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
        let mut args = [b"test-vcf-sweep\0".to_vec(), c_path(&input)];

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
