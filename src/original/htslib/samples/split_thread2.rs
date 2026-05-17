// original: print_usage (htslib/samples/split_thread2.c:38)
pub unsafe fn samples_split_thread2_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: split_t2 infile outdir\nSplits the input file alignments to read1 and read2 and saves as 1.sam and 2.bam in given directory\nShows the usage of thread pool\n".as_ptr(),
    );
}

// original: main (htslib/samples/split_thread2.c:51)
pub unsafe fn samples_split_thread2_c_51_main() {
    todo!("translate HTSlib main from htslib/samples/split_thread2.c:51");
}
