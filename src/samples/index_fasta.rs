use std::io::Write;

// original: print_usage (htslib/samples/index_fasta.c:39)
pub unsafe fn samples_index_fasta_c_39_print_usage() {
    let mut __out = std::io::stdout();
    write!(__out, "Usage: index_fasta <file>\nIndexes a fasta/fastq file and saves along with source.\n").unwrap();
}

// original: main (htslib/samples/index_fasta.c:51)
pub unsafe fn samples_index_fasta_c_51_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut ret = 1;
    let mut __out = std::io::stdout();

    if argc != 2 {
        samples_index_fasta_c_39_print_usage();
        return ret;
    }
    let filename = *argv.add(1);

    if crate::htslib_rs::faidx::fai_build3(filename.cast(), std::ptr::null(), std::ptr::null()) == -1
    {
        writeln!(__out, "Indexing failed with {}", std::io::Error::last_os_error().raw_os_error().unwrap_or(0)).unwrap();
    } else {
        ret = 0;
    }
    __out.flush().unwrap();
    ret
}
