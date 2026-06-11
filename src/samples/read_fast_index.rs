use std::ffi::{c_char, c_int};

use crate::htslib_rs::hts::hts_pos_t;

// original: print_usage (htslib/samples/read_fast_index.c:38)
pub unsafe fn samples_read_fast_index_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: read_fast_i <infile> A/Q 0/1 regiondef\nReads the fasta/fastq file using index and shows the content.\nFor fasta files use A and Q for fastq files.\nRegion can be 1 or more of <reference name>[:start-end] entries separated by comma.\nFor single region, give regcount as 0 and non 0 for multi-regions.\n".as_ptr(),
    );
}

// original: main (htslib/samples/read_fast_index.c:53)
pub unsafe fn samples_read_fast_index_c_53_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    const HTS_PARSE_LIST: c_int = 4;
    let mut ret = libc::EXIT_FAILURE;

    if argc != 5 {
        samples_read_fast_index_c_38_print_usage(crate::htslib_rs::c_compat::stdout.cast());
        return ret;
    }
    let inname = *argv.add(1);
    let fmt = if **argv.add(2) == b'Q' as c_char {
        crate::htslib_rs::faidx::FAI_FASTQ
    } else {
        crate::htslib_rs::faidx::FAI_FASTA
    };
    let usemulti = libc::atoi(*argv.add(3));
    let mut region = *argv.add(4);

    let idx = crate::htslib_rs::faidx::fai_load3_format(
        inname,
        std::ptr::null(),
        std::ptr::null(),
        crate::htslib_rs::faidx::FAI_CREATE,
        fmt,
    );
    if idx.is_null() {
        libc::printf(c"Failed to load index\n".as_ptr());
        return ret;
    }

    if usemulti == 0 {
        let mut len: hts_pos_t = 0;
        let mut data = crate::htslib_rs::faidx::fai_fetch64(idx, region, &mut len);
        if data.is_null() {
            if len == -1 {
                libc::printf(c"Failed to get data\n".as_ptr());
                crate::htslib_rs::faidx::fai_destroy(idx);
                return ret;
            }
            libc::printf(c"Data not found for given region\n".as_ptr());
        } else {
            libc::printf(c"Data: %ld %s\n".as_ptr(), len as libc::c_long, data);
            libc::free(data.cast());
            if fmt == crate::htslib_rs::faidx::FAI_FASTQ {
                data = crate::htslib_rs::faidx::fai_fetchqual64(idx, region, &mut len);
                if data.is_null() {
                    if len == -1 {
                        libc::printf(c"Failed to get data\n".as_ptr());
                        crate::htslib_rs::faidx::fai_destroy(idx);
                        return ret;
                    }
                    libc::printf(c"Data not found for given region\n".as_ptr());
                } else {
                    libc::printf(c"Qual: %ld %s\n".as_ptr(), len as libc::c_long, data);
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
                region,
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_LIST,
            );
            if remaining.is_null() {
                break;
            }
            if crate::htslib_rs::faidx::fai_adjust_region(&*idx, tid, &mut beg, &mut end) == -1 {
                libc::printf(c"Error in adjusting region for tid %d\n".as_ptr(), tid);
                crate::htslib_rs::faidx::fai_destroy(idx);
                return ret;
            }

            let name = crate::htslib_rs::faidx::faidx_iseq(&*idx, tid)
                .map_or(std::ptr::null(), |s| s.as_ptr() as *const c_char);
            let mut len: hts_pos_t = 0;
            let mut data =
                crate::htslib_rs::faidx::faidx_fetch_seq64(idx, name, beg, end, &mut len);
            if data.is_null() {
                if len == -1 {
                    libc::printf(c"Failed to get data\n".as_ptr());
                    crate::htslib_rs::faidx::fai_destroy(idx);
                    return ret;
                }
                libc::printf(c"No data found for given region\n".as_ptr());
            } else {
                libc::printf(c"Data: %ld %s\n".as_ptr(), len as libc::c_long, data);
                libc::free(data.cast());
                if fmt == crate::htslib_rs::faidx::FAI_FASTQ {
                    data =
                        crate::htslib_rs::faidx::faidx_fetch_qual64(idx, name, beg, end, &mut len);
                    if data.is_null() {
                        if len == -1 {
                            libc::printf(c"Failed to get qual data\n".as_ptr());
                            crate::htslib_rs::faidx::fai_destroy(idx);
                            return ret;
                        }
                        libc::printf(c"No data found for given region\n".as_ptr());
                    } else {
                        libc::printf(c"Qual: %ld %s\n".as_ptr(), len as libc::c_long, data);
                        libc::free(data.cast());
                    }
                }
            }
            region = remaining.cast_mut();
        }
    }

    ret = libc::EXIT_SUCCESS;
    crate::htslib_rs::faidx::fai_destroy(idx);
    ret
}
