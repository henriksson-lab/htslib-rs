use crate::htslib_rs::{hts, sam};
use std::io::Write;

// original: print_usage (htslib/samples/index_reg_read.c:37)
pub unsafe fn samples_index_reg_read_c_37_print_usage() {
    eprint!("Usage: read_reg infile idxfile region\nReads alignments matching to a specific region\n\\. from start of file\n\\* only unmapped reads\nREFNAME all reads referring REFNAME\nREFNAME:S all reads referring REFNAME and overlapping from S onwards\nREFNAME:S-E all reads referring REFNAME overlapping from S to E\nREFNAME:-E all reads referring REFNAME overlapping upto E\n");
}

// original: main (htslib/samples/index_reg_read.c:55)
pub unsafe fn samples_index_reg_read_c_55_main(args: &[&[u8]]) -> i32 {
    let mut __out = std::io::stdout();
    let mut ret = 1;
    let mut outfile = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut idx = std::ptr::null_mut();
    let mut iter = std::ptr::null_mut();

    if args.len() != 4 {
        samples_index_reg_read_c_37_print_usage();
        return ret;
    }
    // NUL-terminated copies for the still-raw C-ABI callees expecting *const c_char
    let inname: Vec<u8> = args[1].iter().copied().chain(std::iter::once(0)).collect();
    let idxfile: Vec<u8> = args[2].iter().copied().chain(std::iter::once(0)).collect();
    let region: Vec<u8> = args[3].iter().copied().chain(std::iter::once(0)).collect();

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
        __out.flush().unwrap();
        return ret;
    }

    let infile = hts::hts_open(inname.as_ptr().cast(), b"r\0".as_ptr().cast());
    if infile.is_null() {
        write!(__out, "Could not open input file\n").unwrap();
    } else {
        outfile = hts::hts_open(b"-\0".as_ptr().cast(), b"w\0".as_ptr().cast());
        if outfile.is_null() {
            write!(__out, "Could not open out file\n").unwrap();
        } else {
            idx = sam::sam_index_load2(infile, inname.as_ptr().cast(), idxfile.as_ptr().cast());
            if idx.is_null() {
                write!(__out, "Failed to load the index\n").unwrap();
            } else {
                in_samhdr = sam::sam_hdr_read(infile);
                if in_samhdr.is_null() {
                    write!(__out, "Failed to read header from file!\n").unwrap();
                } else {
                    iter = sam::sam_itr_querys(idx, in_samhdr, region.as_ptr().cast());
                    if iter.is_null() {
                        write!(__out, "Failed to get iterator\n").unwrap();
                    } else {
                        let mut c = sam::sam_itr_next(infile, iter, bamdata);
                        while c >= 0 {
                            if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                                write!(__out, "Failed to write output\n").unwrap();
                                break;
                            }
                            c = sam::sam_itr_next(infile, iter, bamdata);
                        }
                        if c == -1 {
                            ret = 0;
                        } else {
                            write!(__out, "Error during read\n").unwrap();
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
    __out.flush().unwrap();
    ret
}
