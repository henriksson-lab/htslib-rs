use std::ffi::{c_char, c_int};

use crate::htslib_rs::{hts::kstring_t, sam};

// original: print_usage (htslib/samples/read_refname.c:37)
pub unsafe fn samples_read_refname_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_refname infile minsize\nThis shows name of references which has length above the given size\n".as_ptr(),
    );
}

// original: main (htslib/samples/read_refname.c:49)
pub unsafe fn samples_read_refname_c_49_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut data = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };

    if argc != 3 && argc != 2 {
        samples_read_refname_c_37_print_usage(hts_sys::stdout.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let minsize = if argc == 3 {
        libc::atoll(*argv.add(2))
    } else {
        0
    };

    let infile = crate::htslib_rs::hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), inname);
        return ret;
    }
    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file!\n".as_ptr());
        crate::htslib_rs::hts::hts_close(infile);
        return ret;
    }

    let linecnt = sam::sam_hdr_count_lines(in_samhdr, c"SQ".as_ptr());
    if linecnt <= 0 {
        if linecnt == 0 {
            libc::printf(c"No reference line present\n".as_ptr());
        } else {
            libc::printf(c"Failed to get reference line count\n".as_ptr());
        }
        sam::sam_hdr_destroy(in_samhdr);
        crate::htslib_rs::hts::hts_close(infile);
        return ret;
    }

    let mut pos = 1;
    for c_idx in 0..linecnt {
        let found =
            sam::sam_hdr_find_tag_pos(in_samhdr, c"SQ".as_ptr(), c_idx, c"LN".as_ptr(), &mut data);
        if found == -2 {
            libc::printf(c"Failed to get length\n".as_ptr());
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            crate::htslib_rs::hts::ks_free(&mut data);
            return ret;
        } else if found == -1 {
            continue;
        }

        let size = libc::atoll(data.s);
        if size < minsize {
            continue;
        }
        let id = sam::sam_hdr_line_name(in_samhdr, c"SQ".as_ptr(), c_idx);
        if id.is_null() {
            libc::printf(c"Failed to get id for reference data\n".as_ptr());
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            crate::htslib_rs::hts::ks_free(&mut data);
            return ret;
        }
        libc::printf(c"%d,%s,%s\n".as_ptr(), pos, id, data.s);
        pos += 1;
    }

    ret = libc::EXIT_SUCCESS;
    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::ks_free(&mut data);
    ret
}
