use std::ffi::{c_char, c_int};

use crate::htslib_rs::sam;

// original: print_usage (htslib/samples/rem_header.c:37)
pub unsafe fn samples_rem_header_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: rem_header infile header [id]\nRemoves header line of given type and id\n"
            .as_ptr(),
    );
}

// original: main (htslib/samples/rem_header.c:49)
pub unsafe fn samples_rem_header_c_49_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;

    if !(3..=4).contains(&argc) {
        samples_rem_header_c_37_print_usage(crate::htslib_rs::c_compat::stderr.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let header = *argv.add(2);
    let idval = if argc == 4 {
        *argv.add(3)
    } else {
        std::ptr::null_mut()
    };

    let id = if *header == b'H' as c_char && *header.add(1) == b'D' as c_char {
        std::ptr::null()
    } else if *header == b'S' as c_char && *header.add(1) == b'Q' as c_char {
        c"SN".as_ptr()
    } else if (*header == b'R' as c_char || *header == b'P' as c_char)
        && *header.add(1) == b'G' as c_char
    {
        c"ID".as_ptr()
    } else if *header == b'C' as c_char && *header.add(1) == b'O' as c_char {
        c"".as_ptr()
    } else {
        libc::printf(c"Invalid header type\n".as_ptr());
        return ret;
    };

    let infile = crate::htslib_rs::hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), inname);
        return ret;
    }
    let outfile = crate::htslib_rs::hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
    if outfile.is_null() {
        libc::printf(c"Could not open stdout\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file!\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        return ret;
    }

    let remove_ret = if !idval.is_null() {
        sam::sam_hdr_remove_line_id(
            &mut *in_samhdr,
            std::ffi::CStr::from_ptr(header),
            if id.is_null() {
                None
            } else {
                Some((
                    std::ffi::CStr::from_ptr(id),
                    std::ffi::CStr::from_ptr(idval),
                ))
            },
        )
    } else {
        sam::sam_hdr_remove_lines(
            &mut *in_samhdr,
            std::ffi::CStr::from_ptr(header),
            if id.is_null() {
                None
            } else {
                Some(std::ffi::CStr::from_ptr(id))
            },
            None,
        )
    };
    if remove_ret != 0 {
        libc::printf(c"Failed to remove header line\n".as_ptr());
    } else if sam::sam_hdr_write(outfile, in_samhdr) < 0 {
        libc::printf(c"Failed to write output\n".as_ptr());
    } else {
        ret = libc::EXIT_SUCCESS;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::hts_close(outfile);
    ret
}
