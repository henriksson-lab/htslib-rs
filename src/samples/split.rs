use std::ffi::{c_char, c_int};

use crate::htslib_rs::{hts, sam};

// original: print_usage (htslib/samples/split.c:37)
pub unsafe fn samples_split_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: split infile outdir\nSplits the input file alignments to read1 and read2 and saves as 1.sam and 2.bam in given directory\nShows the basic writing of output\n".as_ptr(),
    );
}

// original: main (htslib/samples/split.c:50)
pub unsafe fn samples_split_c_50_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut infile = std::ptr::null_mut();
    let mut outfile1 = std::ptr::null_mut();
    let mut outfile2 = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut bamdata = std::ptr::null_mut();

    if argc != 3 {
        samples_split_c_37_print_usage(crate::htslib_rs::c_compat::stdout.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let outdir = *argv.add(2);

    let size = libc::strlen(outdir) + c"/1.sam".to_bytes_with_nul().len();
    let file1 = libc::malloc(size).cast::<c_char>();
    let file2 = libc::malloc(size).cast::<c_char>();
    if file1.is_null() || file2.is_null() {
        libc::printf(c"Failed to set output path\n".as_ptr());
    } else {
        libc::snprintf(file1, size, c"%s/1.sam".as_ptr(), outdir);
        libc::snprintf(file2, size, c"%s/2.bam".as_ptr(), outdir);
        bamdata = sam::bam_init1();
        if bamdata.is_null() {
            libc::printf(c"Failed to initialize bamdata\n".as_ptr());
        } else {
            infile = hts::hts_open(inname, c"r".as_ptr());
            if infile.is_null() {
                libc::printf(c"Could not open %s\n".as_ptr(), inname);
            } else {
                outfile1 = hts::hts_open(file1, c"w".as_ptr());
                outfile2 = hts::hts_open(file2, c"wb".as_ptr());
                if outfile1.is_null() || outfile2.is_null() {
                    libc::printf(c"Could not open output file\n".as_ptr());
                } else {
                    in_samhdr = sam::sam_hdr_read(infile);
                    if in_samhdr.is_null() {
                        libc::printf(c"Failed to read header from file!\n".as_ptr());
                    } else if sam::sam_hdr_write(outfile1, in_samhdr) == -1
                        || sam::sam_hdr_write(outfile2, in_samhdr) == -1
                    {
                        libc::printf(c"Failed to write header\n".as_ptr());
                    } else {
                        let mut c = sam::sam_read1(infile, in_samhdr, bamdata);
                        while c >= 0 {
                            if ((*bamdata).core.flag as c_int & sam::BAM_FREAD1) != 0 {
                                if sam::sam_c_4553_sam_write1(outfile1, in_samhdr, bamdata) < 0 {
                                    libc::printf(c"Failed to write output data\n".as_ptr());
                                    break;
                                }
                            } else if ((*bamdata).core.flag as c_int & sam::BAM_FREAD2) != 0
                                && sam::sam_c_4553_sam_write1(outfile2, in_samhdr, bamdata) < 0
                            {
                                libc::printf(c"Failed to write output data\n".as_ptr());
                                break;
                            }
                            c = sam::sam_read1(infile, in_samhdr, bamdata);
                        }
                        if c == -1 {
                            ret = libc::EXIT_SUCCESS;
                        } else {
                            libc::printf(c"Error in reading data\n".as_ptr());
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
    if !bamdata.is_null() {
        sam::bam_destroy1(bamdata);
    }
    if !file1.is_null() {
        libc::free(file1.cast());
    }
    if !file2.is_null() {
        libc::free(file2.cast());
    }
    if !outfile1.is_null() {
        hts::hts_close(outfile1);
    }
    if !outfile2.is_null() {
        hts::hts_close(outfile2);
    }
    ret
}
