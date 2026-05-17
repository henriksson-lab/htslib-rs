// original: print_usage (htslib/samples/index_write.c:38)
pub unsafe fn samples_index_write_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: idx_on_write infile shiftsize outdir\nCreates compressed sam file and index file for it in given directory\n".as_ptr(),
    );
}

// original: main (htslib/samples/index_write.c:50)
pub unsafe fn samples_index_write_c_50_main() {
    todo!("translate HTSlib main from htslib/samples/index_write.c:50");
}
