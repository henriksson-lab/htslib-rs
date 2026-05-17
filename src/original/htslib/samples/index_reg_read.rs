// original: print_usage (htslib/samples/index_reg_read.c:37)
pub unsafe fn samples_index_reg_read_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_reg infile idxfile region\nReads alignments matching to a specific region\n\\. from start of file\n\\* only unmapped reads\nREFNAME all reads referring REFNAME\nREFNAME:S all reads referring REFNAME and overlapping from S onwards\nREFNAME:S-E all reads referring REFNAME overlapping from S to E\nREFNAME:-E all reads referring REFNAME overlapping upto E\n".as_ptr(),
    );
}

// original: main (htslib/samples/index_reg_read.c:55)
pub unsafe fn samples_index_reg_read_c_55_main() {
    todo!("translate HTSlib main from htslib/samples/index_reg_read.c:55");
}
