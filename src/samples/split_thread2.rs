use crate::htslib_rs::{hts, sam, thread_pool};
use std::io::Write;

// original: print_usage (htslib/samples/split_thread2.c:38)
pub unsafe fn samples_split_thread2_c_38_print_usage() {
    eprint!("Usage: split_t2 infile outdir\nSplits the input file alignments to read1 and read2 and saves as 1.sam and 2.bam in given directory\nShows the usage of thread pool\n");
}

// original: main (htslib/samples/split_thread2.c:51)
pub unsafe fn samples_split_thread2_c_51_main(args: &[&[u8]]) -> i32 {
    let mut __out = std::io::stdout();
    let mut ret = 1;
    let mut infile = std::ptr::null_mut();
    let mut outfile1 = std::ptr::null_mut();
    let mut outfile2 = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut bamdata = std::ptr::null_mut();
    let mut tpool = hts::htsThreadPool {
        pool: std::ptr::null_mut(),
        qsize: 0,
    };

    if args.len() != 3 {
        samples_split_thread2_c_38_print_usage();
        return ret;
    }
    let inname = args[1];
    // argv entries are NUL-terminated C strings; use bytes up to the first NUL.
    let outdir = &args[2][..args[2].iter().position(|&b| b == 0).unwrap_or(args[2].len())];

    // NUL-terminated names for raw C-ABI hts_open
    let mut inname_c = inname.to_vec();
    inname_c.push(0);

    let mut file1 = outdir.to_vec();
    file1.extend_from_slice(b"/1.sam\0");
    let mut file2 = outdir.to_vec();
    file2.extend_from_slice(b"/2.bam\0");

    bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
    } else {
        infile = hts::hts_open(inname_c.as_ptr().cast(), c"r".as_ptr());
        if infile.is_null() {
            write!(__out, "Could not open {}\n", String::from_utf8_lossy(inname)).unwrap();
        } else {
            outfile1 = hts::hts_open(file1.as_ptr().cast(), c"w".as_ptr());
            outfile2 = hts::hts_open(file2.as_ptr().cast(), c"wb".as_ptr());
            if outfile1.is_null() || outfile2.is_null() {
                write!(__out, "Could not open output file\n").unwrap();
            } else {
                tpool.pool = thread_pool::hts_tpool_init(4);
                if tpool.pool.is_null() {
                    write!(__out, "Failed to initialize the thread pool\n").unwrap();
                } else if hts::hts_set_thread_pool(infile, &mut tpool) < 0
                    || hts::hts_set_thread_pool(outfile1, &mut tpool) < 0
                    || hts::hts_set_thread_pool(outfile2, &mut tpool) < 0
                {
                    write!(__out, "Failed to set thread options\n").unwrap();
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
                                if sam::sam_c_4553_sam_write1(outfile1, in_samhdr, bamdata) < 0
                                {
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
    if !tpool.pool.is_null() {
        thread_pool::hts_tpool_destroy(tpool.pool);
    }
    __out.flush().unwrap();
    ret
}
