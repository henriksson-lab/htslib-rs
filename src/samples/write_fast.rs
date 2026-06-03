use std::ffi::{c_char, c_int};

use crate::htslib_rs::{hts::htsFile, sam};

// original: print_usage (htslib/samples/write_fast.c:39)
pub unsafe fn samples_write_fast_c_39_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: write_fast <file> <sequence> [<qualities]\nAppends a fasta/fastq file.\n".as_ptr(),
    );
}

// original: main (htslib/samples/write_fast.c:51)
pub unsafe fn samples_write_fast_c_51_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut mode = [b'a' as c_char, 0, 0, 0];
    let mut qual: *const c_char = std::ptr::null();
    let mut name = [0 as c_char; 256];

    if !(3..=4).contains(&argc) {
        samples_write_fast_c_39_print_usage(crate::htslib_rs::c_compat::stdout.cast());
        return ret;
    }

    let outname = *argv.add(1);
    let data = *argv.add(2);
    if argc == 4 {
        qual = *argv.add(3);
        if libc::strlen(data) != libc::strlen(qual) {
            libc::printf(c"Incorrect reference and quality data\n".as_ptr());
            return ret;
        }
    }

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to initialize bamdata\n".as_ptr());
        return ret;
    }

    if sam::sam_open_mode(mode.as_mut_ptr().add(1), outname, std::ptr::null()) < 0 {
        libc::printf(c"Invalid file name\n".as_ptr());
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let hfile = crate::htslib_rs::hfile::hopen(outname, mode.as_ptr());
    if hfile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), outname);
        sam::bam_destroy1(bamdata);
        return ret;
    }
    let outfile: *mut htsFile = crate::htslib_rs::hts::hts_hopen(hfile, outname, mode.as_ptr());
    if outfile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), outname);
        crate::htslib_rs::hfile::hclose(hfile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    libc::snprintf(
        name.as_mut_ptr(),
        name.len(),
        c"Test_%ld".as_ptr(),
        libc::time(std::ptr::null_mut()) as libc::c_long,
    );
    if sam::bam_set1(
        bamdata,
        libc::strlen(name.as_ptr()),
        name.as_ptr(),
        sam::BAM_FUNMAP as u16,
        -1,
        -1,
        0,
        0,
        std::ptr::null(),
        -1,
        -1,
        0,
        libc::strlen(data),
        data,
        qual,
        0,
    ) < 0
    {
        libc::printf(c"Failed to set data\n".as_ptr());
    } else if sam::sam_c_4553_sam_write1(outfile, std::ptr::null(), bamdata) < 0 {
        libc::printf(c"Failed to write data\n".as_ptr());
    } else {
        ret = libc::EXIT_SUCCESS;
    }

    crate::htslib_rs::hts::hts_close(outfile);
    sam::bam_destroy1(bamdata);
    ret
}
