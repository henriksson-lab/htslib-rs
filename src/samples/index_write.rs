use std::ffi::{c_char, c_int};

use crate::htslib_rs::{hts, sam};

// original: print_usage (htslib/samples/index_write.c:38)
pub unsafe fn samples_index_write_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: idx_on_write infile shiftsize outdir\nCreates compressed sam file and index file for it in given directory\n".as_ptr(),
    );
}

// original: main (htslib/samples/index_write.c:50)
pub unsafe fn samples_index_write_c_50_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut outmode = [b'w' as c_char, 0, 0, 0];
    let mut ret = libc::EXIT_FAILURE;
    let mut outfile = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();

    if argc != 4 {
        samples_index_write_c_38_print_usage(hts_sys::stderr.cast());
        return ret;
    }

    let inname = *argv.add(1);
    let size = libc::atoi(*argv.add(2));
    let outdir = *argv.add(3);
    let slash = libc::strrchr(inname, b'/' as c_int);
    let basename = if slash.is_null() {
        inname
    } else {
        slash.add(1)
    };
    let mut c = (libc::strlen(basename) + libc::strlen(outdir) + 10) as c_int;
    let fileidx = libc::malloc(c as usize).cast::<c_char>();
    let outname = libc::malloc(c as usize).cast::<c_char>();
    if fileidx.is_null() || outname.is_null() {
        libc::printf(c"Couldnt allocate memory\n".as_ptr());
        if !fileidx.is_null() {
            libc::free(fileidx.cast());
        }
        if !outname.is_null() {
            libc::free(outname.cast());
        }
        return ret;
    }

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to initialize bamdata\n".as_ptr());
        libc::free(fileidx.cast());
        libc::free(outname.cast());
        return ret;
    }

    let infile = hts::hts_open(inname, c"r".as_ptr());
    if !infile.is_null() {
        if (*infile).format.format == hts::HTS_FORMAT_CRAM {
            libc::snprintf(
                fileidx,
                c as usize,
                c"%s/%s.crai".as_ptr(),
                outdir,
                basename,
            );
            libc::snprintf(outname, c as usize, c"%s/%s".as_ptr(), outdir, basename);
        } else if (*infile).format.format == hts::HTS_FORMAT_SAM
            && (*infile).format.compression == hts::HTS_COMPRESSION_NO_COMPRESSION
        {
            libc::snprintf(outname, c as usize, c"%s/%s.gz".as_ptr(), outdir, basename);
            libc::snprintf(
                fileidx,
                c as usize,
                c"%s/%s.gz.%s".as_ptr(),
                outdir,
                basename,
                if size == 0 {
                    c"bai".as_ptr()
                } else {
                    c"csi".as_ptr()
                },
            );
        } else {
            libc::snprintf(outname, c as usize, c"%s/%s".as_ptr(), outdir, basename);
            libc::snprintf(
                fileidx,
                c as usize,
                c"%s/%s.%s".as_ptr(),
                outdir,
                basename,
                if size == 0 {
                    c"bai".as_ptr()
                } else {
                    c"csi".as_ptr()
                },
            );
        }
    }

    if !outname.is_null() {
        sam::sam_open_mode(outmode.as_mut_ptr().add(1), outname, std::ptr::null());
        outfile = hts::hts_open(outname, outmode.as_ptr());
    }
    if outfile.is_null() || infile.is_null() {
        libc::printf(c"Could not open files\n".as_ptr());
    } else {
        in_samhdr = sam::sam_hdr_read(infile);
        if in_samhdr.is_null() {
            libc::printf(c"Failed to read header from file!\n".as_ptr());
        } else if sam::sam_hdr_write(outfile, in_samhdr) != 0 {
            libc::printf(c"Failed to write header\n".as_ptr());
        } else if sam::sam_idx_init(outfile, in_samhdr, size, fileidx) != 0 {
            libc::printf(c"idx initialization failed\n".as_ptr());
        } else {
            c = sam::sam_read1(infile, in_samhdr, bamdata);
            while c >= 0 {
                if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                    libc::printf(c"Failed to write data\n".as_ptr());
                    break;
                }
                c = sam::sam_read1(infile, in_samhdr, bamdata);
            }
            if c == -1 {
                if sam::sam_idx_save(outfile) != 0 {
                    libc::printf(c"Could not save index\n".as_ptr());
                } else {
                    ret = libc::EXIT_SUCCESS;
                }
            } else if c < -1 {
                libc::printf(c"Error in reading data\n".as_ptr());
            }
        }
    }

    if !in_samhdr.is_null() {
        sam::sam_hdr_destroy(in_samhdr);
    }
    if !infile.is_null() {
        hts::hts_close(infile);
    }
    if !bamdata.is_null() {
        sam::bam_destroy1(bamdata);
    }
    if !fileidx.is_null() {
        libc::free(fileidx.cast());
    }
    if !outname.is_null() {
        libc::free(outname.cast());
    }
    if !outfile.is_null() {
        hts::hts_close(outfile);
    }
    ret
}
