use crate::htslib_rs::{hts::kstring_t, sam};
use std::io::Write;

// original: print_usage (htslib/samples/add_header.c:37)
pub unsafe fn samples_add_header_c_37_print_usage() {
    eprint!("Usage: add_header infile\nAdds new header lines of SQ, RG, PG and CO types\n");
}

// original: main (htslib/samples/add_header.c:49)
pub unsafe fn samples_add_header_c_49_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut ret = 1;
    let sq = b"@SQ\tSN:TR1\tLN:100\n@SQ\tSN:TR2\tLN:50\n";
    let mut data = kstring_t { data: Vec::new() };
    let mut __out = std::io::stdout();
    if argc != 2 {
        samples_add_header_c_37_print_usage();
        crate::htslib_rs::hts::ks_free(&mut data);
        return ret;
    }

    let inname = *argv.add(1);
    let infile = crate::htslib_rs::hts::hts_open(inname.cast(), c"r".as_ptr());
    if infile.is_null() {
        let inname_bytes = std::ffi::CStr::from_ptr(inname.cast()).to_bytes();
        writeln!(__out, "Could not open {}", String::from_utf8_lossy(inname_bytes)).unwrap();
        crate::htslib_rs::hts::ks_free(&mut data);
        __out.flush().unwrap();
        return ret;
    }
    let outfile = crate::htslib_rs::hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
    if outfile.is_null() {
        writeln!(__out, "Could not open stdout").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::ks_free(&mut data);
        __out.flush().unwrap();
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        writeln!(__out, "Failed to read header from file!").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        crate::htslib_rs::hts::ks_free(&mut data);
        __out.flush().unwrap();
        return ret;
    }

    for i in 0..argc {
        crate::htslib_rs::hts::kputs(
            std::ffi::CStr::from_ptr((*argv.add(i as usize)).cast()).to_bytes(),
            &mut data,
        );
        crate::htslib_rs::hts::kputc(b' ' as i32, &mut data);
    }

    if sam::sam_hdr_add_lines(&mut *in_samhdr, sq) != 0 {
        writeln!(__out, "Failed to add SQ lines").unwrap();
    } else if sam::sam_hdr_add_line(
        &mut *in_samhdr,
        b"RG",
        &[
            (Some(b"ID"), Some(b"RG1")),
            (Some(b"LB"), Some(b"Test")),
            (Some(b"SM"), Some(b"S1")),
        ],
    ) != 0
    {
        writeln!(__out, "Failed to add RG line").unwrap();
    } else {
        let add_pg_ret = sam::sam_hdr_add_pg(
            &mut *in_samhdr,
            b"add_header",
            &[
                (Some(b"VN"), Some(b"Test")),
                (Some(b"CL"), Some(&data.data)),
            ],
        );
        let add_co_ret = if add_pg_ret == 0 {
            sam::sam_hdr_add_line(&mut *in_samhdr, b"CO", &[(Some(b"Test data"), None)])
        } else {
            0
        };
        if add_pg_ret != 0 || add_co_ret != 0 {
            writeln!(__out, "Failed to add PG line").unwrap();
        } else if sam::sam_hdr_write(outfile, in_samhdr) < 0 {
            writeln!(__out, "Failed to write output").unwrap();
        } else {
            ret = 0;
        }
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::hts_close(outfile);
    crate::htslib_rs::hts::ks_free(&mut data);
    __out.flush().unwrap();
    ret
}
