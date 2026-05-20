/*
    Copyright (C) 2017, 2020, 2023, 2025 Genome Research Ltd.

    Author: Petr Danecek <pd3@sanger.ac.uk>

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
    THE SOFTWARE.
*/

/*
    Test bcf synced reader allele pairing
*/

use crate::htslib_mini_rs::{
    hts::{htsExactFormat, hts_readlist},
    vcf::{bcf_hdr_t, bcf_seqname_safe, bcf_srs_t},
};
use std::ffi::{c_char, c_int};

macro_rules! error {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        libc::fprintf(hts_sys::stderr.cast(), $fmt.as_ptr() $(, $arg)*);
        libc::exit(libc::EXIT_FAILURE);
    }};
}

// original: usage (htslib/test/test-bcf-sr.c:54)
pub unsafe fn test_test_bcf_sr_c_54_usage(exit_code: c_int) -> ! {
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"Usage: test-bcf-sr [OPTIONS] vcf-list.txt\n".as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"       test-bcf-sr [OPTIONS] --args file1.bcf [...]\n".as_ptr(),
    );
    libc::fprintf(hts_sys::stderr.cast(), c"Options:\n".as_ptr());
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"       --args                   pass filenames directly in argument list\n".as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"       --no-index               allow streaming\n".as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"   -o, --output <file>          output file (stdout if not set)\n".as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"   -O, --output-fmt <fmt>       fmt: vcf,bcf,summary\n".as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"   -p, --pair <logic[+ref]>     logic: snps,indels,both,snps+ref,indels+ref,both+ref,exact,some,all\n"
            .as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"   -r, --regions <reg_list>     comma-separated list of regions\n".as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"   -t, --targets <reg_list>     comma-separated list of targets\n".as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"   -u, --usefptr                use hfile pointer interface on reader addition\n"
            .as_ptr(),
    );
    libc::fprintf(hts_sys::stderr.cast(), c"\n".as_ptr());
    libc::exit(exit_code);
}

// original: write_summary_format (htslib/test/test-bcf-sr.c:71)
pub unsafe fn test_test_bcf_sr_c_71_write_summary_format(sr: *mut bcf_srs_t, out: *mut libc::FILE) {
    let mut n;
    while {
        n = crate::htslib_mini_rs::vcf::bcf_sr_next_line(sr);
        n > 0
    } {
        let mut i = 0;
        while i < (*sr).nreaders {
            if *(*sr).has_line.add(i as usize) == 0 {
                i += 1;
                continue;
            }
            let rec = *(*(*sr).readers.add(i as usize)).buffer;
            if rec.is_null() {
                error!(c"bcf_sr_get_line() unexpectedly returned NULL\n");
            }
            libc::fprintf(
                out,
                c"%s:%ld".as_ptr(),
                bcf_seqname_safe((*(*sr).readers.add(i as usize)).header, rec),
                (*rec).pos + 1,
            );
            break;
        }

        i = 0;
        while i < (*sr).nreaders {
            libc::fprintf(out, c"\t".as_ptr());

            if *(*sr).has_line.add(i as usize) == 0 {
                libc::fprintf(out, c"%s".as_ptr(), c"-".as_ptr());
                i += 1;
                continue;
            }

            let rec = *(*(*sr).readers.add(i as usize)).buffer;
            if rec.is_null() {
                error!(c"bcf_sr_get_line() unexpectedly returned NULL\n");
            }
            libc::fprintf(
                out,
                c"%s".as_ptr(),
                if (*rec).n_allele() > 1 {
                    *(*rec).d.allele.add(1)
                } else {
                    c".".as_ptr().cast_mut()
                },
            );
            let mut j = 2;
            while j < (*rec).n_allele() as c_int {
                libc::fprintf(out, c",%s".as_ptr(), *(*rec).d.allele.add(j as usize));
                j += 1;
            }
            i += 1;
        }
        libc::fprintf(out, c"\n".as_ptr());
    }
}

// original: write_vcf_bcf_format (htslib/test/test-bcf-sr.c:107)
pub unsafe fn test_test_bcf_sr_c_107_write_vcf_bcf_format(
    sr: *mut bcf_srs_t,
    hdr: *mut bcf_hdr_t,
    vcf_out: *mut crate::htslib_mini_rs::hts::htsFile,
    fmt_type: *const c_char,
) {
    let mut n;
    if crate::htslib_mini_rs::vcf::bcf_hdr_write(vcf_out, hdr) != 0 {
        error!(c"Couldn't write %s header\n", fmt_type);
    }

    while {
        n = crate::htslib_mini_rs::vcf::bcf_sr_next_line(sr);
        n > 0
    } {
        let mut i = 0;
        while i < (*sr).nreaders {
            if *(*sr).has_line.add(i as usize) == 0 {
                i += 1;
                continue;
            }
            let rec = *(*(*sr).readers.add(i as usize)).buffer;
            if rec.is_null() {
                error!(c"bcf_sr_get_line() unexpectedly returned NULL\n");
            }
            if crate::htslib_mini_rs::vcf::vcf_write(vcf_out, hdr, rec) < 0 {
                error!(c"vcf_write() failed\n");
            }
            i += 1;
        }
    }
}

const BCF_SR_PAIR_SNPS: c_int = hts_sys::BCF_SR_PAIR_SNPS as c_int;
const BCF_SR_PAIR_INDELS: c_int = hts_sys::BCF_SR_PAIR_INDELS as c_int;
const BCF_SR_PAIR_ANY: c_int = hts_sys::BCF_SR_PAIR_ANY as c_int;
const BCF_SR_PAIR_SOME: c_int = hts_sys::BCF_SR_PAIR_SOME as c_int;
const BCF_SR_PAIR_SNP_REF: c_int = hts_sys::BCF_SR_PAIR_SNP_REF as c_int;
const BCF_SR_PAIR_INDEL_REF: c_int = hts_sys::BCF_SR_PAIR_INDEL_REF as c_int;
const BCF_SR_PAIR_EXACT: c_int = hts_sys::BCF_SR_PAIR_EXACT as c_int;
const BCF_SR_PAIR_BOTH: c_int = BCF_SR_PAIR_SNPS | BCF_SR_PAIR_INDELS;
const BCF_SR_PAIR_BOTH_REF: c_int =
    BCF_SR_PAIR_SNPS | BCF_SR_PAIR_INDELS | BCF_SR_PAIR_SNP_REF | BCF_SR_PAIR_INDEL_REF;
const BCF_SR_ALLOW_NO_IDX: hts_sys::bcf_sr_opt_t = 2;
// original: main (htslib/test/test-bcf-sr.c:126)
pub unsafe fn test_test_bcf_sr_c_126_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    static mut LOPTIONS: [libc::option; 9] = [
        libc::option {
            name: c"help".as_ptr(),
            has_arg: libc::no_argument,
            flag: std::ptr::null_mut(),
            val: b'h' as c_int,
        },
        libc::option {
            name: c"output-fmt".as_ptr(),
            has_arg: libc::required_argument,
            flag: std::ptr::null_mut(),
            val: b'O' as c_int,
        },
        libc::option {
            name: c"pair".as_ptr(),
            has_arg: libc::required_argument,
            flag: std::ptr::null_mut(),
            val: b'p' as c_int,
        },
        libc::option {
            name: c"regions".as_ptr(),
            has_arg: libc::required_argument,
            flag: std::ptr::null_mut(),
            val: b'r' as c_int,
        },
        libc::option {
            name: c"targets".as_ptr(),
            has_arg: libc::required_argument,
            flag: std::ptr::null_mut(),
            val: b't' as c_int,
        },
        libc::option {
            name: c"no-index".as_ptr(),
            has_arg: libc::no_argument,
            flag: std::ptr::null_mut(),
            val: 1000,
        },
        libc::option {
            name: c"args".as_ptr(),
            has_arg: libc::no_argument,
            flag: std::ptr::null_mut(),
            val: 1001,
        },
        libc::option {
            name: c"usefptr".as_ptr(),
            has_arg: libc::no_argument,
            flag: std::ptr::null_mut(),
            val: b'u' as c_int,
        },
        libc::option {
            name: std::ptr::null(),
            has_arg: 0,
            flag: std::ptr::null_mut(),
            val: 0,
        },
    ];

    let mut c;
    let mut pair = 0;
    let mut use_index = 1;
    let mut use_fofn = 1;
    let mut usefptr = 0;
    let mut out_fmt: htsExactFormat = hts_sys::htsExactFormat_text_format;
    let mut out_fn: *const c_char = std::ptr::null();
    let mut regions: *const c_char = std::ptr::null();
    let mut targets: *const c_char = std::ptr::null();
    let mut htsfp: *mut *mut crate::htslib_mini_rs::hts::htsFile = std::ptr::null_mut();

    while {
        c = libc::getopt_long(
            argc,
            argv,
            c"o:O:p:r:t:hu".as_ptr(),
            std::ptr::addr_of_mut!(LOPTIONS).cast(),
            std::ptr::null_mut(),
        );
        c >= 0
    } {
        match c {
            x if x == b'o' as c_int => {
                out_fn = libc::optarg;
            }
            x if x == b'O' as c_int => {
                if libc::strcasecmp(libc::optarg, c"vcf".as_ptr()) == 0 {
                    out_fmt = hts_sys::htsExactFormat_vcf;
                } else if libc::strcasecmp(libc::optarg, c"bcf".as_ptr()) == 0 {
                    out_fmt = hts_sys::htsExactFormat_bcf;
                } else if libc::strcasecmp(libc::optarg, c"summary".as_ptr()) == 0 {
                    out_fmt = hts_sys::htsExactFormat_text_format;
                } else {
                    error!(c"Unknown output format \"%s\"\n", libc::optarg);
                }
            }
            x if x == b'p' as c_int => {
                if libc::strcmp(libc::optarg, c"snps".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_SNPS;
                } else if libc::strcmp(libc::optarg, c"snp+ref".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_SNPS | BCF_SR_PAIR_SNP_REF;
                } else if libc::strcmp(libc::optarg, c"snps+ref".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_SNPS | BCF_SR_PAIR_SNP_REF;
                } else if libc::strcmp(libc::optarg, c"indels".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_INDELS;
                } else if libc::strcmp(libc::optarg, c"indel+ref".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_INDELS | BCF_SR_PAIR_INDEL_REF;
                } else if libc::strcmp(libc::optarg, c"indels+ref".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_INDELS | BCF_SR_PAIR_INDEL_REF;
                } else if libc::strcmp(libc::optarg, c"both".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_BOTH;
                } else if libc::strcmp(libc::optarg, c"both+ref".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_BOTH_REF;
                } else if libc::strcmp(libc::optarg, c"any".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_ANY;
                } else if libc::strcmp(libc::optarg, c"all".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_ANY;
                } else if libc::strcmp(libc::optarg, c"some".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_SOME;
                } else if libc::strcmp(libc::optarg, c"exact".as_ptr()) == 0 {
                    pair = BCF_SR_PAIR_EXACT;
                } else {
                    error!(c"The --pair logic \"%s\" not recognised.\n", libc::optarg);
                }
            }
            x if x == b'r' as c_int => {
                regions = libc::optarg;
            }
            x if x == b't' as c_int => {
                targets = libc::optarg;
            }
            1000 => {
                use_index = 0;
            }
            1001 => {
                use_fofn = 0;
            }
            x if x == b'u' as c_int => {
                usefptr = 1;
            }
            x if x == b'h' as c_int => {
                test_test_bcf_sr_c_54_usage(libc::EXIT_SUCCESS);
            }
            _ => test_test_bcf_sr_c_54_usage(libc::EXIT_FAILURE),
        }
    }
    if pair == 0 {
        pair = BCF_SR_PAIR_EXACT;
    }
    if libc::optind == argc {
        test_test_bcf_sr_c_54_usage(libc::EXIT_FAILURE);
    }

    let mut nvcf = 0;
    let vcfs: *mut *mut c_char;
    if use_fofn != 0 {
        vcfs = hts_readlist(*argv.add(libc::optind as usize), 1, &mut nvcf);
        if vcfs.is_null() {
            error!(c"Could not parse %s\n", *argv.add(libc::optind as usize));
        }
    } else {
        vcfs = argv.add(libc::optind as usize);
        nvcf = argc - libc::optind;
    }

    let sr = crate::htslib_mini_rs::vcf::bcf_sr_init();
    if sr.is_null() {
        error!(c"bcf_sr_init() failed\n");
    }
    hts_sys::bcf_sr_set_opt(sr, hts_sys::bcf_sr_opt_t_BCF_SR_PAIR_LOGIC, pair);
    if use_index != 0 {
        hts_sys::bcf_sr_set_opt(sr, hts_sys::bcf_sr_opt_t_BCF_SR_REQUIRE_IDX);
    } else {
        hts_sys::bcf_sr_set_opt(sr, BCF_SR_ALLOW_NO_IDX);
    }

    if !regions.is_null() && crate::htslib_mini_rs::vcf::bcf_sr_set_regions(sr, regions, 0) != 0 {
        error!(c"Failed to set regions\n");
    }

    if !targets.is_null() && crate::htslib_mini_rs::vcf::bcf_sr_set_targets(sr, targets, 0, 0) != 0
    {
        error!(c"Failed to set targets\n");
    }

    if usefptr != 0 {
        htsfp = libc::malloc(
            std::mem::size_of::<*mut crate::htslib_mini_rs::hts::htsFile>() * nvcf as usize,
        )
        .cast();
        if htsfp.is_null() {
            error!(c"Failed to allocate memory\n");
        }
    }

    let mut i = 0;
    while i < nvcf {
        if usefptr == 0 {
            if crate::htslib_mini_rs::vcf::bcf_sr_add_reader(sr, *vcfs.add(i as usize)) == 0 {
                error!(
                    c"Failed to open %s: %s\n",
                    *vcfs.add(i as usize),
                    crate::htslib_mini_rs::vcf::bcf_sr_strerror((*sr).errnum as c_int)
                );
            }
        } else {
            *htsfp.add(i as usize) =
                crate::htslib_mini_rs::hts::hts_open(*vcfs.add(i as usize), c"r".as_ptr());
            if (*htsfp.add(i as usize)).is_null() {
                error!(
                    c"Failed to open %s: %s\n",
                    *vcfs.add(i as usize),
                    crate::htslib_mini_rs::vcf::bcf_sr_strerror((*sr).errnum as c_int)
                );
            }

            /*
              with name, index can be anywhere, named as anything
              w/o name it has to be along with file with default naming
            */
            let mut idxname = libc::strstr(
                *vcfs.add(i as usize),
                hts_sys::HTS_IDX_DELIM.as_ptr().cast(),
            );
            if !idxname.is_null() {
                idxname = idxname.add(libc::strlen(hts_sys::HTS_IDX_DELIM.as_ptr().cast()));
            }
            if crate::htslib_mini_rs::vcf::bcf_sr_add_hreader(
                sr,
                *htsfp.add(i as usize),
                1,
                idxname,
            ) == 0
            {
                error!(
                    c"Failed to add reader %s: %s\n",
                    *vcfs.add(i as usize),
                    crate::htslib_mini_rs::vcf::bcf_sr_strerror((*sr).errnum as c_int)
                );
            }
        }
        i += 1;
    }

    if (*sr).readers.is_null() || (*sr).nreaders < 1 {
        error!(c"No readers set, even though one was added\n");
    }

    if out_fmt == hts_sys::htsExactFormat_text_format {
        let mut out = hts_sys::stdout.cast::<libc::FILE>();
        if !out_fn.is_null() {
            out = libc::fopen(out_fn, c"w".as_ptr());
            if out.is_null() {
                error!(
                    c"Couldn't open \"%s\" for writing: %s\n",
                    out_fn,
                    libc::strerror(*libc::__errno_location())
                );
            }
        }
        test_test_bcf_sr_c_71_write_summary_format(sr, out);
        if !out_fn.is_null() && libc::fclose(out) != 0 {
            error!(
                c"Error on closing %s : %s\n",
                out_fn,
                libc::strerror(*libc::__errno_location())
            );
        }
    } else {
        let fmt_type = if out_fmt == hts_sys::htsExactFormat_vcf {
            c"VCF".as_ptr()
        } else {
            c"BCF".as_ptr()
        };

        let hdr = (*(*sr).readers).header;
        if hdr.is_null() {
            error!(c"%s output, but don't have a header\n", fmt_type);
        }

        if out_fn.is_null() {
            out_fn = c"-".as_ptr();
        }
        let vcf_out = crate::htslib_mini_rs::hts::hts_open(
            out_fn,
            if out_fmt == hts_sys::htsExactFormat_vcf {
                c"w".as_ptr()
            } else {
                c"wb".as_ptr()
            },
        );
        if vcf_out.is_null() {
            error!(
                c"Couldn't open \"%s\" for writing: %s\n",
                out_fn,
                libc::strerror(*libc::__errno_location())
            );
        }
        test_test_bcf_sr_c_107_write_vcf_bcf_format(sr, hdr, vcf_out, fmt_type);
        if crate::htslib_mini_rs::hts::hts_close(vcf_out) != 0 {
            error!(c"Error on closing \"%s\"\n", out_fn);
        }
    }

    if (*sr).errnum != 0 {
        error!(
            c"Synced reader error: %s\n",
            crate::htslib_mini_rs::vcf::bcf_sr_strerror((*sr).errnum as c_int)
        );
    }

    crate::htslib_mini_rs::vcf::bcf_sr_destroy(sr);
    if use_fofn != 0 {
        i = 0;
        while i < nvcf {
            libc::free((*vcfs.add(i as usize)).cast());
            i += 1;
        }
        libc::free(vcfs.cast());
    }
    if usefptr != 0 {
        libc::free(htsfp.cast());
    }

    0
}
