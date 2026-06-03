use std::ffi::{c_char, c_int};

use crate::htslib_rs::{hts, sam};

// original: print_usage (htslib/samples/mod_bam.c:38)
pub unsafe fn samples_mod_bam_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: mod_bam infile QNAME fieldpos newval\nModifies the alignment data field\nfieldpos - 1 QNAME 2 FLAG 3 RNAME 4 POS 5 MAPQ 6 CIGAR 7 RNEXT 8 PNEXT 9 TLEN 10 SEQ 11 QUAL\n".as_ptr(),
    );
}

// original: main (htslib/samples/mod_bam.c:50)
pub unsafe fn samples_mod_bam_c_50_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut in_samhdr = std::ptr::null_mut();
    let mut infile = std::ptr::null_mut();
    let mut outfile = std::ptr::null_mut();

    if argc != 5 {
        samples_mod_bam_c_38_print_usage(crate::htslib_rs::c_compat::stderr.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let qname = *argv.add(2);
    let field = libc::atoi(*argv.add(3));
    let val = *argv.add(4);

    let mut bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to allocate data memory!\n".as_ptr());
    } else {
        infile = hts::hts_open(inname, c"r".as_ptr());
        outfile = hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
        if infile.is_null() || outfile.is_null() {
            libc::printf(c"Could not open input/output\n".as_ptr());
        } else {
            in_samhdr = sam::sam_hdr_read(infile);
            if in_samhdr.is_null() {
                libc::printf(c"Failed to read header from file!\n".as_ptr());
            } else if sam::sam_hdr_write(outfile, in_samhdr) == -1 {
                libc::printf(c"Failed to write header\n".as_ptr());
            } else {
                let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
                while ret_r >= 0 {
                    ret = 0;
                    if libc::strcasecmp(qname, sam::bam_get_qname(bamdata)) == 0 {
                        match field {
                            1 => {
                                ret = sam::bam_set_qname(bamdata, val);
                            }
                            2 => {
                                (*bamdata).core.flag = (libc::atol(val) & 0xffff) as u16;
                            }
                            3 | 7 => {
                                ret = sam::sam_hdr_name2tid(in_samhdr, val);
                                if ret < 0 {
                                    libc::printf(c"Invalid reference name\n".as_ptr());
                                    ret = -1;
                                } else if field == 3 {
                                    (*bamdata).core.tid = ret;
                                } else {
                                    (*bamdata).core.mtid = ret;
                                }
                            }
                            4 => {
                                (*bamdata).core.pos = libc::atoll(val) as hts::hts_pos_t;
                            }
                            5 => {
                                (*bamdata).core.qual = (libc::atoi(val) & 0x0ff) as u8;
                            }
                            6 => {
                                let mut cigar: *mut u32 = std::ptr::null_mut();
                                let mut size = 0usize;
                                let ncigar = sam::sam_parse_cigar(
                                    val,
                                    std::ptr::null_mut(),
                                    &mut cigar,
                                    &mut size,
                                );
                                if ncigar < 0 {
                                    libc::printf(c"Failed to parse cigar\n".as_ptr());
                                    ret = -1;
                                } else {
                                    let newbam = sam::bam_init1();
                                    if newbam.is_null() {
                                        libc::printf(c"Failed to create new bam data\n".as_ptr());
                                        ret = -1;
                                    } else if sam::bam_set1(
                                        newbam,
                                        (*bamdata).core.l_qname as usize,
                                        sam::bam_get_qname(bamdata),
                                        (*bamdata).core.flag,
                                        (*bamdata).core.tid,
                                        (*bamdata).core.pos,
                                        (*bamdata).core.qual,
                                        ncigar as usize,
                                        cigar,
                                        (*bamdata).core.mtid,
                                        (*bamdata).core.mpos,
                                        (*bamdata).core.isize,
                                        (*bamdata).core.l_qseq as usize,
                                        sam::bam_get_seq(bamdata).cast(),
                                        sam::bam_get_qual(bamdata).cast(),
                                        sam::bam_get_l_aux(bamdata) as usize,
                                    ) < 0
                                    {
                                        libc::printf(c"Failed to set bamdata\n".as_ptr());
                                        sam::bam_destroy1(newbam);
                                        ret = -1;
                                    } else {
                                        libc::memcpy(
                                            sam::bam_get_seq(newbam).cast_mut().cast(),
                                            sam::bam_get_seq(bamdata).cast(),
                                            ((*bamdata).core.l_qseq as usize).div_ceil(2),
                                        );
                                        libc::memcpy(
                                            sam::bam_get_aux(newbam).cast_mut().cast(),
                                            sam::bam_get_aux(bamdata).cast(),
                                            sam::bam_get_l_aux(bamdata) as usize,
                                        );
                                        sam::bam_destroy1(bamdata);
                                        bamdata = newbam;
                                    }
                                }
                                if !cigar.is_null() {
                                    libc::free(cigar.cast());
                                }
                            }
                            8 => {
                                (*bamdata).core.mpos = libc::atoll(val) as hts::hts_pos_t;
                            }
                            9 => {
                                (*bamdata).core.isize = libc::atoll(val) as hts::hts_pos_t;
                            }
                            10 => {
                                let len = libc::strlen(val) as c_int;
                                if (*bamdata).core.l_qseq != len {
                                    libc::printf(c"SEQ length different\n".as_ptr());
                                    ret = -1;
                                } else {
                                    let seq = sam::bam_get_seq(bamdata).cast_mut();
                                    for i in 0..len as usize {
                                        sam::bam_set_seqi(
                                            seq,
                                            i,
                                            sam::SEQ_NT16_TABLE[*val.add(i) as u8 as usize],
                                        );
                                    }
                                }
                            }
                            11 => {
                                let len = libc::strlen(val) as c_int;
                                if len != (*bamdata).core.l_qseq {
                                    libc::printf(c"Qual length different than sequence\n".as_ptr());
                                    ret = -1;
                                } else {
                                    let qual = sam::bam_get_qual(bamdata).cast_mut();
                                    for i in 0..len as usize {
                                        *qual.add(i) = (*val.add(i) as u8).wrapping_sub(33);
                                    }
                                }
                            }
                            _ => {
                                libc::printf(c"Invalid input\n".as_ptr());
                                ret = libc::EXIT_FAILURE;
                                break;
                            }
                        }
                        if ret < 0 {
                            libc::printf(c"Failed to set new data\n".as_ptr());
                            ret = libc::EXIT_FAILURE;
                            break;
                        }
                    }
                    if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                        libc::printf(c"Failed to write bam data\n".as_ptr());
                        ret = libc::EXIT_FAILURE;
                        break;
                    }
                    ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
                }
                if ret_r == -1 || ret != libc::EXIT_FAILURE {
                    ret = libc::EXIT_SUCCESS;
                } else {
                    libc::printf(c"Failed to read data\n".as_ptr());
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
    if !outfile.is_null() {
        hts::hts_close(outfile);
    }
    if !bamdata.is_null() {
        sam::bam_destroy1(bamdata);
    }
    ret
}
