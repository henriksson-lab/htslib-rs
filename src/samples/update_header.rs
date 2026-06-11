use std::ffi::{c_char, c_int, CStr};

use crate::htslib_rs::sam;

// original: print_usage (htslib/samples/update_header.c:37)
pub unsafe fn samples_update_header_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: update_header infile header idval tag value\nUpdates the tag's value on line given in id on header of given type\n".as_ptr(),
    );
}

// original: main (htslib/samples/update_header.c:49)
pub unsafe fn samples_update_header_c_49_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;

    if argc != 6 {
        samples_update_header_c_37_print_usage(crate::htslib_rs::c_compat::stderr.cast());
        return ret;
    }

    let inname = *argv.add(1);
    let header = *argv.add(2);
    let idval = *argv.add(3);
    let mut tag = *argv.add(4);
    let val = *argv.add(5);

    let id = if *header == b'H' as c_char && *header.add(1) == b'D' as c_char {
        libc::printf(c"This sample doesnt not support modifying HD fields\n".as_ptr());
        std::ptr::null()
    } else if *header == b'S' as c_char && *header.add(1) == b'Q' as c_char {
        c"SN".as_ptr()
    } else if (*header == b'R' as c_char || *header == b'P' as c_char)
        && *header.add(1) == b'G' as c_char
    {
        c"ID".as_ptr()
    } else if *header == b'C' as c_char && *header.add(1) == b'O' as c_char {
        tag = std::ptr::null_mut();
        libc::printf(c"This sample doesnt not support modifying CO fields\n".as_ptr());
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

    // Stay on the native slice-arg sam_hdr_update_line so the header's hrecs
    // is populated by Rust code (matching layout). Routing through hts_sys'
    // variadic C entry-point would write a C-allocated sam_hrecs_t into
    // (*h).hrecs that our native sam_hdr_rebuild/sam_hdr_write cannot
    // interpret afterwards.
    let id_pair = if id.is_null() {
        None
    } else {
        Some((CStr::from_ptr(id), CStr::from_ptr(idval)))
    };
    if sam::sam_hdr_update_line(
        &mut *in_samhdr,
        CStr::from_ptr(header),
        id_pair,
        &[(tag, val)],
    ) < 0
    {
        libc::printf(c"Failed to update data\n".as_ptr());
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
