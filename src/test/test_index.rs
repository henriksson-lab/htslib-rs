use crate::htslib_rs::{hts, sam, vcf};

unsafe extern "C" {
    static mut optarg: *mut u8;
    static mut optind: i32;
}

// original: usage (htslib/test/test_index.c:32)
pub unsafe fn test_test_index_c_32_usage(is_stderr: bool) -> ! {
    eprint!("Usage: test_index [opts] in.{{sam.gz,bam,cram}}|in.{{vcf.gz,bcf}}\n\n");
    eprint!("  -b       Use BAI index (BAM, SAM)\n");
    eprint!("  -c       Use CSI index (BAM, SAM, VCF, BCF)\n");
    eprint!("  -t       Use TBI index (VCF) \n");
    eprint!("  -m bits  Adjust min_shift; implies CSI\n");
    eprint!("\nThe default index format is CSI for sam/bam/vcf/bcf and CRAI for crams\n");
    libc::exit(is_stderr as i32);
}

// original: main (htslib/test/test_index.c:42)
pub unsafe fn test_test_index_c_42_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut min_shift = 14;

    loop {
        let c = libc::getopt(argc, argv.cast(), c"bctm:".as_ptr());
        if c < 0 {
            break;
        }
        match c {
            c if c == b't' as i32 || c == b'b' as i32 => min_shift = 0,
            c if c == b'c' as i32 => min_shift = 14,
            c if c == b'm' as i32 => min_shift = libc::atoi(optarg.cast()),
            c if c == b'h' as i32 => test_test_index_c_32_usage(false),
            _ => test_test_index_c_32_usage(true),
        }
    }

    if optind >= argc {
        test_test_index_c_32_usage(true);
    }

    let fname = *argv.add(optind as usize);
    let in_ = hts::hts_open(fname.cast(), c"r".as_ptr());
    if in_.is_null() {
        eprintln!("Error opening file");
        libc::exit(1);
    }

    let ret = match (*in_).format.format {
        x if x == crate::htslib_rs::hts::HTS_FORMAT_SAM
            || x == crate::htslib_rs::hts::HTS_FORMAT_BAM
            || x == crate::htslib_rs::hts::HTS_FORMAT_CRAM =>
        {
            sam::sam_index_build(fname.cast(), min_shift)
        }
        _ => vcf::bcf_index_build(fname.cast(), min_shift),
    };

    if ret < 0 {
        eprintln!("Failed to build index for file");
        libc::exit(1);
    }

    if hts::hts_close(in_) < 0 {
        eprintln!("Error closing file");
        libc::exit(1);
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::ptr;
    use std::sync::Mutex;

    static GETOPT_LOCK: Mutex<()> = Mutex::new(());

    fn fixture(path: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
    }

    fn temp_bam(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "htslib-rs-test-index-{label}-{}-{}.bam",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ))
    }

    fn c_path(path: &Path) -> Vec<u8> {
        let mut v = path.to_string_lossy().into_owned().into_bytes();
        v.push(0);
        v
    }

    unsafe fn run_main(args: &[Vec<u8>]) -> i32 {
        // NOTE: callers must already hold `ORIGINAL_MAIN_LOCK` (see
        // src/test/mod.rs). `GETOPT_LOCK` is retained for backward-compat
        // but is now effectively a no-op while the global lock is held.
        let _guard = GETOPT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        optarg = ptr::null_mut();
        // optind = 0 (not 1) forces glibc getopt to fully reinitialize its
        // internal scan state; these C-`main()` tests share one process, so a
        // prior getopt run would otherwise leave stale state ("invalid option").
        optind = 0;
        let mut argv = args
            .iter()
            .map(|arg| arg.as_ptr().cast_mut())
            .collect::<Vec<_>>();
        test_test_index_c_42_main(argv.len() as i32, argv.as_mut_ptr())
    }

    unsafe fn count_region_records(path: &Path, region: &[u8]) -> usize {
        let path_c = c_path(path);
        let fp = hts::hts_open(path_c.as_ptr().cast(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path.display());

        let hdr = sam::sam_hdr_read(fp);
        assert!(!hdr.is_null());
        let idx = sam::sam_index_load(fp, path_c.as_ptr().cast());
        assert!(
            !idx.is_null(),
            "failed to load index for {}",
            path.display()
        );
        let itr = sam::sam_itr_querys(idx, hdr, region.as_ptr().cast());
        assert!(!itr.is_null());
        let rec = sam::bam_init1();
        assert!(!rec.is_null());

        let mut count = 0usize;
        while sam::sam_itr_next(fp, itr, rec) >= 0 {
            count += 1;
        }

        sam::bam_destroy1(rec);
        hts::hts_itr_destroy(itr);
        hts::hts_idx_destroy(idx);
        sam::sam_hdr_destroy(hdr);
        assert_eq!(hts::hts_close(fp), 0);
        count
    }

    fn copy_range_bam_without_indexes(label: &str) -> PathBuf {
        let bam = temp_bam(label);
        let _ = std::fs::remove_file(&bam);
        let _ = std::fs::remove_file(format!("{}.bai", bam.display()));
        let _ = std::fs::remove_file(format!("{}.csi", bam.display()));
        std::fs::copy(fixture("htslib/test/range.bam"), &bam).unwrap();
        bam
    }

    #[test]
    fn original_test_index_main_builds_bam_bai_and_loads_region() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            let bam = copy_range_bam_without_indexes("bai");
            let bam_c = c_path(&bam);
            let args = [
                b"test_index\0".to_vec(),
                b"-b\0".to_vec(),
                bam_c,
            ];

            assert_eq!(run_main(&args), 0);

            assert!(PathBuf::from(format!("{}.bai", bam.display())).is_file());
            assert_eq!(count_region_records(&bam, b"CHROMOSOME_II:2976-3070\0"), 2);

            let _ = std::fs::remove_file(format!("{}.bai", bam.display()));
            let _ = std::fs::remove_file(bam);
        }
    }

    #[test]
    fn original_test_index_main_builds_bam_csi_with_min_shift_option() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            let bam = copy_range_bam_without_indexes("csi");
            let bam_c = c_path(&bam);
            let args = [
                b"test_index\0".to_vec(),
                b"-m\0".to_vec(),
                b"12\0".to_vec(),
                bam_c,
            ];

            assert_eq!(run_main(&args), 0);

            assert!(PathBuf::from(format!("{}.csi", bam.display())).is_file());
            assert_eq!(count_region_records(&bam, b"CHROMOSOME_IV:1422-1483\0"), 3);

            let _ = std::fs::remove_file(format!("{}.csi", bam.display()));
            let _ = std::fs::remove_file(bam);
        }
    }
}
