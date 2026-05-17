// original: print_usage (htslib/samples/index_multireg_read.c:37)
pub unsafe fn samples_index_multireg_read_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_multireg infile count regspec_csv\n    Reads alignment of a target matching to given region specifications\n    read_multireg infile.sam 2 R1:10-100,R2:200".as_ptr(),
    );
}

// original: main (htslib/samples/index_multireg_read.c:50)
pub unsafe fn samples_index_multireg_read_c_50_main() {
    todo!("translate HTSlib main from htslib/samples/index_multireg_read.c:50");
}
