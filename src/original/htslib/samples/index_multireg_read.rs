use std::ffi::{c_char, c_int};

use crate::htslib_mini_rs::{hts, sam};

// original: print_usage (htslib/samples/index_multireg_read.c:37)
pub unsafe fn samples_index_multireg_read_c_37_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_multireg infile count regspec_csv\n    Reads alignment of a target matching to given region specifications\n    read_multireg infile.sam 2 R1:10-100,R2:200".as_ptr(),
    );
}

// original: main (htslib/samples/index_multireg_read.c:50)
pub unsafe fn samples_index_multireg_read_c_50_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut infile = std::ptr::null_mut();
    let mut outfile = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut bamdata = std::ptr::null_mut();
    let mut idx = std::ptr::null_mut();
    let mut iter = std::ptr::null_mut();

    if argc != 4 {
        samples_index_multireg_read_c_37_print_usage(hts_sys::stderr.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let regcnt = libc::atoi(*argv.add(2)) as libc::c_uint;
    let mut regions =
        libc::calloc(regcnt as usize, std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    let mut ptr = *argv.add(3);
    let mut c: c_int = 0;
    while !ptr.is_null() && (c as libc::c_uint) < regcnt {
        *regions.add(c as usize) = ptr;
        ptr = libc::strchr(ptr, b',' as c_int);
        if !ptr.is_null() {
            *ptr = 0;
            ptr = ptr.add(1);
        }
        c += 1;
    }

    if regcnt == 0 {
        libc::printf(c"Region count can not be 0\n".as_ptr());
    } else {
        bamdata = sam::bam_init1();
        if bamdata.is_null() {
            libc::printf(c"Failed to initialize bamdata\n".as_ptr());
        } else {
            infile = hts::hts_open(inname, c"r".as_ptr());
            outfile = hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
            if outfile.is_null() || infile.is_null() {
                libc::printf(c"Could not open in/out files\n".as_ptr());
            } else {
                idx = sam::sam_index_load(infile, inname);
                if idx.is_null() {
                    libc::printf(c"Failed to load the index\n".as_ptr());
                } else {
                    in_samhdr = sam::sam_hdr_read(infile);
                    if in_samhdr.is_null() {
                        libc::printf(c"Failed to read header from file!\n".as_ptr());
                    } else {
                        iter = sam::sam_c_1768_sam_itr_regarray(idx, in_samhdr, regions, regcnt);
                        if iter.is_null() {
                            libc::printf(c"Failed to get iterator\n".as_ptr());
                        } else {
                            libc::free(regions.cast());
                            regions = std::ptr::null_mut();
                            c = hts::hts_itr_multi_next(infile, iter, bamdata.cast());
                            while c >= 0 {
                                if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                                    libc::printf(c"Failed to write output\n".as_ptr());
                                    break;
                                }
                                c = hts::hts_itr_multi_next(infile, iter, bamdata.cast());
                            }
                            if c == -1 {
                                ret = libc::EXIT_SUCCESS;
                            } else {
                                libc::printf(c"Error during read\n".as_ptr());
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
    if !regions.is_null() {
        libc::free(regions.cast());
    }
    ret
}
