use std::ffi::{c_char, c_int};

use crate::htslib_rs::{hts::kstring_t, sam};

// original: print_usage (htslib/samples/add_header.c:37)
pub unsafe fn samples_add_header_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: add_header infile\nAdds new header lines of SQ, RG, PG and CO types\n".as_ptr(),
    );
}

// original: main (htslib/samples/add_header.c:49)
pub unsafe fn samples_add_header_c_49_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let sq = c"@SQ\tSN:TR1\tLN:100\n@SQ\tSN:TR2\tLN:50\n";
    let mut data = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    if argc != 2 {
        samples_add_header_c_37_print_usage(crate::htslib_rs::c_compat::stderr.cast());
        crate::htslib_rs::hts::ks_free(&mut data);
        return ret;
    }

    let inname = *argv.add(1);
    let infile = crate::htslib_rs::hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), inname);
        crate::htslib_rs::hts::ks_free(&mut data);
        return ret;
    }
    let outfile = crate::htslib_rs::hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
    if outfile.is_null() {
        libc::printf(c"Could not open stdout\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::ks_free(&mut data);
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file!\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        crate::htslib_rs::hts::hts_close(outfile);
        crate::htslib_rs::hts::ks_free(&mut data);
        return ret;
    }

    for i in 0..argc {
        crate::htslib_rs::hts::kputs(*argv.add(i as usize), &mut data);
        crate::htslib_rs::hts::kputc(b' ' as c_int, &mut data);
    }

    if sam::sam_hdr_add_lines(in_samhdr, sq.as_ptr(), 0) != 0 {
        libc::printf(c"Failed to add SQ lines\n".as_ptr());
    } else if sam::sam_hdr_add_line(
        in_samhdr,
        c"RG".as_ptr(),
        &[
            (c"ID".as_ptr(), c"RG1".as_ptr()),
            (c"LB".as_ptr(), c"Test".as_ptr()),
            (c"SM".as_ptr(), c"S1".as_ptr()),
        ],
    ) != 0
    {
        libc::printf(c"Failed to add RG line\n".as_ptr());
    } else {
        let add_pg_ret = sam::sam_hdr_add_pg(
            in_samhdr,
            c"add_header".as_ptr(),
            &[(c"VN".as_ptr(), c"Test".as_ptr()), (c"CL".as_ptr(), data.s)],
        );
        let add_co_ret = if add_pg_ret == 0 {
            sam::sam_hdr_add_line(
                in_samhdr,
                c"CO".as_ptr(),
                &[(c"Test data".as_ptr(), std::ptr::null::<c_char>())],
            )
        } else {
            0
        };
        if add_pg_ret != 0 || add_co_ret != 0 {
            libc::printf(c"Failed to add PG line\n".as_ptr());
        } else if sam::sam_hdr_write(outfile, in_samhdr) < 0 {
            libc::printf(c"Failed to write output\n".as_ptr());
        } else {
            ret = libc::EXIT_SUCCESS;
        }
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::hts_close(outfile);
    crate::htslib_rs::hts::ks_free(&mut data);
    ret
}
