use std::ffi::CStr;
use std::io::Write;

use crate::htslib_rs::{hts::hts_pos_t, sam};

// original: print_usage (htslib/samples/read_bam.c:37)
pub unsafe fn samples_read_bam_c_37_print_usage() {
    eprint!("Usage: read_bam infile\nShows the alignment data from file\n");
}

// original: main (htslib/samples/read_bam.c:48)
pub unsafe fn samples_read_bam_c_48_main(args: &[Vec<u8>]) -> i32 {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    const BAM_CIGAR_STR: &[u8; 10] = b"MIDNSHP=XB";
    let mut ret = 1;
    let mut __out = std::io::stdout();

    if args.len() != 2 {
        samples_read_bam_c_37_print_usage();
        return ret;
    }
    let inname = &args[1];

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to allocate data memory!\n").unwrap();
        return ret;
    }
    // build NUL-terminated buffers for the still-raw C-ABI callee
    let mut inname_nul = inname.clone();
    inname_nul.push(0);
    let infile = crate::htslib_rs::hts::hts_open(inname_nul.as_ptr().cast(), b"r\0".as_ptr().cast());
    if infile.is_null() {
        write!(__out, "Could not open {}\n", String::from_utf8_lossy(inname)).unwrap();
        sam::bam_destroy1(bamdata);
        return ret;
    }
    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        write!(__out, "Failed to read header from file!\n").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        let qname = CStr::from_ptr(sam::bam_get_qname(bamdata).cast()).to_bytes();
        write!(__out, "NAME: {}\n", String::from_utf8_lossy(qname)).unwrap();
        let flags = sam::bam_flag2str((*bamdata).core.flag as i32);
        let flags_bytes = CStr::from_ptr(flags.cast()).to_bytes().to_vec();
        write!(
            __out,
            "FLG: {} - {}\n",
            (*bamdata).core.flag as i32,
            String::from_utf8_lossy(&flags_bytes),
        ).unwrap();
        drop(flags_bytes);

        let tidname = sam::sam_hdr_tid2name(&*in_samhdr, (*bamdata).core.tid);
        let tidname_bytes: &[u8] = if tidname.is_null() {
            b""
        } else {
            CStr::from_ptr(tidname.cast()).to_bytes()
        };
        write!(
            __out,
            "RNAME/TID: {} - {}\n",
            (*bamdata).core.tid,
            String::from_utf8_lossy(tidname_bytes),
        ).unwrap();
        write!(__out, "POS: {}\n", (*bamdata).core.pos as i64 + 1).unwrap();
        write!(__out, "MQUAL: {}\n", (*bamdata).core.qual as i32).unwrap();

        let cigar = sam::bam_get_cigar(bamdata);
        write!(__out, "CGR: ").unwrap();
        for i in 0..(*bamdata).core.n_cigar {
            let cig = *cigar.add(i as usize);
            write!(
                __out,
                "{}{}",
                sam::bam_cigar_oplen(cig) as i32,
                BAM_CIGAR_STR[sam::bam_cigar_op(cig) as usize] as char,
            ).unwrap();
        }
        write!(__out, "\nTLEN/ISIZE: {}\n", (*bamdata).core.isize as i64).unwrap();

        let data = sam::bam_get_seq(bamdata);
        if (*bamdata).core.l_qseq as hts_pos_t
            != sam::bam_cigar2qlen((*bamdata).core.n_cigar as i32, cigar)
        {
            write!(__out, "\nLength doesnt matches to cigar data\n").unwrap();
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            sam::bam_destroy1(bamdata);
            return ret;
        }

        write!(__out, "SEQ: ").unwrap();
        for i in 0..(*bamdata).core.l_qseq {
            write!(
                __out,
                "{}",
                SEQ_NT16_STR[sam::bam_seqi(data, i as usize) as usize] as char,
            ).unwrap();
        }
        write!(__out, "\nQUAL: ").unwrap();
        let qual = sam::bam_get_qual(bamdata);
        for i in 0..(*bamdata).core.l_qseq {
            write!(__out, "{}", (*qual.add(i as usize) as u8 + 33) as char).unwrap();
        }
        write!(__out, "\n\n").unwrap();

        ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    }

    if ret_r == -1 {
        ret = 0;
    } else {
        write!(__out, "Failed to read data\n").unwrap();
    }
    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    __out.flush().unwrap();
    ret
}
