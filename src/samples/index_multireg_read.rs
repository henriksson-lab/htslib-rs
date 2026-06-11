use crate::htslib_rs::{hts, sam};
use std::io::Write;

// original: print_usage (htslib/samples/index_multireg_read.c:37)
pub unsafe fn samples_index_multireg_read_c_37_print_usage() {
    eprint!(
        "Usage: read_multireg infile count regspec_csv\n    Reads alignment of a target matching to given region specifications\n    read_multireg infile.sam 2 R1:10-100,R2:200"
    );
}

// original: main (htslib/samples/index_multireg_read.c:50)
pub unsafe fn samples_index_multireg_read_c_50_main(args: &[Vec<u8>]) -> i32 {
    let mut __out = std::io::stdout();
    let mut ret = 1;
    let mut infile = std::ptr::null_mut();
    let mut outfile = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut bamdata = std::ptr::null_mut();
    let mut idx = std::ptr::null_mut();
    let mut iter = std::ptr::null_mut();

    if args.len() != 4 {
        samples_index_multireg_read_c_37_print_usage();
        return ret;
    }
    // NUL-terminated copy for the raw-ptr hts/sam API boundary.
    let mut inname: Vec<u8> = args[1].clone();
    inname.push(0);
    // argv entries are NUL-terminated C strings; use bytes up to the first NUL.
    let regcnt: u32 = String::from_utf8_lossy(
        &args[2][..args[2].iter().position(|&b| b == 0).unwrap_or(args[2].len())],
    )
    .parse()
    .unwrap_or(0);

    // Split the comma-separated region spec into up to regcnt NUL-terminated
    // byte strings, then collect raw pointers for the raw-ptr sam_itr_regarray API.
    let mut region_bufs: Vec<Vec<u8>> = Vec::new();
    for field in args[3].split(|&b| b == b',') {
        if region_bufs.len() as u32 >= regcnt {
            break;
        }
        let mut buf = field.to_vec();
        buf.push(0);
        region_bufs.push(buf);
    }
    let mut regions: Vec<*const u8> = region_bufs.iter().map(|b| b.as_ptr()).collect();

    if regcnt == 0 {
        write!(__out, "Region count can not be 0\n").unwrap();
    } else {
        bamdata = sam::bam_init1();
        if bamdata.is_null() {
            write!(__out, "Failed to initialize bamdata\n").unwrap();
        } else {
            infile = hts::hts_open(inname.as_ptr().cast(), c"r".as_ptr());
            outfile = hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
            if outfile.is_null() || infile.is_null() {
                write!(__out, "Could not open in/out files\n").unwrap();
            } else {
                idx = sam::sam_index_load(infile, inname.as_ptr().cast());
                if idx.is_null() {
                    write!(__out, "Failed to load the index\n").unwrap();
                } else {
                    in_samhdr = sam::sam_hdr_read(infile);
                    if in_samhdr.is_null() {
                        write!(__out, "Failed to read header from file!\n").unwrap();
                    } else {
                        iter = sam::sam_c_1768_sam_itr_regarray(
                            idx,
                            in_samhdr,
                            regions.as_mut_ptr().cast(),
                            regcnt,
                        );
                        if iter.is_null() {
                            write!(__out, "Failed to get iterator\n").unwrap();
                        } else {
                            region_bufs.clear();
                            regions.clear();
                            let mut c = (hts::hts_itr_multi_next(infile, iter, bamdata.cast()) >= 0)
                                as i32;
                            while c != 0 {
                                if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                                    write!(__out, "Failed to write output\n").unwrap();
                                    break;
                                }
                                c = (hts::hts_itr_multi_next(infile, iter, bamdata.cast()) >= 0)
                                    as i32;
                            }
                            if c == -1 {
                                ret = 0;
                            } else {
                                // Records were written to `outfile` (stdout) via its
                                // hFILE buffer; flush that buffer so this diagnostic,
                                // written through Rust's stdout, lands after the records.
                                hts::hts_flush(outfile);
                                let out_hfile = hts::hts_hfile(outfile);
                                if !out_hfile.is_null() {
                                    crate::htslib_rs::hfile::hflush(out_hfile);
                                }
                                write!(__out, "Error during read\n").unwrap();
                                __out.flush().unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    if !in_samhdr.is_null() {
        sam::sam_hdr_destroy(in_samhdr);
    }
    if !infile.is_null() {
        hts::hts_close(infile);
    }
    if !outfile.is_null() {
        hts::hts_close(outfile);
    }
    if !bamdata.is_null() {
        sam::bam_destroy1(bamdata);
    }
    if !iter.is_null() {
        hts::hts_itr_destroy(iter);
    }
    if !idx.is_null() {
        hts::hts_idx_destroy(idx);
    }
    __out.flush().unwrap();
    ret
}
