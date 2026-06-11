use std::ffi::c_char;
use std::io::Write;

use crate::htslib_rs::sam;

unsafe fn printauxdata(__out: &mut impl Write, type_: u8, idx: i32, data: *const u8) -> i32 {
    match type_ {
        b'A' => {
            write!(__out, "{}", sam::bam_aux2A(data) as u8 as char).unwrap();
        }
        b'c' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            write!(__out, "{}", value as i8).unwrap();
        }
        b'C' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            write!(__out, "{}", value as u8).unwrap();
        }
        b's' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            write!(__out, "{}", value as i16).unwrap();
        }
        b'S' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            write!(__out, "{}", value as u16).unwrap();
        }
        b'i' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            write!(__out, "{}", value as i32).unwrap();
        }
        b'I' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            write!(__out, "{}", value as u32).unwrap();
        }
        b'f' | b'd' => {
            let value = if idx > -1 {
                sam::bam_auxB2f(data, idx as u32)
            } else {
                sam::bam_aux2f(data)
            };
            write!(__out, "{}", value as f32 as f64).unwrap();
        }
        b'H' | b'Z' => {
            // bam_aux2Z returns a raw C-string pointer (parallel raw API boundary)
            let z = std::ffi::CStr::from_ptr(sam::bam_aux2Z(data).cast::<c_char>()).to_bytes();
            write!(__out, "{}", String::from_utf8_lossy(z)).unwrap();
        }
        b'B' => {
            let aux_b_count = sam::bam_auxB_len(data);
            let aux_b_type = sam::bam_aux_type(data.add(1));
            write!(__out, "{}", aux_b_type as u8 as char).unwrap();
            for i in 0..aux_b_count {
                write!(__out, ",").unwrap();
                if printauxdata(__out, aux_b_type as u8, i as i32, data) == 1 {
                    return 1;
                }
            }
        }
        _ => {
            writeln!(__out, "Invalid aux tag?").unwrap();
            return 1;
        }
    }
    0
}

// original: print_usage (htslib/samples/dump_aux.c:37)
pub unsafe fn samples_dump_aux_c_37_print_usage() {
    eprint!("Usage: dump_aux infile\nDump the aux tags from alignments\n");
}

// original: printauxdata (htslib/samples/dump_aux.c:51)
pub unsafe fn samples_dump_aux_c_51_printauxdata(
    __out: &mut impl Write,
    type_: u8,
    idx: i32,
    data: *const u8,
) -> i32 {
    printauxdata(__out, type_, idx, data)
}

// original: main (htslib/samples/dump_aux.c:114)
pub unsafe fn samples_dump_aux_c_114_main(args: &[Vec<u8>]) -> i32 {
    let mut ret: i32 = 1;
    let mut __out = std::io::stdout();

    if args.len() != 2 {
        samples_dump_aux_c_37_print_usage();
        return ret;
    }
    // inname is forwarded to hts_open, a raw-ptr C-ABI API expecting *const c_char
    let inname = args[1].as_ptr().cast::<c_char>();

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        writeln!(__out, "Failed to allocate data memory!").unwrap();
        return ret;
    }

    let infile = crate::htslib_rs::hts::hts_open(inname, c"r".as_ptr());
    if infile.is_null() {
        writeln!(
            __out,
            "Could not open {}",
            String::from_utf8_lossy(&args[1])
        ).unwrap();
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let in_samhdr = sam::sam_hdr_read(infile);
    if in_samhdr.is_null() {
        writeln!(__out, "Failed to read header from file!").unwrap();
        crate::htslib_rs::hts::hts_close(infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        *crate::htslib_rs::c_compat::__errno_location() = 0;
        let mut data = sam::bam_aux_first(bamdata);
        while !data.is_null() {
            let type_ = sam::bam_aux_type(data) as u8;
            // bam_aux_tag returns a raw 2-byte tag pointer (parallel raw API boundary)
            let tag = std::slice::from_raw_parts(sam::bam_aux_tag(data).cast::<u8>(), 2);
            let displayed = if b"cCsSiI".contains(&type_) {
                b'i'
            } else {
                type_
            };
            write!(
                __out,
                "{}{}:{}:",
                tag[0] as char, tag[1] as char, displayed as char
            ).unwrap();
            if samples_dump_aux_c_51_printauxdata(&mut __out, type_, -1, data) == 1 {
                writeln!(__out, "Failed to dump aux data").unwrap();
                sam::sam_hdr_destroy(in_samhdr);
                crate::htslib_rs::hts::hts_close(infile);
                sam::bam_destroy1(bamdata);
                return ret;
            }
            write!(__out, " ").unwrap();
            data = sam::bam_aux_next(bamdata, data);
        }
        if *crate::htslib_rs::c_compat::__errno_location()
            != crate::htslib_rs::c_compat::ENOENT
        {
            writeln!(__out, "\nFailed to get aux data").unwrap();
            sam::sam_hdr_destroy(in_samhdr);
            crate::htslib_rs::hts::hts_close(infile);
            sam::bam_destroy1(bamdata);
            return ret;
        }
        writeln!(__out).unwrap();
        ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    }

    if ret_r < -1 {
        writeln!(__out, "Failed to read data").unwrap();
    } else {
        ret = 0;
    }

    sam::sam_hdr_destroy(in_samhdr);
    crate::htslib_rs::hts::hts_close(infile);
    sam::bam_destroy1(bamdata);
    __out.flush().unwrap();
    ret
}
