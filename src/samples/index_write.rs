use crate::htslib_rs::{hts, sam};
use std::io::Write;

// original: print_usage (htslib/samples/index_write.c:38)
pub unsafe fn samples_index_write_c_38_print_usage() {
    eprint!("Usage: idx_on_write infile shiftsize outdir\nCreates compressed sam file and index file for it in given directory\n");
}

// original: main (htslib/samples/index_write.c:50)
pub unsafe fn samples_index_write_c_50_main(args: &[Vec<u8>]) -> i32 {
    let mut __out = std::io::stdout();
    let mut outmode = [b'w', 0, 0, 0];
    let mut ret = 1;
    let mut outfile = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();

    if args.len() != 4 {
        samples_index_write_c_38_print_usage();
        return ret;
    }

    // argv entries are NUL-terminated C strings; use bytes up to the first NUL.
    let inname = &args[1][..args[1].iter().position(|&b| b == 0).unwrap_or(args[1].len())];
    let size = std::str::from_utf8(&args[2][..args[2].iter().position(|&b| b == 0).unwrap_or(args[2].len())])
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let outdir = &args[3][..args[3].iter().position(|&b| b == 0).unwrap_or(args[3].len())];
    let basename = match inname.iter().rposition(|&b| b == b'/') {
        None => &inname[..],
        Some(pos) => &inname[pos + 1..],
    };

    let mut fileidx: Vec<u8> = Vec::new();
    let mut outname: Vec<u8> = Vec::new();

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
        __out.flush().unwrap();
        return ret;
    }

    // hts_open still expects raw C strings; build NUL-terminated byte buffers.
    let mut inname_c = inname.to_vec();
    inname_c.push(0);
    let infile = hts::hts_open(inname_c.as_ptr().cast(), c"r".as_ptr());
    if !infile.is_null() {
        if (*infile).format.format == hts::HTS_FORMAT_CRAM {
            fileidx = outdir.to_vec();
            fileidx.extend_from_slice(b"/");
            fileidx.extend_from_slice(basename);
            fileidx.extend_from_slice(b".crai");
            outname = outdir.to_vec();
            outname.extend_from_slice(b"/");
            outname.extend_from_slice(basename);
        } else if (*infile).format.format == hts::HTS_FORMAT_SAM
            && (*infile).format.compression == hts::HTS_COMPRESSION_NO_COMPRESSION
        {
            outname = outdir.to_vec();
            outname.extend_from_slice(b"/");
            outname.extend_from_slice(basename);
            outname.extend_from_slice(b".gz");
            fileidx = outdir.to_vec();
            fileidx.extend_from_slice(b"/");
            fileidx.extend_from_slice(basename);
            fileidx.extend_from_slice(b".gz.");
            fileidx.extend_from_slice(if size == 0 { b"bai" } else { b"csi" });
        } else {
            outname = outdir.to_vec();
            outname.extend_from_slice(b"/");
            outname.extend_from_slice(basename);
            fileidx = outdir.to_vec();
            fileidx.extend_from_slice(b"/");
            fileidx.extend_from_slice(basename);
            fileidx.extend_from_slice(b".");
            fileidx.extend_from_slice(if size == 0 { b"bai" } else { b"csi" });
        }
    }

    if !outname.is_empty() {
        // sam_open_mode / hts_open still expect raw C strings; NUL-terminate.
        let mut outname_c = outname.clone();
        outname_c.push(0);
        sam::sam_open_mode(
            outmode.as_mut_ptr().add(1).cast(),
            outname_c.as_ptr().cast(),
            std::ptr::null(),
        );
        outfile = hts::hts_open(outname_c.as_ptr().cast(), outmode.as_ptr().cast());
    }
    if outfile.is_null() || infile.is_null() {
        write!(__out, "Could not open files\n").unwrap();
    } else {
        in_samhdr = sam::sam_hdr_read(infile);
        let mut fileidx_c = fileidx.clone();
        fileidx_c.push(0);
        if in_samhdr.is_null() {
            write!(__out, "Failed to read header from file!\n").unwrap();
        } else if sam::sam_hdr_write(outfile, in_samhdr) != 0 {
            write!(__out, "Failed to write header\n").unwrap();
        } else if sam::sam_idx_init(outfile, in_samhdr, size, fileidx_c.as_ptr().cast()) != 0 {
            write!(__out, "idx initialization failed\n").unwrap();
        } else {
            let mut c = sam::sam_read1(infile, in_samhdr, bamdata);
            while c >= 0 {
                if sam::sam_c_4553_sam_write1(outfile, in_samhdr, bamdata) < 0 {
                    write!(__out, "Failed to write data\n").unwrap();
                    break;
                }
                c = sam::sam_read1(infile, in_samhdr, bamdata);
            }
            if c == -1 {
                if sam::sam_idx_save(outfile) != 0 {
                    write!(__out, "Could not save index\n").unwrap();
                } else {
                    ret = 0;
                }
            } else if c < -1 {
                write!(__out, "Error in reading data\n").unwrap();
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
    if !outfile.is_null() {
        hts::hts_close(outfile);
    }
    __out.flush().unwrap();
    ret
}
