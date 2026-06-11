use crate::htslib_rs::sam;
use std::io::Write;

// original: print_usage (htslib/samples/mod_aux_ba.c:37)
pub unsafe fn samples_mod_aux_ba_c_37_print_usage(fp: *mut libc::FILE) {
    let _ = fp;
    eprint!("Usage: mod_aux_ba infile\nUpdates the count of bases as an aux array on all alignments\nBA:B:I,count of ACTGN\n");
}

// original: main (htslib/samples/mod_aux_ba.c:49)
pub unsafe fn samples_mod_aux_ba_c_49_main(args: &[Vec<u8>]) -> i32 {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut __out = std::io::stdout();
    let mut ret = 1;

    if args.len() != 2 {
        samples_mod_aux_ba_c_37_print_usage(std::ptr::null_mut());
        return ret;
    }
    // callee still expects a raw C-ABI *const c_char; build a NUL-terminated buffer
    let mut inname = args[1].clone();
    inname.push(0);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to allocate data memory!\n").unwrap();
        __out.flush().unwrap();
        return ret;
    }

    let infile = crate::htslib_rs::hts::hts_open(inname.as_ptr().cast(), c"r".as_ptr());
    if infile.is_null() {
        write!(__out, "Could not open {}\n", String::from_utf8_lossy(&args[1])).unwrap();
        sam::bam_destroy1(bamdata);
        __out.flush().unwrap();
        return ret;
    }
    let outfile = crate::htslib_rs::hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
    if outfile.is_null() {
        write!(__out, "Could not open std output\n").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        __out.flush().unwrap();
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        write!(__out, "Failed to read header from file!\n").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        sam::bam_destroy1(bamdata);
        __out.flush().unwrap();
        return ret;
    }

    if sam::sam_hdr_write(outfile, in_samhdr) == -1 {
        write!(__out, "Failed to write header\n").unwrap();
        sam::sam_hdr_destroy(in_samhdr);
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        sam::bam_destroy1(bamdata);
        __out.flush().unwrap();
        return ret;
    }

    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        *libc::__errno_location() = 0;
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
            c"BA".as_ptr().cast(),
            b'I',
            cnt.len() as u32,
            cnt.as_mut_ptr().cast(),
        ) != 0
        {
            write!(
                __out,
                "Failed to update base array, errno {}",
                *libc::__errno_location(),
            )
            .unwrap();
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            crate::htslib_rs::hts::hts_close(outfile);
            sam::bam_destroy1(bamdata);
            __out.flush().unwrap();
            return ret;
        }

        if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
            write!(__out, "Failed to write output\n").unwrap();
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            crate::htslib_rs::hts::hts_close(outfile);
            sam::bam_destroy1(bamdata);
            __out.flush().unwrap();
            return ret;
        }
        ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    }

    if ret_r < -1 {
        write!(__out, "Failed to read data\n").unwrap();
    } else {
        ret = 0;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::hts_close(outfile);
    sam::bam_destroy1(bamdata);
    __out.flush().unwrap();
    ret
}
