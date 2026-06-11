use crate::htslib_rs::{hts, sam};
use std::io::Write;

// original: print_usage (htslib/samples/mod_bam.c:38)
pub unsafe fn samples_mod_bam_c_38_print_usage() {
    eprint!(
        "Usage: mod_bam infile QNAME fieldpos newval\nModifies the alignment data field\nfieldpos - 1 QNAME 2 FLAG 3 RNAME 4 POS 5 MAPQ 6 CIGAR 7 RNEXT 8 PNEXT 9 TLEN 10 SEQ 11 QUAL\n"
    );
}

// original: main (htslib/samples/mod_bam.c:50)
pub unsafe fn samples_mod_bam_c_50_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut ret = 1;
    let mut __out = std::io::stdout();
    let mut in_samhdr = std::ptr::null_mut();
    let mut infile = std::ptr::null_mut();
    let mut outfile = std::ptr::null_mut();

    if argc != 5 {
        samples_mod_bam_c_38_print_usage();
        return ret;
    }
    // command-line args are NUL-terminated C strings; expose as byte slices,
    // and pass `.as_ptr().cast()` at raw C-ABI boundaries in other files.
    let inname = std::ffi::CStr::from_ptr((*argv.add(1)).cast()).to_bytes();
    let qname = std::ffi::CStr::from_ptr((*argv.add(2)).cast()).to_bytes();
    let field_bytes = std::ffi::CStr::from_ptr((*argv.add(3)).cast()).to_bytes();
    let field: i32 = std::str::from_utf8(field_bytes).unwrap_or("").trim().parse().unwrap_or(0);
    let val = std::ffi::CStr::from_ptr((*argv.add(4)).cast()).to_bytes();

    let mut bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to allocate data memory!\n").unwrap();
    } else {
        // NUL-terminated copies for raw C-ABI callees in other files.
        let inname_c: Vec<u8> = inname.iter().copied().chain(std::iter::once(0)).collect();
        infile = hts::hts_open(inname_c.as_ptr().cast(), c"r".as_ptr());
        outfile = hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
        if infile.is_null() || outfile.is_null() {
            write!(__out, "Could not open input/output\n").unwrap();
        } else {
            in_samhdr = sam::sam_hdr_read(infile);
            if in_samhdr.is_null() {
                write!(__out, "Failed to read header from file!\n").unwrap();
            } else if sam::sam_hdr_write(outfile, in_samhdr) == -1 {
                write!(__out, "Failed to write header\n").unwrap();
            } else {
                let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
                while ret_r >= 0 {
                    ret = 0;
                    let cur_qname =
                        std::ffi::CStr::from_ptr(sam::bam_get_qname(bamdata).cast::<i8>()).to_bytes();
                    if qname.eq_ignore_ascii_case(cur_qname) {
                        match field {
                            1 => {
                                let val_c: Vec<u8> =
                                    val.iter().copied().chain(std::iter::once(0)).collect();
                                ret = sam::bam_set_qname(bamdata, val_c.as_ptr().cast());
                            }
                            2 => {
                                let v: i64 =
                                    std::str::from_utf8(val).unwrap_or("").trim().parse().unwrap_or(0);
                                (*bamdata).core.flag = (v & 0xffff) as u16;
                            }
                            3 | 7 => {
                                ret = sam::sam_hdr_name2tid(&mut *in_samhdr, val);
                                if ret < 0 {
                                    write!(__out, "Invalid reference name\n").unwrap();
                                    ret = -1;
                                } else if field == 3 {
                                    (*bamdata).core.tid = ret;
                                } else {
                                    (*bamdata).core.mtid = ret;
                                }
                            }
                            4 => {
                                let v: i64 =
                                    std::str::from_utf8(val).unwrap_or("").trim().parse().unwrap_or(0);
                                (*bamdata).core.pos = v as hts::hts_pos_t;
                            }
                            5 => {
                                let v: i32 =
                                    std::str::from_utf8(val).unwrap_or("").trim().parse().unwrap_or(0);
                                (*bamdata).core.qual = (v & 0x0ff) as u8;
                            }
                            6 => {
                                let mut cigar: *mut u32 = std::ptr::null_mut();
                                let mut size = 0usize;
                                let ncigar = sam::sam_parse_cigar(
                                    val.as_ptr().cast(),
                                    std::ptr::null_mut(),
                                    &mut cigar,
                                    &mut size,
                                );
                                if ncigar < 0 {
                                    write!(__out, "Failed to parse cigar\n").unwrap();
                                    ret = -1;
                                } else {
                                    let newbam = sam::bam_init1();
                                    if newbam.is_null() {
                                        write!(__out, "Failed to create new bam data\n").unwrap();
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
                                        write!(__out, "Failed to set bamdata\n").unwrap();
                                        sam::bam_destroy1(newbam);
                                        ret = -1;
                                    } else {
                                        let seq_len = ((*bamdata).core.l_qseq as usize).div_ceil(2);
                                        std::slice::from_raw_parts_mut(
                                            sam::bam_get_seq(newbam).cast_mut(),
                                            seq_len,
                                        )
                                        .copy_from_slice(std::slice::from_raw_parts(
                                            sam::bam_get_seq(bamdata),
                                            seq_len,
                                        ));
                                        let aux_len = sam::bam_get_l_aux(bamdata) as usize;
                                        std::slice::from_raw_parts_mut(
                                            sam::bam_get_aux(newbam).cast_mut(),
                                            aux_len,
                                        )
                                        .copy_from_slice(std::slice::from_raw_parts(
                                            sam::bam_get_aux(bamdata),
                                            aux_len,
                                        ));
                                        sam::bam_destroy1(bamdata);
                                        bamdata = newbam;
                                    }
                                }
                                if !cigar.is_null() {
                                    // cigar was allocated by the raw C-ABI sam_parse_cigar;
                                    // hand the buffer back to its owner to release.
                                    drop(Box::from_raw(cigar));
                                }
                            }
                            8 => {
                                let v: i64 =
                                    std::str::from_utf8(val).unwrap_or("").trim().parse().unwrap_or(0);
                                (*bamdata).core.mpos = v as hts::hts_pos_t;
                            }
                            9 => {
                                let v: i64 =
                                    std::str::from_utf8(val).unwrap_or("").trim().parse().unwrap_or(0);
                                (*bamdata).core.isize = v as hts::hts_pos_t;
                            }
                            10 => {
                                let len = val.len() as i32;
                                if (*bamdata).core.l_qseq != len {
                                    write!(__out, "SEQ length different\n").unwrap();
                                    ret = -1;
                                } else {
                                    let seq = sam::bam_get_seq(bamdata).cast_mut();
                                    for i in 0..len as usize {
                                        sam::bam_set_seqi(
                                            seq,
                                            i,
                                            sam::SEQ_NT16_TABLE[val[i] as usize],
                                        );
                                    }
                                }
                            }
                            11 => {
                                let len = val.len() as i32;
                                if len != (*bamdata).core.l_qseq {
                                    write!(__out, "Qual length different than sequence\n").unwrap();
                                    ret = -1;
                                } else {
                                    let qual = std::slice::from_raw_parts_mut(
                                        sam::bam_get_qual(bamdata).cast_mut(),
                                        len as usize,
                                    );
                                    for i in 0..len as usize {
                                        qual[i] = val[i].wrapping_sub(33);
                                    }
                                }
                            }
                            _ => {
                                write!(__out, "Invalid input\n").unwrap();
                                ret = 1;
                                break;
                            }
                        }
                        if ret < 0 {
                            write!(__out, "Failed to set new data\n").unwrap();
                            ret = 1;
                            break;
                        }
                    }
                    if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                        write!(__out, "Failed to write bam data\n").unwrap();
                        ret = 1;
                        break;
                    }
                    ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
                }
                if ret_r == -1 || ret != 1 {
                    ret = 0;
                } else {
                    write!(__out, "Failed to read data\n").unwrap();
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
    __out.flush().unwrap();
    ret
}
