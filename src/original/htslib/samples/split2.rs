// original: print_usage (htslib/samples/split2.c:37)
pub unsafe fn samples_split2_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: split2 infile outdir\nSplits the input file alignments to read1 and read2 and saves as 1.sam and 2.bam in given directory\nShows file type selection through name and format api\n".as_ptr(),
    );
}

// original: main (htslib/samples/split2.c:50)
pub unsafe fn samples_split2_c_50_main() {
    todo!("translate HTSlib main from htslib/samples/split2.c:50");
}
