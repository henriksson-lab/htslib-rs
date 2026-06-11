use crate::htslib_rs::{hts, sam};
use std::io::Write;

// original: print_usage (htslib/samples/split2.c:37)
pub unsafe fn samples_split2_c_37_print_usage() {
    eprint!(
        "Usage: split2 infile outdir\nSplits the input file alignments to read1 and read2 and saves as 1.sam and 2.bam in given directory\nShows file type selection through name and format api\n"
    );
}

// original: main (htslib/samples/split2.c:50)
pub unsafe fn samples_split2_c_50_main(args: &[Vec<u8>]) -> i32 {
    let mut __out = std::io::stdout();
    let mut mode1 = [b'w', 0, 0, 0, 0];
    let mut mode2 = [b'w', 0, 0, 0, 0];
    let mut ret = 1;
    let mut infile = std::ptr::null_mut();
    let mut outfile1 = std::ptr::null_mut();
    let mut outfile2 = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut bamdata = std::ptr::null_mut();

    if args.len() != 3 {
        samples_split2_c_37_print_usage();
        return ret;
    }
    // NUL-terminated copies for raw-ptr C-ABI callees in other files
    let mut inname = args[1].clone();
    inname.push(0);
    // argv entries are NUL-terminated C strings; use bytes up to the first NUL.
    let outdir = &args[2][..args[2].iter().position(|&b| b == 0).unwrap_or(args[2].len())];

    // Build "<outdir>/1.sam.gz" and "<outdir>/2.sam" as NUL-terminated byte buffers
    let mut file1: Vec<u8> = outdir.to_vec();
    file1.extend_from_slice(b"/1.sam.gz\0");
    let mut file2: Vec<u8> = outdir.to_vec();
    file2.extend_from_slice(b"/2.sam\0");

    bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
    } else if sam::sam_open_mode(
        mode1.as_mut_ptr().add(1).cast(),
        file1.as_ptr().cast(),
        std::ptr::null(),
    ) == -1
        || sam::sam_open_mode(
            mode2.as_mut_ptr().add(1).cast(),
            file2.as_ptr().cast(),
            c"sam.gz".as_ptr().cast(),
        ) == -1
    {
        write!(__out, "Failed to set open mode\n").unwrap();
    } else {
        infile = hts::hts_open(inname.as_ptr().cast(), c"r".as_ptr());
        if infile.is_null() {
            write!(__out, "Could not open {}\n", String::from_utf8_lossy(&args[1])).unwrap();
        } else {
            outfile1 = hts::hts_open(file1.as_ptr().cast(), mode1.as_ptr().cast());
            outfile2 =
                hts::hts_open_format(file2.as_ptr().cast(), mode2.as_ptr().cast(), std::ptr::null());
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
    if !outfile1.is_null() {
        hts::hts_close(outfile1);
    }
    if !outfile2.is_null() {
        hts::hts_close(outfile2);
    }
    __out.flush().unwrap();
    ret
}
