use std::ffi::{c_char, c_int, c_uint};

use crate::htslib_mini_rs::sam;

unsafe fn printauxdata(fp: *mut libc::FILE, type_: c_char, idx: i32, data: *const u8) -> c_int {
    match type_ as u8 {
        b'A' => {
            libc::fprintf(fp, c"%c".as_ptr(), sam::bam_aux2A(data) as c_int);
        }
        b'c' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            libc::fprintf(fp, c"%d".as_ptr(), value as i8 as c_int);
        }
        b'C' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            libc::fprintf(fp, c"%u".as_ptr(), value as u8 as c_uint);
        }
        b's' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            libc::fprintf(fp, c"%d".as_ptr(), value as i16 as c_int);
        }
        b'S' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            libc::fprintf(fp, c"%u".as_ptr(), value as u16 as c_uint);
        }
        b'i' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            libc::fprintf(fp, c"%d".as_ptr(), value as i32);
        }
        b'I' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            libc::fprintf(fp, c"%u".as_ptr(), value as u32);
        }
        b'f' | b'd' => {
            let value = if idx > -1 {
                sam::bam_auxB2f(data, idx as u32)
            } else {
                sam::bam_aux2f(data)
            };
            libc::fprintf(fp, c"%g".as_ptr(), value as f32 as f64);
        }
        b'H' | b'Z' => {
            libc::fprintf(fp, c"%s".as_ptr(), sam::bam_aux2Z(data));
        }
        b'B' => {
            let aux_b_count = sam::bam_auxB_len(data);
            let aux_b_type = sam::bam_aux_type(data.add(1));
            libc::fprintf(fp, c"%c".as_ptr(), aux_b_type as c_int);
            for i in 0..aux_b_count {
                libc::fprintf(fp, c",".as_ptr());
                if printauxdata(fp, aux_b_type, i as i32, data) == libc::EXIT_FAILURE {
                    return libc::EXIT_FAILURE;
                }
            }
        }
        _ => {
            libc::printf(c"Invalid aux tag?\n".as_ptr());
            return libc::EXIT_FAILURE;
        }
    }
    libc::EXIT_SUCCESS
}

// original: print_usage (htslib/samples/dump_aux.c:37)
pub unsafe fn samples_dump_aux_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: dump_aux infile\nDump the aux tags from alignments\n".as_ptr(),
    );
}

// original: printauxdata (htslib/samples/dump_aux.c:51)
pub unsafe fn samples_dump_aux_c_51_printauxdata(
    fp: *mut libc::FILE,
    type_: c_char,
    idx: i32,
    data: *const u8,
) -> c_int {
    printauxdata(fp, type_, idx, data)
}

// original: main (htslib/samples/dump_aux.c:114)
pub unsafe fn samples_dump_aux_c_114_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;

    if argc != 2 {
        samples_dump_aux_c_37_print_usage(hts_sys::stderr.cast());
        return ret;
    }
    let inname = *argv.add(1);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to allocate data memory!\n".as_ptr());
        return ret;
    }

    let infile = crate::htslib_mini_rs::hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), inname);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file!\n".as_ptr());
        crate::htslib_mini_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        *crate::htslib_mini_rs::c_compat::__errno_location() = 0;
        let mut data = sam::bam_aux_first(bamdata);
        while !data.is_null() {
            let type_ = sam::bam_aux_type(data);
            libc::printf(
                c"%.2s:%c:".as_ptr(),
                sam::bam_aux_tag(data),
                if libc::strchr(c"cCsSiI".as_ptr(), type_ as c_int).is_null() {
                    type_ as c_int
                } else {
                    b'i' as c_int
                },
            );
            if samples_dump_aux_c_51_printauxdata(hts_sys::stdout.cast(), type_, -1, data)
                == libc::EXIT_FAILURE
            {
                libc::printf(c"Failed to dump aux data\n".as_ptr());
                sam::sam_hdr_destroy(in_samhdr);
                crate::htslib_mini_rs::hts::hts_close(infile);
                sam::bam_destroy1(bamdata);
                return ret;
            }
            libc::printf(c" ".as_ptr());
            data = sam::bam_aux_next(bamdata, data);
        }
        if *crate::htslib_mini_rs::c_compat::__errno_location()
            != crate::htslib_mini_rs::c_compat::ENOENT as c_int
        {
            libc::printf(c"\nFailed to get aux data\n".as_ptr());
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_mini_rs::hts::hts_close(infile);
            sam::bam_destroy1(bamdata);
            return ret;
        }
        libc::printf(c"\n".as_ptr());
        ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    }

    if ret_r < -1 {
        libc::printf(c"Failed to read data\n".as_ptr());
    } else {
        ret = libc::EXIT_SUCCESS;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_mini_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    ret
}
