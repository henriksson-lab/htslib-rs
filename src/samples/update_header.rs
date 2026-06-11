use crate::htslib_rs::sam;
use std::io::Write;

// original: print_usage (htslib/samples/update_header.c:37)
pub unsafe fn samples_update_header_c_37_print_usage() {
    eprint!("Usage: update_header infile header idval tag value\nUpdates the tag's value on line given in id on header of given type\n");
}

// original: main (htslib/samples/update_header.c:49)
pub unsafe fn samples_update_header_c_49_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut __out = std::io::stdout();
    let mut ret = 1;

    if argc != 6 {
        samples_update_header_c_37_print_usage();
        return ret;
    }

    let inname = *argv.add(1);
    let header = std::ffi::CStr::from_ptr((*argv.add(2)).cast()).to_bytes();
    let idval = std::ffi::CStr::from_ptr((*argv.add(3)).cast()).to_bytes();
    let mut tag: Option<&[u8]> = Some(std::ffi::CStr::from_ptr((*argv.add(4)).cast()).to_bytes());
    let val = std::ffi::CStr::from_ptr((*argv.add(5)).cast()).to_bytes();

    let id: Option<&[u8]> = if header.len() >= 2 && header[0] == b'H' && header[1] == b'D' {
        writeln!(__out, "This sample doesnt not support modifying HD fields").unwrap();
        None
    } else if header.len() >= 2 && header[0] == b'S' && header[1] == b'Q' {
        Some(b"SN")
    } else if header.len() >= 2
        && (header[0] == b'R' || header[0] == b'P')
        && header[1] == b'G'
    {
        Some(b"ID")
    } else if header.len() >= 2 && header[0] == b'C' && header[1] == b'O' {
        tag = None;
        writeln!(__out, "This sample doesnt not support modifying CO fields").unwrap();
        Some(b"")
    } else {
        writeln!(__out, "Invalid header type").unwrap();
        __out.flush().unwrap();
        return ret;
    };

    let infile = crate::htslib_rs::hts::hts_open(inname.cast(), c"r".as_ptr());
    if infile.is_null() {
        writeln!(
            __out,
            "Could not open {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(inname.cast()).to_bytes())
        )
        .unwrap();
        __out.flush().unwrap();
        return ret;
    }
    let outfile = crate::htslib_rs::hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
    if outfile.is_null() {
        writeln!(__out, "Could not open stdout").unwrap();
        __out.flush().unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        writeln!(__out, "Failed to read header from file!").unwrap();
        __out.flush().unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        return ret;
    }

    // Stay on the native slice-arg sam_hdr_update_line so the header's hrecs
    // is populated by Rust code (matching layout). Routing through hts_sys'
    // variadic C entry-point would write a C-allocated sam_hrecs_t into
    // (*h).hrecs that our native sam_hdr_rebuild/sam_hdr_write cannot
    // interpret afterwards.
    let id_pair = id.map(|id| (id, idval));
    if sam::sam_hdr_update_line(&mut *in_samhdr, header, id_pair, &[(tag, Some(val))]) < 0 {
        writeln!(__out, "Failed to update data").unwrap();
    } else if sam::sam_hdr_write(outfile, in_samhdr) < 0 {
        writeln!(__out, "Failed to write output").unwrap();
    } else {
        ret = 0;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::hts_close(outfile);
    __out.flush().unwrap();
    ret
}
