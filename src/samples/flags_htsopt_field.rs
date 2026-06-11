use crate::htslib_rs::{hts, sam};
use std::io::Write;

// original: print_usage (htslib/samples/flags_htsopt_field.c:37)
pub unsafe fn samples_flags_htsopt_field_c_37_print_usage() {
    let mut __out = std::io::stdout();
    write!(__out, "Usage: flags_field <infile>\nShows the count of read1 and read2 alignments\nThis shows reading selected fields from CRAM file\n").unwrap();
}

// original: main (htslib/samples/flags_htsopt_field.c:50)
pub unsafe fn samples_flags_htsopt_field_c_50_main(args: &[&[u8]]) -> i32 {
    let mut ret: i32 = 1;
    let mut cntread1: i64 = 0;
    let mut cntread2: i64 = 0;
    let mut in_samhdr = std::ptr::null_mut();
    let mut __out = std::io::stdout();

    if args.len() != 2 {
        samples_flags_htsopt_field_c_37_print_usage();
        __out.flush().unwrap();
        return ret;
    }
    // argv entries are NUL-terminated C strings; use bytes up to the first NUL.
    let inname = &args[1][..args[1].iter().position(|&b| b == 0).unwrap_or(args[1].len())];

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
        __out.flush().unwrap();
        return ret;
    }
    // NUL-terminated copies for the still-raw C-ABI callees
    let mut inname_c: Vec<u8> = inname.to_vec();
    inname_c.push(0);
    let infile = hts::hts_open(inname_c.as_ptr().cast(), c"r".as_ptr());
    if infile.is_null() {
        write!(__out, "Could not open {}\n", String::from_utf8_lossy(inname)).unwrap();
        sam::bam_destroy1(bamdata);
        __out.flush().unwrap();
        return ret;
    }
    if hts::hts_set_opt_int(
        infile,
        hts::CRAM_OPT_REQUIRED_FIELDS,
        crate::htslib_rs::cram::decode_pipeline::SAM_FLAG,
    ) < 0
    {
        write!(__out, "Failed to set htsoption\n").unwrap();
    } else {
        in_samhdr = sam::sam_hdr_read(infile);
        if in_samhdr.is_null() {
            write!(__out, "Failed to read header from file\n").unwrap();
        } else {
            let mut c = sam::sam_read1(infile, in_samhdr, bamdata);
            while c >= 0 {
                if ((*bamdata).core.flag as i32 & sam::BAM_FREAD1) != 0 {
                    cntread1 += 1;
                }
                if ((*bamdata).core.flag as i32 & sam::BAM_FREAD2) != 0 {
                    cntread2 += 1;
                }
                c = sam::sam_read1(infile, in_samhdr, bamdata);
            }
            if c != -1 {
                write!(__out, "Failed to get data\n").unwrap();
            } else {
                write!(
                    __out,
                    "File {} has {} read1 and {} read2 alignments\n",
                    String::from_utf8_lossy(inname),
                    cntread1,
                    cntread2,
                ).unwrap();
                ret = 0;
            }
        }
    }

    if !in_samhdr.is_null() {
        sam::sam_hdr_destroy(in_samhdr);
    }
    hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    __out.flush().unwrap();
    ret
}
