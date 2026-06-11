use crate::htslib_rs::hts::hts_pos_t;
use std::io::Write;

// original: print_usage (htslib/samples/read_fast_index.c:38)
pub fn samples_read_fast_index_c_38_print_usage() {
    eprint!(
        "Usage: read_fast_i <infile> A/Q 0/1 regiondef\nReads the fasta/fastq file using index and shows the content.\nFor fasta files use A and Q for fastq files.\nRegion can be 1 or more of <reference name>[:start-end] entries separated by comma.\nFor single region, give regcount as 0 and non 0 for multi-regions.\n"
    );
}

// original: main (htslib/samples/read_fast_index.c:53)
pub unsafe fn samples_read_fast_index_c_53_main(argc: i32, argv: *mut *mut u8) -> i32 {
    const HTS_PARSE_LIST: i32 = 4;
    let mut ret = 1;
    let mut __out = std::io::stdout();

    if argc != 5 {
        samples_read_fast_index_c_38_print_usage();
        return ret;
    }
    let inname = *argv.add(1);
    let fmt = if **argv.add(2) == b'Q' {
        crate::htslib_rs::faidx::FAI_FASTQ
    } else {
        crate::htslib_rs::faidx::FAI_FASTA
    };
    // boundary: argv string still passed to raw-ptr atoi; parse the digit run here
    let usemulti = libc::atoi(*argv.add(3).cast());
    let mut region = *argv.add(4);

    let idx = crate::htslib_rs::faidx::fai_load3_format(
        inname.cast(),
        std::ptr::null(),
        std::ptr::null(),
        crate::htslib_rs::faidx::FAI_CREATE,
        fmt,
    );
    if idx.is_null() {
        writeln!(__out, "Failed to load index").unwrap();
        __out.flush().unwrap();
        return ret;
    }

    if usemulti == 0 {
        let mut len: hts_pos_t = 0;
        // boundary: faidx returns a malloc'd C string (raw ptr)
        let mut data = crate::htslib_rs::faidx::fai_fetch64(idx, region.cast(), &mut len);
        if data.is_null() {
            if len == -1 {
                writeln!(__out, "Failed to get data").unwrap();
                __out.flush().unwrap();
                crate::htslib_rs::faidx::fai_destroy(idx);
                return ret;
            }
            writeln!(__out, "Data not found for given region").unwrap();
        } else {
            writeln!(
                __out,
                "Data: {} {}",
                len,
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(data.cast()).to_bytes())
            ).unwrap();
            libc::free(data.cast());
            if fmt == crate::htslib_rs::faidx::FAI_FASTQ {
                data = crate::htslib_rs::faidx::fai_fetchqual64(idx, region.cast(), &mut len);
                if data.is_null() {
                    if len == -1 {
                        writeln!(__out, "Failed to get data").unwrap();
                        __out.flush().unwrap();
                        crate::htslib_rs::faidx::fai_destroy(idx);
                        return ret;
                    }
                    writeln!(__out, "Data not found for given region").unwrap();
                } else {
                    writeln!(
                        __out,
                        "Qual: {} {}",
                        len,
                        String::from_utf8_lossy(std::ffi::CStr::from_ptr(data.cast()).to_bytes())
                    ).unwrap();
                    libc::free(data.cast());
                }
            }
        }
    } else {
        loop {
            let mut tid = -1;
            let mut beg: hts_pos_t = 0;
            let mut end: hts_pos_t = 0;
            let remaining = crate::htslib_rs::faidx::fai_parse_region(
                idx,
                region.cast(),
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_LIST,
            );
            if remaining.is_null() {
                break;
            }
            if crate::htslib_rs::faidx::fai_adjust_region(&*idx, tid, &mut beg, &mut end) == -1 {
                writeln!(__out, "Error in adjusting region for tid {}", tid).unwrap();
                __out.flush().unwrap();
                crate::htslib_rs::faidx::fai_destroy(idx);
                return ret;
            }

            let name = crate::htslib_rs::faidx::faidx_iseq(&*idx, tid)
                .map_or(std::ptr::null(), |s| s.as_ptr() as *const u8);
            let mut len: hts_pos_t = 0;
            let mut data =
                crate::htslib_rs::faidx::faidx_fetch_seq64(idx, name.cast(), beg, end, &mut len);
            if data.is_null() {
                if len == -1 {
                    writeln!(__out, "Failed to get data").unwrap();
                    __out.flush().unwrap();
                    crate::htslib_rs::faidx::fai_destroy(idx);
                    return ret;
                }
                writeln!(__out, "No data found for given region").unwrap();
            } else {
                writeln!(
                    __out,
                    "Data: {} {}",
                    len,
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr(data.cast()).to_bytes())
                ).unwrap();
                libc::free(data.cast());
                if fmt == crate::htslib_rs::faidx::FAI_FASTQ {
                    data = crate::htslib_rs::faidx::faidx_fetch_qual64(
                        idx,
                        name.cast(),
                        beg,
                        end,
                        &mut len,
                    );
                    if data.is_null() {
                        if len == -1 {
                            writeln!(__out, "Failed to get qual data").unwrap();
                            __out.flush().unwrap();
                            crate::htslib_rs::faidx::fai_destroy(idx);
                            return ret;
                        }
                        writeln!(__out, "No data found for given region").unwrap();
                    } else {
                        writeln!(
                            __out,
                            "Qual: {} {}",
                            len,
                            String::from_utf8_lossy(
                                std::ffi::CStr::from_ptr(data.cast()).to_bytes()
                            )
                        ).unwrap();
                        libc::free(data.cast());
                    }
                }
            }
            region = remaining.cast_mut().cast();
        }
    }

    ret = 0;
    crate::htslib_rs::faidx::fai_destroy(idx);
    __out.flush().unwrap();
    ret
}
