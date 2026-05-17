use std::ffi::{c_char, c_int};

// original: print_usage (htslib/samples/index_fasta.c:39)
pub unsafe fn samples_index_fasta_c_39_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: index_fasta <file>\nIndexes a fasta/fastq file and saves along with source.\n"
            .as_ptr(),
    );
}

// original: main (htslib/samples/index_fasta.c:51)
pub unsafe fn samples_index_fasta_c_51_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;

    if argc != 2 {
        samples_index_fasta_c_39_print_usage(hts_sys::stdout.cast());
        return ret;
    }
    let filename = *argv.add(1);

    if crate::htslib_mini_rs::faidx::fai_build3(filename, std::ptr::null(), std::ptr::null()) == -1
    {
        libc::printf(
            c"Indexing failed with %d\n".as_ptr(),
            *crate::htslib_mini_rs::c_compat::__errno_location(),
        );
    } else {
        ret = libc::EXIT_SUCCESS;
    }
    ret
}
