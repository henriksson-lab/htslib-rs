use crate::htslib_rs::sam;
use std::io::Write;

// original: print_usage (htslib/samples/read_fast.c:37)
pub unsafe fn samples_read_fast_c_37_print_usage() {
    eprint!("Usage: read_fast <infile>\nReads the fasta/fastq file and shows the content.\n");
}

// original: main (htslib/samples/read_fast.c:49)
pub unsafe fn samples_read_fast_c_49_main(args: &[Vec<u8>]) -> i32 {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = 1;
    let mut __out = std::io::stdout();

    if args.len() != 2 {
        samples_read_fast_c_37_print_usage();
        return ret;
    }
    // NUL-terminate for the still-raw C-ABI hts_open boundary
    let inname: Vec<u8> = {
        let mut v = args[1].clone();
        v.push(0);
        v
    };

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
        __out.flush().unwrap();
        return ret;
    }
    let infile =
        crate::htslib_rs::hts::hts_open(inname.as_ptr().cast(), c"r".as_ptr());
    if infile.is_null() {
        write!(
            __out,
            "Could not open {}\n",
            String::from_utf8_lossy(&args[1])
        ).unwrap();
        __out.flush().unwrap();
        sam::bam_destroy1(bamdata);
        return ret;
    }
    if (*infile).format.format != crate::htslib_rs::hts::HTS_FORMAT_FASTA_FORMAT
        && (*infile).format.format != crate::htslib_rs::hts::HTS_FORMAT_FASTQ_FORMAT
    {
        write!(__out, "Invalid file specified\n").unwrap();
        __out.flush().unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        write!(__out, "Failed to read header from file\n").unwrap();
        __out.flush().unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let mut c = sam::sam_read1(infile, in_samhdr, bamdata);
    while c >= 0 {
        write!(__out, "\nname: ").unwrap();
        // bam_get_qname returns a raw C-ABI pointer to a NUL-terminated name
        let qname = sam::bam_get_qname(bamdata);
        let qname_bytes = std::ffi::CStr::from_ptr(qname.cast()).to_bytes();
        write!(__out, "{}", String::from_utf8_lossy(qname_bytes)).unwrap();
        write!(__out, "\nsequence: ").unwrap();
        for idx in 0..(*bamdata).core.l_qseq {
            let base = SEQ_NT16_STR
                [sam::bam_seqi(sam::bam_get_seq(bamdata), idx as usize) as usize];
            write!(__out, "{}", base as char).unwrap();
        }
        if (*infile).format.format == crate::htslib_rs::hts::HTS_FORMAT_FASTQ_FORMAT {
            write!(__out, "\nquality: ").unwrap();
            let qual = sam::bam_get_qual(bamdata);
            for idx in 0..(*bamdata).core.l_qseq {
                write!(__out, "{}", (*qual.add(idx as usize) as i32 + 33) as u8 as char).unwrap();
            }
        }
        c = sam::sam_read1(infile, in_samhdr, bamdata);
    }
    write!(__out, "\n").unwrap();
    if c != -1 {
        write!(__out, "Failed to get data\n").unwrap();
    } else {
        ret = 0;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    __out.flush().unwrap();
    ret
}
