use std::ffi::{c_char, c_int};

use crate::htslib_mini_rs::sam;

// original: print_usage (htslib/samples/modstate.c:37)
pub unsafe fn samples_modstate_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: modstate infile option\nShows the base modifications on the alignment\nOption can be 1 or 2 to select the api to use\n".as_ptr(),
    );
}

// original: main (htslib/samples/modstate.c:49)
pub unsafe fn samples_modstate_c_49_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = libc::EXIT_FAILURE;

    if argc != 3 {
        samples_modstate_c_37_print_usage(hts_sys::stderr.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let opt = libc::atoi(*argv.add(2)) - 1;

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to allocate data memory!\n".as_ptr());
        return ret;
    }

    let ms = sam::hts_base_mod_state_alloc();
    if ms.is_null() {
        libc::printf(c"Failed to allocate state memory\n".as_ptr());
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let infile = crate::htslib_mini_rs::hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), inname);
        sam::bam_destroy1(bamdata);
        sam::hts_base_mod_state_free(ms);
        return ret;
    }
    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file!\n".as_ptr());
        crate::htslib_mini_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        sam::hts_base_mod_state_free(ms);
        return ret;
    }

    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        let mut i = 0;
        let data = sam::bam_get_seq(bamdata);
        if sam::bam_parse_basemod(bamdata, ms) != 0 {
            libc::printf(c"Failed to parse the base mods\n".as_ptr());
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_mini_rs::hts::hts_close(infile);
            sam::bam_destroy1(bamdata);
            sam::hts_base_mod_state_free(ms);
            return ret;
        }

        libc::printf(c"Modifications:".as_ptr());
        let mut cnt = 0;
        let bm = sam::bam_mods_recorded(ms, &mut cnt);
        for k in 0..cnt {
            libc::printf(c"%c".as_ptr(), *bm.add(k as usize));
        }
        libc::printf(c"\n".as_ptr());

        let mut mods = [sam::hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 5];
        if opt != 0 {
            while i < (*bamdata).core.l_qseq {
                let r =
                    sam::bam_mods_at_next_pos(bamdata, ms, mods.as_mut_ptr(), mods.len() as c_int);
                if r <= -1 {
                    libc::printf(c"Failed to get modifications\n".as_ptr());
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_mini_rs::hts::hts_close(infile);
                    sam::bam_destroy1(bamdata);
                    sam::hts_base_mod_state_free(ms);
                    return ret;
                } else if r as usize > mods.len() {
                    libc::printf(
                        c"More modifications than this app can handle, update the app\n".as_ptr(),
                    );
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_mini_rs::hts::hts_close(infile);
                    sam::bam_destroy1(bamdata);
                    sam::hts_base_mod_state_free(ms);
                    return ret;
                } else if r == 0 {
                    libc::printf(
                        c"%c".as_ptr(),
                        SEQ_NT16_STR[sam::bam_seqi(data, i as usize) as usize] as c_int,
                    );
                }
                for j in 0..r {
                    let m = mods[j as usize];
                    libc::printf(
                        c"%c%c%c".as_ptr(),
                        m.canonical_base,
                        if m.strand != 0 { b'-' } else { b'+' } as c_int,
                        m.modified_base,
                    );
                }
                i += 1;
            }
        } else {
            let mut pos = 0;
            loop {
                let r = sam::bam_next_basemod(
                    bamdata,
                    ms,
                    mods.as_mut_ptr(),
                    mods.len() as c_int,
                    &mut pos,
                );
                if r < 0 {
                    libc::printf(c"Failed to get modifications\n".as_ptr());
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_mini_rs::hts::hts_close(infile);
                    sam::bam_destroy1(bamdata);
                    sam::hts_base_mod_state_free(ms);
                    return ret;
                }
                while i < (*bamdata).core.l_qseq && i < pos {
                    libc::printf(
                        c"%c".as_ptr(),
                        SEQ_NT16_STR[sam::bam_seqi(data, i as usize) as usize] as c_int,
                    );
                    i += 1;
                }
                for j in 0..r {
                    let m = mods[j as usize];
                    libc::printf(
                        c"%c%c%c".as_ptr(),
                        m.canonical_base,
                        if m.strand != 0 { b'-' } else { b'+' } as c_int,
                        m.modified_base,
                    );
                }
                if i == pos {
                    i += 1;
                }
                if r == 0 {
                    while i < (*bamdata).core.l_qseq {
                        libc::printf(
                            c"%c".as_ptr(),
                            SEQ_NT16_STR[sam::bam_seqi(data, i as usize) as usize] as c_int,
                        );
                        i += 1;
                    }
                    break;
                }
            }
        }
        libc::printf(c"\n".as_ptr());
        ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    }

    if ret_r == -1 {
        let mut strand = 0;
        let mut implicit = 0;
        let mut canonical = 0;
        let modification = b"mhfcgebaon";
        libc::printf(c"\n\nLast alignment has \n".as_ptr());
        for code in modification {
            if sam::bam_mods_query_type(
                ms,
                *code as c_int,
                &mut strand,
                &mut implicit,
                &mut canonical,
            ) != 0
            {
                libc::printf(c"No modification of %c type\n".as_ptr(), *code as c_int);
            } else {
                libc::printf(
                    c"%s strand has %c modified with %c, can %sassume unlisted as unmodified\n"
                        .as_ptr(),
                    if strand != 0 {
                        c"-/bottom/reverse".as_ptr()
                    } else {
                        c"+/top/forward".as_ptr()
                    },
                    canonical as c_int,
                    *code as c_int,
                    if implicit != 0 {
                        c"".as_ptr()
                    } else {
                        c"not ".as_ptr()
                    },
                );
            }
        }
        ret = libc::EXIT_SUCCESS;
    } else {
        libc::printf(c"Failed to read data\n".as_ptr());
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_mini_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    sam::hts_base_mod_state_free(ms);
    ret
}
