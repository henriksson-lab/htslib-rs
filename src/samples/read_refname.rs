use crate::htslib_rs::{hts::kstring_t, sam};
use std::io::Write;

// original: print_usage (htslib/samples/read_refname.c:37)
pub unsafe fn samples_read_refname_c_37_print_usage() {
    let mut __out = std::io::stdout();
    write!(
        __out,
        "Usage: read_refname infile minsize\nThis shows name of references which has length above the given size\n"
    ).unwrap();
}

// original: main (htslib/samples/read_refname.c:49)
pub unsafe fn samples_read_refname_c_49_main(argv: &[&[u8]]) -> i32 {
    let mut ret = 1;
    let mut data = kstring_t { data: Vec::new() };
    let mut __out = std::io::stdout();

    let argc = argv.len() as i32;
    if argc != 3 && argc != 2 {
        samples_read_refname_c_37_print_usage();
        return ret;
    }
    let inname = argv[1];
    let minsize: i64 = if argc == 3 {
        // argv[2] is ASCII digits; parse as integer
        std::str::from_utf8(argv[2]).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0)
    } else {
        0
    };

    // hts_open is a raw C-ABI callee: pass a NUL-terminated byte buffer.
    let mut inname_c = inname.to_vec();
    inname_c.push(0);
    let infile = crate::htslib_rs::hts::hts_open(inname_c.as_ptr().cast(), c"r".as_ptr());
    if infile.is_null() {
        eprintln!("Could not open {}", String::from_utf8_lossy(inname));
        return ret;
    }
    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        eprintln!("Failed to read header from file!");
        crate::htslib_rs::hts::hts_close(infile);
        return ret;
    }

    let linecnt = sam::sam_hdr_count_lines(&mut *in_samhdr, b"SQ");
    if linecnt <= 0 {
        if linecnt == 0 {
            eprintln!("No reference line present");
        } else {
            eprintln!("Failed to get reference line count");
        }
        sam::sam_hdr_destroy(in_samhdr);
        crate::htslib_rs::hts::hts_close(infile);
        return ret;
    }

    let mut pos = 1;
    for c_idx in 0..linecnt {
        let found =
            sam::sam_hdr_find_tag_pos(&mut *in_samhdr, b"SQ", c_idx, b"LN", &mut data);
        if found == -2 {
            eprintln!("Failed to get length");
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            crate::htslib_rs::hts::ks_free(&mut data);
            return ret;
        } else if found == -1 {
            continue;
        }

        let size: i64 = std::str::from_utf8(&data.data)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        if size < minsize {
            continue;
        }
        let id = sam::sam_hdr_line_name(&mut *in_samhdr, b"SQ", c_idx);
        if id.is_null() {
            eprintln!("Failed to get id for reference data");
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            crate::htslib_rs::hts::ks_free(&mut data);
            return ret;
        }
        // id is a NUL-terminated C string returned by the raw-ptr API.
        let id_bytes = std::ffi::CStr::from_ptr(id.cast()).to_bytes();
        writeln!(
            __out,
            "{},{},{}",
            pos,
            String::from_utf8_lossy(id_bytes),
            String::from_utf8_lossy(&data.data)
        ).unwrap();
        pos += 1;
    }

    ret = 0;
    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    crate::htslib_rs::hts::ks_free(&mut data);
    __out.flush().unwrap();
    ret
}
