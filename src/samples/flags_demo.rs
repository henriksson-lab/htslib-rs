use crate::htslib_rs::sam;
use std::io::Write;

// original: print_usage (htslib/samples/flags_demo.c:37)
pub unsafe fn samples_flags_demo_c_37_print_usage() {
    eprintln!(
        "Usage: flags <infile>\nShows the count of read1 and read2 alignments\nThis shows basic reading and alignment flag access"
    );
}

// original: main (htslib/samples/flags_demo.c:50)
pub unsafe fn samples_flags_demo_c_50_main(args: &[Vec<u8>]) -> i32 {
    let mut ret: i32 = 1;
    let mut cntread1: i64 = 0;
    let mut cntread2: i64 = 0;
    let mut __out = std::io::stdout();

    if args.len() != 2 {
        samples_flags_demo_c_37_print_usage();
        return ret;
    }
    let inname = &args[1];

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        writeln!(__out, "Failed to initialize bamdata").unwrap();
        __out.flush().unwrap();
        return ret;
    }
    let infile = crate::htslib_rs::hts::hts_open(inname.as_ptr().cast(), c"r".as_ptr());
    if infile.is_null() {
        writeln!(__out, "Could not open {}", String::from_utf8_lossy(inname)).unwrap();
        sam::bam_destroy1(bamdata);
        __out.flush().unwrap();
        return ret;
    }
    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        writeln!(__out, "Failed to read header from file").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        __out.flush().unwrap();
        return ret;
    }

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
        writeln!(__out, "Failed to get data").unwrap();
    } else {
        writeln!(
            __out,
            "File {} has {} read1 and {} read2 alignments",
            String::from_utf8_lossy(inname),
            cntread1,
            cntread2,
        ).unwrap();
        ret = 0;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    __out.flush().unwrap();
    ret
}
