use std::ffi::{c_char, c_int};

use crate::htslib_rs::sam;

// original: print_usage (htslib/samples/read_fast.c:37)
pub unsafe fn samples_read_fast_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_fast <infile>\nReads the fasta/fastq file and shows the content.\n".as_ptr(),
    );
}

// original: main (htslib/samples/read_fast.c:49)
pub unsafe fn samples_read_fast_c_49_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = libc::EXIT_FAILURE;

    if argc != 2 {
        samples_read_fast_c_37_print_usage(crate::htslib_rs::c_compat::stdout.cast());
        return ret;
    }
    let inname = *argv.add(1);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to initialize bamdata\n".as_ptr());
        return ret;
    }
    let infile = crate::htslib_rs::hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), inname);
        sam::bam_destroy1(bamdata);
        return ret;
    }
    if (*infile).format.format != crate::htslib_rs::hts::HTS_FORMAT_FASTA_FORMAT
        && (*infile).format.format != crate::htslib_rs::hts::HTS_FORMAT_FASTQ_FORMAT
    {
        libc::printf(c"Invalid file specified\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let mut c = sam::sam_read1(infile, in_samhdr, bamdata);
    while c >= 0 {
        libc::printf(c"\nname: ".as_ptr());
        libc::printf(c"%s".as_ptr(), sam::bam_get_qname(bamdata));
        libc::printf(c"\nsequence: ".as_ptr());
        for idx in 0..(*bamdata).core.l_qseq {
            libc::printf(
                c"%c".as_ptr(),
                SEQ_NT16_STR[sam::bam_seqi(sam::bam_get_seq(bamdata), idx as usize) as usize]
                    as c_int,
            );
        }
        if (*infile).format.format == crate::htslib_rs::hts::HTS_FORMAT_FASTQ_FORMAT {
            libc::printf(c"\nquality: ".as_ptr());
            let qual = sam::bam_get_qual(bamdata);
            for idx in 0..(*bamdata).core.l_qseq {
                libc::printf(c"%c".as_ptr(), *qual.add(idx as usize) as c_int + 33);
            }
        }
        c = sam::sam_read1(infile, in_samhdr, bamdata);
    }
    libc::printf(c"\n".as_ptr());
    if c != -1 {
        libc::printf(c"Failed to get data\n".as_ptr());
    } else {
        ret = libc::EXIT_SUCCESS;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    ret
}
