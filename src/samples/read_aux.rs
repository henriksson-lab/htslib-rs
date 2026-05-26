use std::ffi::{c_char, c_int, c_uint};

use crate::htslib_rs::{hts::kstring_t, sam};

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

// original: print_usage (htslib/samples/read_aux.c:37)
pub unsafe fn samples_read_aux_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_aux infile tag\nRead the given aux tag from alignments either as SAM string or as raw data\n".as_ptr(),
    );
}

// original: printauxdata (htslib/samples/read_aux.c:51)
pub unsafe fn samples_read_aux_c_51_printauxdata(
    fp: *mut libc::FILE,
    type_: c_char,
    idx: i32,
    data: *const u8,
) -> c_int {
    printauxdata(fp, type_, idx, data)
}

// original: main (htslib/samples/read_aux.c:114)
pub unsafe fn samples_read_aux_c_114_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut sdata = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };

    if argc != 3 {
        samples_read_aux_c_37_print_usage(crate::htslib_rs::c_compat::stderr.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let tag = *argv.add(2);

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

    let mut i = 0;
    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        *crate::htslib_rs::c_compat::__errno_location() = 0;
        i += 1;
        crate::htslib_rs::hts::ks_clear(&mut sdata);
        if i % 2 != 0 {
            let c = sam::bam_aux_get_str(bamdata, tag, &mut sdata);
            if c == 1 {
                libc::printf(c"%s\n".as_ptr(), sdata.s);
            } else if c == 0
                && *crate::htslib_rs::c_compat::__errno_location()
                    == crate::htslib_rs::c_compat::ENOENT as c_int
            {
                libc::printf(c"Tag not present\n".as_ptr());
            } else {
                libc::printf(c"Failed to get tag\n".as_ptr());
                sam::sam_hdr_destroy(in_samhdr);
                crate::htslib_rs::hts::hts_close(infile);
                sam::bam_destroy1(bamdata);
                crate::htslib_rs::hts::ks_free(&mut sdata);
                return ret;
            }
        } else {
            let data = sam::bam_aux_get(bamdata, tag);
            if data.is_null() {
                if *crate::htslib_rs::c_compat::__errno_location()
                    == crate::htslib_rs::c_compat::ENOENT as c_int
                {
                    libc::printf(c"Tag not present\n".as_ptr());
                } else {
                    libc::printf(c"Invalid aux data\n".as_ptr());
                }
            } else {
                if samples_read_aux_c_51_printauxdata(
                    crate::htslib_rs::c_compat::stdout.cast(),
                    sam::bam_aux_type(data),
                    -1,
                    data,
                ) == libc::EXIT_FAILURE
                {
                    libc::printf(c"Failed to read aux data\n".as_ptr());
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_rs::hts::hts_close(infile);
                    sam::bam_destroy1(bamdata);
                    crate::htslib_rs::hts::ks_free(&mut sdata);
                    return ret;
                }
                libc::printf(c"\n".as_ptr());
            }
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
    sam::bam_destroy1(bamdata);
    crate::htslib_rs::hts::ks_free(&mut sdata);
    ret
}
