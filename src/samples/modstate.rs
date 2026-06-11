use std::io::Write;

use crate::htslib_rs::sam;

// original: print_usage (htslib/samples/modstate.c:37)
pub unsafe fn samples_modstate_c_37_print_usage() {
    eprint!("Usage: modstate infile option\nShows the base modifications on the alignment\nOption can be 1 or 2 to select the api to use\n");
}

// original: main (htslib/samples/modstate.c:49)
pub unsafe fn samples_modstate_c_49_main(argc: i32, argv: *mut *mut u8) -> i32 {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = 1;
    let mut __out = std::io::stdout();

    if argc != 3 {
        samples_modstate_c_37_print_usage();
        return ret;
    }
    let inname = *argv.add(1);
    let opt = libc::atoi((*argv.add(2)).cast()) - 1;

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to allocate data memory!\n").unwrap();
        return ret;
    }

    let mut ms = sam::hts_base_mod_state_alloc();

    let infile = crate::htslib_rs::hts::hts_open(inname.cast(), c"r".as_ptr());
    if infile.is_null() {
        let inname_str = std::ffi::CStr::from_ptr(inname.cast()).to_bytes();
        write!(__out, "Could not open {}\n", String::from_utf8_lossy(inname_str)).unwrap();
        sam::bam_destroy1(bamdata);
        sam::hts_base_mod_state_free(Some(ms));
        return ret;
    }
    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        write!(__out, "Failed to read header from file!\n").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        sam::hts_base_mod_state_free(Some(ms));
        return ret;
    }

    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        let mut i = 0;
        let data = sam::bam_get_seq(bamdata);
        if sam::bam_parse_basemod(&*bamdata, &mut ms) != 0 {
            write!(__out, "Failed to parse the base mods\n").unwrap();
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            sam::bam_destroy1(bamdata);
            sam::hts_base_mod_state_free(Some(ms));
            return ret;
        }

        write!(__out, "Modifications:").unwrap();
        let mut cnt = 0;
        let bm = sam::bam_mods_recorded(&mut ms, &mut cnt);
        for k in 0..cnt {
            write!(__out, "{}", bm[k as usize] as u8 as char).unwrap();
        }
        write!(__out, "\n").unwrap();

        let mut mods = [sam::hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 5];
        if opt != 0 {
            while i < (*bamdata).core.l_qseq {
                let r = sam::bam_mods_at_next_pos(&*bamdata, &mut ms, &mut mods);
                if r <= -1 {
                    write!(__out, "Failed to get modifications\n").unwrap();
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_rs::hts::hts_close(infile);
                    sam::bam_destroy1(bamdata);
                    sam::hts_base_mod_state_free(Some(ms));
                    return ret;
                } else if r as usize > mods.len() {
                    write!(__out, "More modifications than this app can handle, update the app\n").unwrap();
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_rs::hts::hts_close(infile);
                    sam::bam_destroy1(bamdata);
                    sam::hts_base_mod_state_free(Some(ms));
                    return ret;
                } else if r == 0 {
                    write!(
                        __out,
                        "{}",
                        SEQ_NT16_STR[sam::bam_seqi(data, i as usize) as usize] as char
                    ).unwrap();
                }
                for j in 0..r {
                    let m = mods[j as usize];
                    write!(
                        __out,
                        "{}{}{}",
                        m.canonical_base as u8 as char,
                        if m.strand != 0 { '-' } else { '+' },
                        m.modified_base as u8 as char
                    ).unwrap();
                }
                i += 1;
            }
        } else {
            let mut pos = 0;
            loop {
                let r = sam::bam_next_basemod(&*bamdata, &mut ms, &mut mods, &mut pos);
                if r < 0 {
                    write!(__out, "Failed to get modifications\n").unwrap();
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_rs::hts::hts_close(infile);
                    sam::bam_destroy1(bamdata);
                    sam::hts_base_mod_state_free(Some(ms));
                    return ret;
                }
                while i < (*bamdata).core.l_qseq && i < pos {
                    write!(
                        __out,
                        "{}",
                        SEQ_NT16_STR[sam::bam_seqi(data, i as usize) as usize] as char
                    ).unwrap();
                    i += 1;
                }
                for j in 0..r {
                    let m = mods[j as usize];
                    write!(
                        __out,
                        "{}{}{}",
                        m.canonical_base as u8 as char,
                        if m.strand != 0 { '-' } else { '+' },
                        m.modified_base as u8 as char
                    ).unwrap();
                }
                if i == pos {
                    i += 1;
                }
                if r == 0 {
                    while i < (*bamdata).core.l_qseq {
                        write!(
                            __out,
                            "{}",
                            SEQ_NT16_STR[sam::bam_seqi(data, i as usize) as usize] as char
                        ).unwrap();
                        i += 1;
                    }
                    break;
                }
            }
        }
        write!(__out, "\n").unwrap();
        ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    }

    if ret_r == -1 {
        let mut strand: i32 = 0;
        let mut implicit: i32 = 0;
        let mut canonical: u8 = 0;
        let modification = b"mhfcgebaon";
        write!(__out, "\n\nLast alignment has \n").unwrap();
        for code in modification {
            if sam::bam_mods_query_type(
                &ms,
                *code as i32,
                Some(&mut strand),
                Some(&mut implicit),
                Some(&mut canonical),
            ) != 0
            {
                write!(__out, "No modification of {} type\n", *code as char).unwrap();
            } else {
                write!(
                    __out,
                    "{} strand has {} modified with {}, can {}assume unlisted as unmodified\n",
                    if strand != 0 {
                        "-/bottom/reverse"
                    } else {
                        "+/top/forward"
                    },
                    canonical as char,
                    *code as char,
                    if implicit != 0 { "" } else { "not " }
                ).unwrap();
            }
        }
        ret = 0;
    } else {
        write!(__out, "Failed to read data\n").unwrap();
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    sam::hts_base_mod_state_free(Some(ms));
    __out.flush().unwrap();
    ret
}
