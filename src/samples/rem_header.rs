use crate::htslib_rs::sam;
use std::io::Write;

// original: print_usage (htslib/samples/rem_header.c:37)
pub unsafe fn samples_rem_header_c_37_print_usage() {
    eprint!("Usage: rem_header infile header [id]\nRemoves header line of given type and id\n");
}

// original: main (htslib/samples/rem_header.c:49)
pub unsafe fn samples_rem_header_c_49_main(args: &[&[u8]]) -> i32 {
    let mut ret = 1;
    let mut __out = std::io::stdout();

    let argc = args.len();
    if !(3..=4).contains(&argc) {
        samples_rem_header_c_37_print_usage();
        return ret;
    }
    let inname = args[1];
    let header = args[2];
    let idval: Option<&[u8]> = if argc == 4 { Some(args[3]) } else { None };

    let id: Option<&[u8]> = if header[0] == b'H' && header[1] == b'D' {
        None
    } else if header[0] == b'S' && header[1] == b'Q' {
        Some(b"SN")
    } else if (header[0] == b'R' || header[0] == b'P') && header[1] == b'G' {
        Some(b"ID")
    } else if header[0] == b'C' && header[1] == b'O' {
        Some(b"")
    } else {
        write!(__out, "Invalid header type\n").unwrap();
        __out.flush().unwrap();
        return ret;
    };

    let inname_c: Vec<u8> = inname.iter().copied().chain(std::iter::once(0)).collect();
    let infile = crate::htslib_rs::hts::hts_open(inname_c.as_ptr().cast(), c"r".as_ptr());
    if infile.is_null() {
        write!(__out, "Could not open {}\n", String::from_utf8_lossy(inname)).unwrap();
        __out.flush().unwrap();
        return ret;
    }
    let outfile = crate::htslib_rs::hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
    if outfile.is_null() {
        write!(__out, "Could not open stdout\n").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        __out.flush().unwrap();
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        write!(__out, "Failed to read header from file!\n").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        __out.flush().unwrap();
        return ret;
    }

    let remove_ret = if let Some(idval) = idval {
        sam::sam_hdr_remove_line_id(
            &mut *in_samhdr,
            header,
            id.map(|id| (id, idval)),
        )
    } else {
        sam::sam_hdr_remove_lines(&mut *in_samhdr, header, id, None)
    };
    if remove_ret != 0 {
        write!(__out, "Failed to remove header line\n").unwrap();
    } else if sam::sam_hdr_write(outfile, in_samhdr) < 0 {
        write!(__out, "Failed to write output\n").unwrap();
    } else {
        ret = 0;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::hts_close(outfile);
    __out.flush().unwrap();
    ret
}
