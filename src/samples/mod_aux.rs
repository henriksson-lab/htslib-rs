use std::io::Write;

use crate::htslib_rs::sam;

// original: print_usage (htslib/samples/mod_aux.c:38)
pub unsafe fn samples_mod_aux_c_38_print_usage() {
    eprint!(
        "Usage: mod_aux infile QNAME tag type val\nAdd/update the given aux tag to all alignments\ntype A-char C-int F-float Z-string\n"
    );
}

// original: main (htslib/samples/mod_aux.c:50)
pub unsafe fn samples_mod_aux_c_50_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut ret: i32 = 1;
    let mut __out = std::io::stdout();

    if argc != 6 {
        samples_mod_aux_c_38_print_usage();
        return ret;
    }
    // Command-line args remain raw NUL-terminated C strings; the sam/hts APIs
    // they are handed to are still raw-ptr C-ABI functions.
    let inname = *argv.add(1);
    let qname = *argv.add(2);
    let tag = *argv.add(3);
    let mut type_: u8 = **argv.add(4);
    let mut val = *argv.add(5);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to allocate data memory!\n").unwrap();
        return ret;
    }
    let infile = crate::htslib_rs::hts::hts_open(inname.cast(), b"r\0".as_ptr().cast());
    if infile.is_null() {
        write!(__out, "Could not open {}\n", String::from_utf8_lossy(std::ffi::CStr::from_ptr(inname.cast()).to_bytes())).unwrap();
        sam::bam_destroy1(bamdata);
        return ret;
    }
    let outfile = crate::htslib_rs::hts::hts_open(b"-\0".as_ptr().cast(), b"w\0".as_ptr().cast());
    if outfile.is_null() {
        write!(__out, "Could not open std output\n").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }
    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        write!(__out, "Failed to read header from file!\n").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        sam::bam_destroy1(bamdata);
        return ret;
    }
    if sam::sam_hdr_write(outfile, in_samhdr) == -1 {
        write!(__out, "Failed to write header\n").unwrap();
        sam::sam_hdr_destroy(in_samhdr);
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        if std::ffi::CStr::from_ptr(sam::bam_get_qname(bamdata).cast())
            .to_bytes()
            .eq_ignore_ascii_case(std::ffi::CStr::from_ptr(qname.cast()).to_bytes())
            == false
        {
            if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                write!(__out, "Failed to write output\n").unwrap();
                sam::sam_hdr_destroy(in_samhdr);
                crate::htslib_rs::hts::hts_close(infile);
                crate::htslib_rs::hts::hts_close(outfile);
                sam::bam_destroy1(bamdata);
                return ret;
            }
            ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
            continue;
        }

        std::io::Error::last_os_error();
        let data = sam::bam_aux_get(bamdata, tag.cast());
        if data.is_null() {
            let mut int_val: i32;
            let mut float_val: f32;
            let length;
            match type_ {
                x if x == b'f' || x == b'd' => {
                    length = std::mem::size_of::<f32>() as i32;
                    float_val = String::from_utf8_lossy(std::ffi::CStr::from_ptr(val.cast()).to_bytes())
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(0.0);
                    val = (&mut float_val as *mut f32).cast();
                    type_ = b'f';
                }
                x if x == b'C' || x == b'S' || x == b'I' => {
                    length = std::mem::size_of::<i32>() as i32;
                    int_val = String::from_utf8_lossy(std::ffi::CStr::from_ptr(val.cast()).to_bytes())
                        .trim()
                        .parse::<i32>()
                        .unwrap_or(0);
                    val = (&mut int_val as *mut i32).cast();
                }
                x if x == b'Z' => {
                    length = std::ffi::CStr::from_ptr(val.cast()).to_bytes().len() as i32 + 1;
                }
                x if x == b'A' => {
                    length = 1;
                }
                _ => {
                    write!(__out, "Invalid type mentioned\n").unwrap();
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_rs::hts::hts_close(infile);
                    crate::htslib_rs::hts::hts_close(outfile);
                    sam::bam_destroy1(bamdata);
                    return ret;
                }
            }
            if sam::bam_aux_append(bamdata, tag.cast(), type_, length, val.cast()) != 0 {
                write!(
                    __out,
                    "Failed to append aux data, errno: {}\n",
                    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
                ).unwrap();
                sam::sam_hdr_destroy(in_samhdr);
                crate::htslib_rs::hts::hts_close(infile);
                crate::htslib_rs::hts::hts_close(outfile);
                sam::bam_destroy1(bamdata);
                return ret;
            }
        } else {
            let auxtype = sam::bam_aux_type(data);
            match type_ {
                x if x == b'f' || x == b'd' => {
                    if auxtype != b'f' && auxtype != b'd' {
                        write!(__out, "Invalid aux type passed\n").unwrap();
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                    let fval = String::from_utf8_lossy(std::ffi::CStr::from_ptr(val.cast()).to_bytes())
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(0.0);
                    if sam::bam_aux_update_float(bamdata, tag.cast(), fval) != 0 {
                        write!(
                            __out,
                            "Failed to update float data, errno: {}\n",
                            std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
                        ).unwrap();
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                }
                x if x == b'C' || x == b'S' || x == b'I' => {
                    if auxtype != b'c'
                        && auxtype != b'C'
                        && auxtype != b's'
                        && auxtype != b'S'
                        && auxtype != b'i'
                        && auxtype != b'I'
                    {
                        write!(__out, "Invalid aux type passed\n").unwrap();
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                    let ival = String::from_utf8_lossy(std::ffi::CStr::from_ptr(val.cast()).to_bytes())
                        .trim()
                        .parse::<i64>()
                        .unwrap_or(0);
                    if sam::bam_aux_update_int(bamdata, tag.cast(), ival) != 0 {
                        write!(
                            __out,
                            "Failed to update int data, errno: {}\n",
                            std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
                        ).unwrap();
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                }
                x if x == b'Z' => {
                    if auxtype != b'Z' {
                        write!(__out, "Invalid aux type passed\n").unwrap();
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                    let length = std::ffi::CStr::from_ptr(val.cast()).to_bytes().len() as i32 + 1;
                    if sam::bam_aux_update_str(bamdata, tag.cast(), length, val.cast()) != 0 {
                        write!(
                            __out,
                            "Failed to update string data, errno: {}\n",
                            std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
                        ).unwrap();
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                }
                x if x == b'A' => {
                    if auxtype != b'A' {
                        write!(__out, "Invalid aux type passed\n").unwrap();
                        sam::sam_hdr_destroy(in_samhdr);
                        crate::htslib_rs::hts::hts_close(infile);
                        crate::htslib_rs::hts::hts_close(outfile);
                        sam::bam_destroy1(bamdata);
                        return ret;
                    }
                    *data.add(1) = *val;
                }
                _ => {
                    write!(__out, "Invalid data type\n").unwrap();
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_rs::hts::hts_close(infile);
                    crate::htslib_rs::hts::hts_close(outfile);
                    sam::bam_destroy1(bamdata);
                    return ret;
                }
            }
        }

        if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
            write!(__out, "Failed to write output\n").unwrap();
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            crate::htslib_rs::hts::hts_close(outfile);
            sam::bam_destroy1(bamdata);
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
