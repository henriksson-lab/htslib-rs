use std::ffi::{c_char, c_int};

use crate::htslib_rs::sam;

// original: print_usage (htslib/samples/mod_aux.c:38)
pub unsafe fn samples_mod_aux_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: mod_aux infile QNAME tag type val\nAdd/update the given aux tag to all alignments\ntype A-char C-int F-float Z-string\n".as_ptr(),
    );
}

// original: main (htslib/samples/mod_aux.c:50)
pub unsafe fn samples_mod_aux_c_50_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;

    if argc != 6 {
        samples_mod_aux_c_38_print_usage(crate::htslib_rs::c_compat::stderr.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let qname = *argv.add(2);
    let tag = *argv.add(3);
    let mut type_ = **argv.add(4);
    let mut val = *argv.add(5);

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
        if libc::strcasecmp(sam::bam_get_qname(bamdata), qname) != 0 {
            if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                libc::printf(c"Failed to write output\n".as_ptr());
                sam::sam_hdr_destroy(in_samhdr);
                crate::htslib_rs::hts::hts_close(infile);
                crate::htslib_rs::hts::hts_close(outfile);
                sam::bam_destroy1(bamdata);
                return ret;
            }
            ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
            continue;
        }

        *crate::htslib_rs::c_compat::__errno_location() = 0;
        let data = sam::bam_aux_get(bamdata, tag);
        if data.is_null() {
            let mut int_val: c_int;
            let mut float_val: f32;
            let length;
            match type_ {
                x if x == b'f' as c_char || x == b'd' as c_char => {
                    length = std::mem::size_of::<f32>() as c_int;
                    float_val = libc::atof(val) as f32;
                    val = (&mut float_val as *mut f32).cast();
                    type_ = b'f' as c_char;
                }
                x if x == b'C' as c_char || x == b'S' as c_char || x == b'I' as c_char => {
                    length = std::mem::size_of::<c_int>() as c_int;
                    int_val = libc::atoi(val);
                    val = (&mut int_val as *mut c_int).cast();
                }
                x if x == b'Z' as c_char => {
                    length = libc::strlen(val) as c_int + 1;
                }
                x if x == b'A' as c_char => {
                    length = 1;
                }
                _ => {
                    libc::printf(c"Invalid type mentioned\n".as_ptr());
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_rs::hts::hts_close(infile);
                    crate::htslib_rs::hts::hts_close(outfile);
                    sam::bam_destroy1(bamdata);
                    return ret;
                }
            }
            if sam::bam_aux_append(bamdata, tag, type_, length, val.cast()) != 0 {
                libc::printf(
                    c"Failed to append aux data, errno: %d\n".as_ptr(),
                    *crate::htslib_rs::c_compat::__errno_location(),
                );
                sam::sam_hdr_destroy(in_samhdr);
                crate::htslib_rs::hts::hts_close(infile);
                crate::htslib_rs::hts::hts_close(outfile);
                sam::bam_destroy1(bamdata);
                return ret;
            }
        } else {
            let auxtype = sam::bam_aux_type(data);
            match type_ {
                x if x == b'f' as c_char || x == b'd' as c_char => {
                    if auxtype != b'f' as c_char && auxtype != b'd' as c_char {
                        libc::printf(c"Invalid aux type passed\n".as_ptr());
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                    if sam::bam_aux_update_float(bamdata, tag, libc::atof(val) as f32) != 0 {
                        libc::printf(
                            c"Failed to update float data, errno: %d\n".as_ptr(),
                            *crate::htslib_rs::c_compat::__errno_location(),
                        );
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                }
                x if x == b'C' as c_char || x == b'S' as c_char || x == b'I' as c_char => {
                    if auxtype != b'c' as c_char
                        && auxtype != b'C' as c_char
                        && auxtype != b's' as c_char
                        && auxtype != b'S' as c_char
                        && auxtype != b'i' as c_char
                        && auxtype != b'I' as c_char
                    {
                        libc::printf(c"Invalid aux type passed\n".as_ptr());
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                    if sam::bam_aux_update_int(bamdata, tag, libc::atoll(val)) != 0 {
                        libc::printf(
                            c"Failed to update int data, errno: %d\n".as_ptr(),
                            *crate::htslib_rs::c_compat::__errno_location(),
                        );
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                }
                x if x == b'Z' as c_char => {
                    if auxtype != b'Z' as c_char {
                        libc::printf(c"Invalid aux type passed\n".as_ptr());
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                    let length = libc::strlen(val) as c_int + 1;
                    if sam::bam_aux_update_str(bamdata, tag, length, val) != 0 {
                        libc::printf(
                            c"Failed to update string data, errno: %d\n".as_ptr(),
                            *crate::htslib_rs::c_compat::__errno_location(),
                        );
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                }
                x if x == b'A' as c_char => {
                    if auxtype != b'A' as c_char {
                        libc::printf(c"Invalid aux type passed\n".as_ptr());
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                    *data.add(1) = *val as u8;
                }
                _ => {
                    libc::printf(c"Invalid data type\n".as_ptr());
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_rs::hts::hts_close(infile);
                    crate::htslib_rs::hts::hts_close(outfile);
                    sam::bam_destroy1(bamdata);
                    return ret;
                }
            }
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
