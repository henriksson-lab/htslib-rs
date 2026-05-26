use std::ffi::{c_char, c_int};

use crate::htslib_rs::sam;

// original: print_usage (htslib/samples/mod_aux_ba.c:37)
pub unsafe fn samples_mod_aux_ba_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: mod_aux_ba infile\nUpdates the count of bases as an aux array on all alignments\nBA:B:I,count of ACTGN\n".as_ptr(),
    );
}

// original: main (htslib/samples/mod_aux_ba.c:49)
pub unsafe fn samples_mod_aux_ba_c_49_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = libc::EXIT_FAILURE;

    if argc != 2 {
        samples_mod_aux_ba_c_37_print_usage(crate::htslib_rs::c_compat::stderr.cast());
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
    let outfile = crate::htslib_rs::hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
    if outfile.is_null() {
        libc::printf(c"Could not open std output\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file!\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    if sam::sam_hdr_write(outfile, in_samhdr) == -1 {
        libc::printf(c"Failed to write header\n".as_ptr());
        sam::sam_hdr_destroy(in_samhdr);
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        *crate::htslib_rs::c_compat::__errno_location() = 0;
        let mut cnt = [0_u32; 5];
        let seq = sam::bam_get_seq(bamdata);
        for i in 0..(*bamdata).core.l_qseq {
            match SEQ_NT16_STR[sam::bam_seqi(seq, i as usize) as usize] {
                b'A' => cnt[0] += 1,
                b'C' => cnt[1] += 1,
                b'G' => cnt[2] += 1,
                b'T' => cnt[3] += 1,
                _ => cnt[4] += 1,
            }
        }

        if sam::bam_aux_update_array(
            bamdata,
            c"BA".as_ptr(),
            b'I',
            cnt.len() as u32,
            cnt.as_mut_ptr().cast(),
        ) != 0
        {
            libc::printf(
                c"Failed to update base array, errno %d".as_ptr(),
                *crate::htslib_rs::c_compat::__errno_location(),
            );
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            crate::htslib_rs::hts::hts_close(outfile);
            sam::bam_destroy1(bamdata);
            return ret;
        }

        if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
            libc::printf(c"Failed to write output\n".as_ptr());
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            crate::htslib_rs::hts::hts_close(outfile);
            sam::bam_destroy1(bamdata);
            return ret;
        }
        ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    }

    if ret_r < -1 {
        libc::printf(c"Failed to read data\n".as_ptr());
    } else {
        ret = libc::EXIT_SUCCESS;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::hts_close(outfile);
    sam::bam_destroy1(bamdata);
    ret
}
