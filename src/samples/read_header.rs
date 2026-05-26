use std::ffi::{c_char, c_int};

use crate::htslib_rs::{hts::kstring_t, sam};

// original: print_usage (htslib/samples/read_header.c:37)
pub unsafe fn samples_read_header_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_header infile header [id val] [tag]\nThis shows given tag from given header or the whole line\n".as_ptr(),
    );
}

// original: main (htslib/samples/read_header.c:49)
pub unsafe fn samples_read_header_c_49_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut data = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };

    if argc < 3 || argc > 6 {
        samples_read_header_c_37_print_usage(crate::htslib_rs::c_compat::stderr.cast());
        return ret;
    }

    let inname = *argv.add(1);
    let header = *argv.add(2);
    let mut tag: *mut c_char = std::ptr::null_mut();
    let mut id: *mut c_char = std::ptr::null_mut();
    let mut idval: *mut c_char = std::ptr::null_mut();

    if argc == 4 {
        tag = *argv.add(3);
        if *header == b'H' as c_char && *header.add(1) == b'D' as c_char {
            id = std::ptr::null_mut();
        } else if *header == b'S' as c_char && *header.add(1) == b'Q' as c_char {
            id = c"SN".as_ptr().cast_mut();
        } else if *header == b'R' as c_char && *header.add(1) == b'G' as c_char {
            id = c"ID".as_ptr().cast_mut();
        } else if *header == b'P' as c_char && *header.add(1) == b'G' as c_char {
            id = c"ID".as_ptr().cast_mut();
        } else if *header == b'C' as c_char && *header.add(1) == b'O' as c_char {
            id = c"".as_ptr().cast_mut();
        } else {
            libc::printf(c"Invalid header type\n".as_ptr());
            return ret;
        }
    } else if argc == 5 {
        id = *argv.add(3);
        idval = *argv.add(4);
    } else if argc == 6 {
        id = *argv.add(3);
        idval = *argv.add(4);
        tag = *argv.add(5);
    }

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

    if !id.is_null() && !idval.is_null() {
        ret = if !tag.is_null() {
            sam::sam_hdr_find_tag_id(in_samhdr, header, id, idval, tag, &mut data)
        } else {
            sam::sam_hdr_find_line_id(in_samhdr, header, id, idval, &mut data)
        };

        if ret == 0 {
            libc::printf(c"%s\n".as_ptr(), data.s);
        } else if ret == -1 {
            libc::printf(c"No matching tag found\n".as_ptr());
            ret = libc::EXIT_FAILURE;
        } else {
            libc::printf(c"Failed to find header line\n".as_ptr());
            ret = libc::EXIT_FAILURE;
        }
    } else {
        let linecnt = sam::sam_hdr_count_lines(in_samhdr, header);
        if linecnt == 0 {
            libc::printf(c"No matching line found\n".as_ptr());
        } else {
            let mut ok = true;
            for c in 0..linecnt {
                ret = if !tag.is_null() {
                    sam::sam_hdr_find_tag_pos(in_samhdr, header, c, tag, &mut data)
                } else {
                    sam::sam_hdr_find_line_pos(in_samhdr, header, c, &mut data)
                };

                if ret == 0 {
                    libc::printf(c"%s\n".as_ptr(), data.s);
                } else if ret == -1 {
                    libc::printf(c"Tag not present\n".as_ptr());
                } else {
                    libc::printf(c"Failed to get tag\n".as_ptr());
                    ok = false;
                    break;
                }
            }
            if ok {
                ret = libc::EXIT_SUCCESS;
            } else {
                ret = libc::EXIT_FAILURE;
            }
        }
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::ks_free(&mut data);
    ret
}
