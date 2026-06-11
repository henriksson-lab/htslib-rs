use std::io::Write;

use crate::htslib_rs::{hts::kstring_t, sam};

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
            write!(__out, "{}", value as i8 as i32).unwrap();
        }
        b'C' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            write!(__out, "{}", value as u8 as u32).unwrap();
        }
        b's' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            write!(__out, "{}", value as i16 as i32).unwrap();
        }
        b'S' => {
            let value = if idx > -1 {
                sam::bam_auxB2i(data, idx as u32)
            } else {
                sam::bam_aux2i(data)
            };
            write!(__out, "{}", value as u16 as u32).unwrap();
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
            let z = sam::bam_aux2Z(data);
            let bytes = std::ffi::CStr::from_ptr(z.cast()).to_bytes();
            write!(__out, "{}", String::from_utf8_lossy(bytes)).unwrap();
        }
        b'B' => {
            let aux_b_count = sam::bam_auxB_len(data);
            let aux_b_type = sam::bam_aux_type(data.add(1)) as u8;
            write!(__out, "{}", aux_b_type as char).unwrap();
            for i in 0..aux_b_count {
                write!(__out, ",").unwrap();
                if printauxdata(__out, aux_b_type, i as i32, data) == 1 {
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

// original: print_usage (htslib/samples/read_aux.c:37)
pub unsafe fn samples_read_aux_c_37_print_usage() {
    eprintln!(
        "Usage: read_aux infile tag\nRead the given aux tag from alignments either as SAM string or as raw data"
    );
}

// original: printauxdata (htslib/samples/read_aux.c:51)
pub unsafe fn samples_read_aux_c_51_printauxdata(
    __out: &mut impl Write,
    type_: u8,
    idx: i32,
    data: *const u8,
) -> i32 {
    printauxdata(__out, type_, idx, data)
}

// original: main (htslib/samples/read_aux.c:114)
pub unsafe fn samples_read_aux_c_114_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut ret = 1;
    let mut __out = std::io::stdout();
    let mut sdata = kstring_t { data: Vec::new() };

    if argc != 3 {
        samples_read_aux_c_37_print_usage();
        return ret;
    }
    let inname = *argv.add(1);
    let tag = *argv.add(2);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        writeln!(__out, "Failed to allocate data memory!").unwrap();
        return ret;
    }
    let infile = crate::htslib_rs::hts::hts_open(inname.cast(), c"r".as_ptr());
    if infile.is_null() {
        let inname_bytes = std::ffi::CStr::from_ptr(inname.cast()).to_bytes();
        writeln!(__out, "Could not open {}", String::from_utf8_lossy(inname_bytes)).unwrap();
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

    let mut i = 0;
    let mut ret_r = sam::sam_read1(infile, in_samhdr, bamdata);
    while ret_r >= 0 {
        *libc::__errno_location() = 0;
        i += 1;
        crate::htslib_rs::hts::ks_clear(&mut sdata);
        if i % 2 != 0 {
            let c = sam::bam_aux_get_str(bamdata, tag.cast(), &mut sdata);
            if c == 1 {
                writeln!(__out, "{}", String::from_utf8_lossy(&sdata.data)).unwrap();
            } else if c == 0 && *libc::__errno_location() == libc::ENOENT {
                writeln!(__out, "Tag not present").unwrap();
            } else {
                writeln!(__out, "Failed to get tag").unwrap();
                sam::sam_hdr_destroy(in_samhdr);
                crate::htslib_rs::hts::hts_close(infile);
                sam::bam_destroy1(bamdata);
                crate::htslib_rs::hts::ks_free(&mut sdata);
                return ret;
            }
        } else {
            let data = sam::bam_aux_get(bamdata, tag.cast());
            if data.is_null() {
                if *libc::__errno_location() == libc::ENOENT {
                    writeln!(__out, "Tag not present").unwrap();
                } else {
                    writeln!(__out, "Invalid aux data").unwrap();
                }
            } else {
                if samples_read_aux_c_51_printauxdata(&mut __out, sam::bam_aux_type(data) as u8, -1, data) == 1 {
                    writeln!(__out, "Failed to read aux data").unwrap();
                    sam::sam_hdr_destroy(in_samhdr);
                    crate::htslib_rs::hts::hts_close(infile);
                    sam::bam_destroy1(bamdata);
                    crate::htslib_rs::hts::ks_free(&mut sdata);
                    return ret;
                }
                writeln!(__out).unwrap();
            }
        }
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
    crate::htslib_rs::hts::ks_free(&mut sdata);
    __out.flush().unwrap();
    ret
}
