// original: print_usage (htslib/samples/update_header.c:37)
pub unsafe fn samples_update_header_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: update_header infile header idval tag value\nUpdates the tag's value on line given in id on header of given type\n".as_ptr(),
    );
}

// original: main (htslib/samples/update_header.c:49)
pub unsafe fn samples_update_header_c_49_main() {
    todo!("translate HTSlib main from htslib/samples/update_header.c:49");
}
