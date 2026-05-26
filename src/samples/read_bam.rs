use std::ffi::{c_char, c_int};

use crate::htslib_rs::{hts::hts_pos_t, sam};

// original: print_usage (htslib/samples/read_bam.c:37)
pub unsafe fn samples_read_bam_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_bam infile\nShows the alignment data from file\n".as_ptr(),
    );
}

// original: main (htslib/samples/read_bam.c:48)
pub unsafe fn samples_read_bam_c_48_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    const BAM_CIGAR_STR: &[u8; 10] = b"MIDNSHP=XB";
    let mut ret = libc::EXIT_FAILURE;

    if argc != 2 {
        samples_read_bam_c_37_print_usage(crate::htslib_rs::c_compat::stderr.cast());
        return ret;
    }
    let inname = *argv.add(1);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to allocate data memory!\n".as_ptr());
        return ret;
    }
    let infile = crate::htslib_rs::hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), inname);
        sam::bam_destroy1(bamdata);
        return ret;
    }
    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file!\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        libc::printf(c"NAME: %s\n".as_ptr(), sam::bam_get_qname(bamdata));
        let flags = sam::bam_flag2str((*bamdata).core.flag as c_int);
        libc::printf(
            c"FLG: %d - %s\n".as_ptr(),
            (*bamdata).core.flag as c_int,
            flags,
        );
        libc::free(flags.cast());

        let tidname = sam::sam_hdr_tid2name(in_samhdr, (*bamdata).core.tid);
        libc::printf(
            c"RNAME/TID: %d - %s\n".as_ptr(),
            (*bamdata).core.tid,
            if tidname.is_null() {
                c"".as_ptr()
            } else {
                tidname
            },
        );
        libc::printf(
            c"POS: %ld\n".as_ptr(),
            (*bamdata).core.pos as libc::c_long + 1,
        );
        libc::printf(c"MQUAL: %d\n".as_ptr(), (*bamdata).core.qual as c_int);

        let cigar = sam::bam_get_cigar(bamdata);
        libc::printf(c"CGR: ".as_ptr());
        for i in 0..(*bamdata).core.n_cigar {
            let cig = *cigar.add(i as usize);
            libc::printf(
                c"%d%c".as_ptr(),
                sam::bam_cigar_oplen(cig) as c_int,
                BAM_CIGAR_STR[sam::bam_cigar_op(cig) as usize] as c_int,
            );
        }
        libc::printf(
            c"\nTLEN/ISIZE: %ld\n".as_ptr(),
            (*bamdata).core.isize as libc::c_long,
        );

        let data = sam::bam_get_seq(bamdata);
        if (*bamdata).core.l_qseq as hts_pos_t
            != sam::bam_cigar2qlen((*bamdata).core.n_cigar as c_int, cigar)
        {
            libc::printf(c"\nLength doesnt matches to cigar data\n".as_ptr());
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            sam::bam_destroy1(bamdata);
            return ret;
        }

        libc::printf(c"SEQ: ".as_ptr());
        for i in 0..(*bamdata).core.l_qseq {
            libc::printf(
                c"%c".as_ptr(),
                SEQ_NT16_STR[sam::bam_seqi(data, i as usize) as usize] as c_int,
            );
        }
        libc::printf(c"\nQUAL: ".as_ptr());
        let qual = sam::bam_get_qual(bamdata);
        for i in 0..(*bamdata).core.l_qseq {
            libc::printf(c"%c".as_ptr(), *qual.add(i as usize) as c_int + 33);
        }
        libc::printf(c"\n\n".as_ptr());

        ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    }

    if ret_r == -1 {
        ret = libc::EXIT_SUCCESS;
    } else {
        libc::printf(c"Failed to read data\n".as_ptr());
    }
    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    ret
}
