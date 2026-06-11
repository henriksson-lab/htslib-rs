use crate::htslib_rs::{hts::kstring_t, sam};
use std::io::Write;

// original: print_usage (htslib/samples/read_header.c:37)
pub unsafe fn samples_read_header_c_37_print_usage() {
    eprint!("Usage: read_header infile header [id val] [tag]\nThis shows given tag from given header or the whole line\n");
}

// original: main (htslib/samples/read_header.c:49)
pub unsafe fn samples_read_header_c_49_main(args: &[Vec<u8>]) -> i32 {
    let mut __out = std::io::stdout();
    let mut ret = 1;
    let mut data = kstring_t { data: Vec::new() };

    let argc = args.len();
    if !(3..=6).contains(&argc) {
        samples_read_header_c_37_print_usage();
        return ret;
    }

    let inname: &[u8] = &args[1];
    let header: &[u8] = &args[2];
    let mut tag: Option<&[u8]> = None;
    let mut id: Option<&[u8]> = None;
    let mut idval: Option<&[u8]> = None;

    if argc == 4 {
        tag = Some(&args[3]);
        if header.first() == Some(&b'H') && header.get(1) == Some(&b'D') {
            id = None;
        } else if header.first() == Some(&b'S') && header.get(1) == Some(&b'Q') {
            id = Some(b"SN");
        } else if (header.first() == Some(&b'R') || header.first() == Some(&b'P'))
            && header.get(1) == Some(&b'G')
        {
            id = Some(b"ID");
        } else if header.first() == Some(&b'C') && header.get(1) == Some(&b'O') {
            id = Some(b"");
        } else {
            write!(__out, "Invalid header type\n").unwrap();
            __out.flush().unwrap();
            return ret;
        }
    } else if argc == 5 {
        id = Some(&args[3]);
        idval = Some(&args[4]);
    } else if argc == 6 {
        id = Some(&args[3]);
        idval = Some(&args[4]);
        tag = Some(&args[5]);
    }

    // hts_open is still a raw-ptr C-ABI API in another file: build NUL-terminated
    // byte buffers and pass them at the boundary.
    let mut inname_c = inname.to_vec();
    inname_c.push(0);
    let infile = crate::htslib_rs::hts::hts_open(inname_c.as_ptr().cast(), b"r\0".as_ptr().cast());
    if infile.is_null() {
        write!(__out, "Could not open {}\n", String::from_utf8_lossy(inname)).unwrap();
        __out.flush().unwrap();
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        write!(__out, "Failed to read header from file!\n").unwrap();
        __out.flush().unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        return ret;
    }

    if let (Some(id), Some(idval)) = (id, idval) {
        ret = if let Some(tag) = tag {
            sam::sam_hdr_find_tag_id(
                &mut *in_samhdr,
                header,
                Some((id, idval)),
                tag,
                &mut data,
            )
        } else {
            sam::sam_hdr_find_line_id(&mut *in_samhdr, header, id, idval, &mut data)
        };

        if ret == 0 {
            write!(__out, "{}\n", String::from_utf8_lossy(&data.data)).unwrap();
        } else if ret == -1 {
            write!(__out, "No matching tag found\n").unwrap();
            ret = 1;
        } else {
            write!(__out, "Failed to find header line\n").unwrap();
            ret = 1;
        }
    } else {
        let linecnt = sam::sam_hdr_count_lines(&mut *in_samhdr, header);
        if linecnt == 0 {
            write!(__out, "No matching line found\n").unwrap();
        } else {
            let mut ok = true;
            for c in 0..linecnt {
                ret = if let Some(tag) = tag {
                    sam::sam_hdr_find_tag_pos(&mut *in_samhdr, header, c, tag, &mut data)
                } else {
                    sam::sam_hdr_find_line_pos(&mut *in_samhdr, header, c, &mut data)
                };

                if ret == 0 {
                    write!(__out, "{}\n", String::from_utf8_lossy(&data.data)).unwrap();
                } else if ret == -1 {
                    write!(__out, "Tag not present\n").unwrap();
                } else {
                    write!(__out, "Failed to get tag\n").unwrap();
                    ok = false;
                    break;
                }
            }
            if ok {
                ret = 0;
            } else {
                ret = 1;
            }
        }
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::ks_free(&mut data);
    __out.flush().unwrap();
    ret
}
