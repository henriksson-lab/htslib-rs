use std::ffi::{c_char, c_int};

use crate::htslib_rs::sam;

// original: print_usage (htslib/samples/flags_demo.c:37)
pub unsafe fn samples_flags_demo_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: flags <infile>\nShows the count of read1 and read2 alignments\nThis shows basic reading and alignment flag access\n".as_ptr(),
    );
}

// original: main (htslib/samples/flags_demo.c:50)
pub unsafe fn samples_flags_demo_c_50_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut cntread1: i64 = 0;
    let mut cntread2: i64 = 0;

    if argc != 2 {
        samples_flags_demo_c_37_print_usage(crate::htslib_rs::c_compat::stdout.cast());
        return ret;
    }
    let inname = *argv.add(1);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to initialize bamdata\n".as_ptr());
        return ret;
    }
    let infile = crate::htslib_rs::hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), inname);
        sam::bam_destroy1(bamdata);
        return ret;
    }
    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let mut c = sam::sam_read1(infile, in_samhdr, bamdata);
    while c >= 0 {
        if ((*bamdata).core.flag as c_int & sam::BAM_FREAD1) != 0 {
            cntread1 += 1;
        }
        if ((*bamdata).core.flag as c_int & sam::BAM_FREAD2) != 0 {
            cntread2 += 1;
        }
        c = sam::sam_read1(infile, in_samhdr, bamdata);
    }
    if c != -1 {
        libc::printf(c"Failed to get data\n".as_ptr());
    } else {
        libc::printf(
            c"File %s has %ld read1 and %ld read2 alignments\n".as_ptr(),
            inname,
            cntread1 as libc::c_long,
            cntread2 as libc::c_long,
        );
        ret = libc::EXIT_SUCCESS;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    ret
}
