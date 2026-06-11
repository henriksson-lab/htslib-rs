use crate::htslib_rs::{hts, sam};
use std::io::Write;

// original: print_usage (htslib/samples/split.c:37)
pub unsafe fn samples_split_c_37_print_usage() {
    eprint!("Usage: split infile outdir\nSplits the input file alignments to read1 and read2 and saves as 1.sam and 2.bam in given directory\nShows the basic writing of output\n");
}

// original: main (htslib/samples/split.c:50)
pub unsafe fn samples_split_c_50_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut __out = std::io::stdout();
    let mut ret = 1;
    let mut infile = std::ptr::null_mut();
    let mut outfile1 = std::ptr::null_mut();
    let mut outfile2 = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut bamdata = std::ptr::null_mut();

    if argc != 3 {
        samples_split_c_37_print_usage();
        return ret;
    }
    let inname = *argv.add(1);
    // outdir as bytes (without trailing NUL) read from the raw C-string arg
    let outdir = {
        let p = *argv.add(2);
        let mut len = 0usize;
        while *p.add(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(p, len).to_vec()
    };

    // build NUL-terminated output paths
    let mut file1 = outdir.clone();
    file1.extend_from_slice(b"/1.sam\0");
    let mut file2 = outdir.clone();
    file2.extend_from_slice(b"/2.bam\0");

    bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
    } else {
        infile = hts::hts_open(inname.cast(), c"r".as_ptr());
        if infile.is_null() {
            write!(__out, "Could not open file\n").unwrap();
        } else {
            outfile1 = hts::hts_open(file1.as_ptr().cast(), c"w".as_ptr());
            outfile2 = hts::hts_open(file2.as_ptr().cast(), c"wb".as_ptr());
            if outfile1.is_null() || outfile2.is_null() {
                write!(__out, "Could not open output file\n").unwrap();
            } else {
                in_samhdr = sam::sam_hdr_read(infile);
                if in_samhdr.is_null() {
                    write!(__out, "Failed to read header from file!\n").unwrap();
                } else if sam::sam_hdr_write(outfile1, in_samhdr) == -1
                    || sam::sam_hdr_write(outfile2, in_samhdr) == -1
                {
                    write!(__out, "Failed to write header\n").unwrap();
                } else {
                    let mut c = sam::sam_read1(infile, in_samhdr, bamdata);
                    while c >= 0 {
                        if ((*bamdata).core.flag as i32 & sam::BAM_FREAD1) != 0 {
                            if sam::sam_c_4553_sam_write1(outfile1, in_samhdr, bamdata) < 0 {
                                write!(__out, "Failed to write output data\n").unwrap();
                                break;
                            }
                        } else if ((*bamdata).core.flag as i32 & sam::BAM_FREAD2) != 0
                            && sam::sam_c_4553_sam_write1(outfile2, in_samhdr, bamdata) < 0
                        {
                            write!(__out, "Failed to write output data\n").unwrap();
                            break;
                        }
                        c = sam::sam_read1(infile, in_samhdr, bamdata);
                    }
                    if c == -1 {
                        ret = 0;
                    } else {
                        write!(__out, "Error in reading data\n").unwrap();
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
    // file1 / file2 are owned Vec<u8>, dropped automatically
    if !outfile1.is_null() {
        hts::hts_close(outfile1);
    }
    if !outfile2.is_null() {
        hts::hts_close(outfile2);
    }
    __out.flush().unwrap();
    ret
}
