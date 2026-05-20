use std::ffi::{c_char, c_int};

use crate::htslib_mini_rs::{hts, sam};

// original: print_usage (htslib/samples/index_reg_read.c:37)
pub unsafe fn samples_index_reg_read_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_reg infile idxfile region\nReads alignments matching to a specific region\n\\. from start of file\n\\* only unmapped reads\nREFNAME all reads referring REFNAME\nREFNAME:S all reads referring REFNAME and overlapping from S onwards\nREFNAME:S-E all reads referring REFNAME overlapping from S to E\nREFNAME:-E all reads referring REFNAME overlapping upto E\n".as_ptr(),
    );
}

// original: main (htslib/samples/index_reg_read.c:55)
pub unsafe fn samples_index_reg_read_c_55_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut outfile = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut idx = std::ptr::null_mut();
    let mut iter = std::ptr::null_mut();

    if argc != 4 {
        samples_index_reg_read_c_37_print_usage(hts_sys::stderr.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let idxfile = *argv.add(2);
    let region = *argv.add(3);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to initialize bamdata\n".as_ptr());
        return ret;
    }

    let infile = hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        libc::printf(c"Could not open input file\n".as_ptr());
    } else {
        outfile = hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
        if outfile.is_null() {
            libc::printf(c"Could not open out file\n".as_ptr());
        } else {
            idx = sam::sam_index_load2(infile, inname, idxfile);
            if idx.is_null() {
                libc::printf(c"Failed to load the index\n".as_ptr());
            } else {
                in_samhdr = sam::sam_hdr_read(infile);
                if in_samhdr.is_null() {
                    libc::printf(c"Failed to read header from file!\n".as_ptr());
                } else {
                    iter = sam::sam_itr_querys(idx, in_samhdr, region);
                    if iter.is_null() {
                        libc::printf(c"Failed to get iterator\n".as_ptr());
                    } else {
                        let mut c = sam::sam_itr_next(infile, iter, bamdata);
                        while c >= 0 {
                            if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                                libc::printf(c"Failed to write output\n".as_ptr());
                                break;
                            }
                            c = sam::sam_itr_next(infile, iter, bamdata);
                        }
                        if c == -1 {
                            ret = libc::EXIT_SUCCESS;
                        } else {
                            libc::printf(c"Error during read\n".as_ptr());
                        }
                    }
                }
            }
        }
    }

    if !in_samhdr.is_null() {
        sam::sam_hdr_destroy(in_samhdr);
    }
    if !infile.is_null() {
        hts::hts_close(infile);
    }
    if !outfile.is_null() {
        hts::hts_close(outfile);
    }
    sam::bam_destroy1(bamdata);
    if !iter.is_null() {
        hts::hts_itr_destroy(iter);
    }
    if !idx.is_null() {
        hts::hts_idx_destroy(idx);
    }
    ret
}
