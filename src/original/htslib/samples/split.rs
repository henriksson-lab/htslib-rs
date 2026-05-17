// original: print_usage (htslib/samples/split.c:37)
pub unsafe fn samples_split_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: split infile outdir\nSplits the input file alignments to read1 and read2 and saves as 1.sam and 2.bam in given directory\nShows the basic writing of output\n".as_ptr(),
    );
}

// original: main (htslib/samples/split.c:50)
pub unsafe fn samples_split_c_50_main() {
    todo!("translate HTSlib main from htslib/samples/split.c:50");
}
