// original: print_usage (htslib/samples/mod_bam.c:38)
pub unsafe fn samples_mod_bam_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: mod_bam infile QNAME fieldpos newval\nModifies the alignment data field\nfieldpos - 1 QNAME 2 FLAG 3 RNAME 4 POS 5 MAPQ 6 CIGAR 7 RNEXT 8 PNEXT 9 TLEN 10 SEQ 11 QUAL\n".as_ptr(),
    );
}

// original: main (htslib/samples/mod_bam.c:50)
pub unsafe fn samples_mod_bam_c_50_main() {
    todo!("translate HTSlib main from htslib/samples/mod_bam.c:50");
}
