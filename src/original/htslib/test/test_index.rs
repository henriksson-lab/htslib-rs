use std::ffi::{c_char, c_int};

use crate::htslib_mini_rs::{hts, sam, vcf};

unsafe extern "C" {
    static mut optarg: *mut c_char;
    static mut optind: c_int;
}

// original: usage (htslib/test/test_index.c:32)
pub unsafe fn test_test_index_c_32_usage(fp: *mut libc::FILE) -> ! {
    libc::fprintf(
        fp,
        c"Usage: test_index [opts] in.{sam.gz,bam,cram}|in.{vcf.gz,bcf}\n\n".as_ptr(),
    );
    libc::fprintf(fp, c"  -b       Use BAI index (BAM, SAM)\n".as_ptr());
    libc::fprintf(
        fp,
        c"  -c       Use CSI index (BAM, SAM, VCF, BCF)\n".as_ptr(),
    );
    libc::fprintf(fp, c"  -t       Use TBI index (VCF) \n".as_ptr());
    libc::fprintf(fp, c"  -m bits  Adjust min_shift; implies CSI\n".as_ptr());
    libc::fprintf(
        fp,
        c"\nThe default index format is CSI for sam/bam/vcf/bcf and CRAI for crams\n".as_ptr(),
    );
    libc::exit((fp == hts_sys::stderr.cast::<libc::FILE>()) as c_int);
}

// original: main (htslib/test/test_index.c:42)
pub unsafe fn test_test_index_c_42_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut min_shift = 14;

    loop {
        let c = libc::getopt(argc, argv, c"bctm:".as_ptr());
        if c < 0 {
            break;
        }
        match c {
            c if c == b't' as c_int || c == b'b' as c_int => min_shift = 0,
            c if c == b'c' as c_int => min_shift = 14,
            c if c == b'm' as c_int => min_shift = libc::atoi(optarg),
            c if c == b'h' as c_int => test_test_index_c_32_usage(hts_sys::stdout.cast()),
            _ => test_test_index_c_32_usage(hts_sys::stderr.cast()),
        }
    }

    if optind >= argc {
        test_test_index_c_32_usage(hts_sys::stderr.cast());
    }

    let fname = *argv.add(optind as usize);
    let in_ = hts::hts_open(fname, c"r".as_ptr());
    if in_.is_null() {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Error opening \"%s\"\n".as_ptr(),
            fname,
        );
        libc::exit(1);
    }

    let ret = match (*in_).format.format {
        x if x == hts_sys::htsExactFormat_sam
            || x == hts_sys::htsExactFormat_bam
            || x == hts_sys::htsExactFormat_cram =>
        {
            sam::sam_index_build(fname, min_shift)
        }
        _ => vcf::bcf_index_build(fname, min_shift),
    };

    if ret < 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Failed to build index for \"%s\"\n".as_ptr(),
            fname,
        );
        libc::exit(1);
    }

    if hts::hts_close(in_) < 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Error closing \"%s\"\n".as_ptr(),
            fname,
        );
        libc::exit(1);
    }

    0
}
