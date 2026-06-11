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

use crate::htslib_rs::{
    hts::{htsExactFormat, hts_readlist},
    vcf::{bcf_hdr_t, bcf_seqname_safe, bcf_srs_t},
};

unsafe extern "C" {
    static mut optarg: *mut u8;
    static mut optind: i32;
}

macro_rules! error {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        libc::fprintf(libc::fdopen(2, c"w".as_ptr()), $fmt.as_ptr() $(, $arg)*);
        libc::exit(libc::EXIT_FAILURE);
    }};
}

// original: usage (htslib/test/test-bcf-sr.c:54)
pub unsafe fn test_test_bcf_sr_c_54_usage(exit_code: i32) -> ! {
    eprint!("Usage: test-bcf-sr [OPTIONS] vcf-list.txt\n");
    eprint!("       test-bcf-sr [OPTIONS] --args file1.bcf [...]\n");
    eprint!("Options:\n");
    eprint!("       --args                   pass filenames directly in argument list\n");
    eprint!("       --no-index               allow streaming\n");
    eprint!("   -o, --output <file>          output file (stdout if not set)\n");
    eprint!("   -O, --output-fmt <fmt>       fmt: vcf,bcf,summary\n");
    eprint!("   -p, --pair <logic[+ref]>     logic: snps,indels,both,snps+ref,indels+ref,both+ref,exact,some,all\n");
    eprint!("   -r, --regions <reg_list>     comma-separated list of regions\n");
    eprint!("   -t, --targets <reg_list>     comma-separated list of targets\n");
    eprint!("   -u, --usefptr                use hfile pointer interface on reader addition\n");
    eprint!("\n");
    libc::exit(exit_code);
}

// original: write_summary_format (htslib/test/test-bcf-sr.c:71)
pub unsafe fn test_test_bcf_sr_c_71_write_summary_format(sr: *mut bcf_srs_t, out: *mut libc::FILE) {
    let mut n;
    while {
        n = crate::htslib_rs::vcf::bcf_sr_next_line(&mut *sr);
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
            let allele1: Vec<u8> = if (*rec).n_allele() > 1 {
                let d = &(*rec).d;
                let mut a = d.allele[1].clone();
                a.push(0);
                a
            } else {
                b".\0".to_vec()
            };
            libc::fprintf(out, c"%s".as_ptr(), allele1.as_ptr().cast::<i8>());
            let mut j = 2;
            while j < (*rec).n_allele() as i32 {
                let d = &(*rec).d;
                let mut a = d.allele[j as usize].clone();
                a.push(0);
                libc::fprintf(out, c",%s".as_ptr(), a.as_ptr().cast::<i8>());
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
    vcf_out: *mut crate::htslib_rs::hts::htsFile,
    fmt_type: *const u8,
) {
    let mut n;
    if crate::htslib_rs::vcf::bcf_hdr_write(vcf_out, hdr) != 0 {
        error!(c"Couldn't write %s header\n", fmt_type);
    }

    while {
        n = crate::htslib_rs::vcf::bcf_sr_next_line(&mut *sr);
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
            if crate::htslib_rs::vcf::bcf_write(vcf_out, hdr, rec) < 0 {
                error!(c"bcf_write() failed\n");
            }
            i += 1;
        }
    }
}

const BCF_SR_PAIR_SNPS: i32 = crate::htslib_rs::vcf::BCF_SR_PAIR_SNPS as i32;
const BCF_SR_PAIR_INDELS: i32 = crate::htslib_rs::vcf::BCF_SR_PAIR_INDELS as i32;
const BCF_SR_PAIR_ANY: i32 = crate::htslib_rs::vcf::BCF_SR_PAIR_ANY as i32;
const BCF_SR_PAIR_SOME: i32 = crate::htslib_rs::vcf::BCF_SR_PAIR_SOME as i32;
const BCF_SR_PAIR_SNP_REF: i32 = crate::htslib_rs::vcf::BCF_SR_PAIR_SNP_REF as i32;
const BCF_SR_PAIR_INDEL_REF: i32 = crate::htslib_rs::vcf::BCF_SR_PAIR_INDEL_REF as i32;
const BCF_SR_PAIR_EXACT: i32 = crate::htslib_rs::vcf::BCF_SR_PAIR_EXACT as i32;
const BCF_SR_PAIR_BOTH: i32 = BCF_SR_PAIR_SNPS | BCF_SR_PAIR_INDELS;
const BCF_SR_PAIR_BOTH_REF: i32 =
    BCF_SR_PAIR_SNPS | BCF_SR_PAIR_INDELS | BCF_SR_PAIR_SNP_REF | BCF_SR_PAIR_INDEL_REF;
const BCF_SR_ALLOW_NO_IDX: crate::htslib_rs::vcf::bcf_sr_opt_t = 2;
// original: main (htslib/test/test-bcf-sr.c:126)
pub unsafe fn test_test_bcf_sr_c_126_main(argc: i32, argv: *mut *mut u8) -> i32 {
    static mut LOPTIONS: [libc::option; 9] = [
        libc::option {
            name: c"help".as_ptr(),
            has_arg: 0,
            flag: std::ptr::null_mut(),
            val: b'h' as i32,
        },
        libc::option {
            name: c"output-fmt".as_ptr(),
            has_arg: 1,
            flag: std::ptr::null_mut(),
            val: b'O' as i32,
        },
        libc::option {
            name: c"pair".as_ptr(),
            has_arg: 1,
            flag: std::ptr::null_mut(),
            val: b'p' as i32,
        },
        libc::option {
            name: c"regions".as_ptr(),
            has_arg: 1,
            flag: std::ptr::null_mut(),
            val: b'r' as i32,
        },
        libc::option {
            name: c"targets".as_ptr(),
            has_arg: 1,
            flag: std::ptr::null_mut(),
            val: b't' as i32,
        },
        libc::option {
            name: c"no-index".as_ptr(),
            has_arg: 0,
            flag: std::ptr::null_mut(),
            val: 1000,
        },
        libc::option {
            name: c"args".as_ptr(),
            has_arg: 0,
            flag: std::ptr::null_mut(),
            val: 1001,
        },
        libc::option {
            name: c"usefptr".as_ptr(),
            has_arg: 0,
            flag: std::ptr::null_mut(),
            val: b'u' as i32,
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
    let mut out_fmt: htsExactFormat = crate::htslib_rs::hts::HTS_FORMAT_TEXT_FORMAT;
    let mut out_fn: *const u8 = std::ptr::null();
    let mut regions: *const u8 = std::ptr::null();
    let mut targets: *const u8 = std::ptr::null();
    let mut htsfp: Vec<*mut crate::htslib_rs::hts::htsFile> = Vec::new();

    while {
        c = libc::getopt_long(
            argc,
            argv.cast(),
            c"o:O:p:r:t:hu".as_ptr(),
            std::ptr::addr_of_mut!(LOPTIONS).cast(),
            std::ptr::null_mut(),
        );
        c >= 0
    } {
        match c {
            x if x == b'o' as i32 => {
                out_fn = optarg;
            }
            x if x == b'O' as i32 => {
                if libc::strcasecmp(optarg.cast(), c"vcf".as_ptr()) == 0 {
                    out_fmt = crate::htslib_rs::hts::HTS_FORMAT_VCF;
                } else if libc::strcasecmp(optarg.cast(), c"bcf".as_ptr()) == 0 {
                    out_fmt = crate::htslib_rs::hts::HTS_FORMAT_BCF;
                } else if libc::strcasecmp(optarg.cast(), c"summary".as_ptr()) == 0 {
                    out_fmt = crate::htslib_rs::hts::HTS_FORMAT_TEXT_FORMAT;
                } else {
                    error!(c"Unknown output format \"%s\"\n", optarg);
                }
            }
            x if x == b'p' as i32 => {
                if libc::strcmp(optarg.cast(), c"snps".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_SNPS;
                } else if libc::strcmp(optarg.cast(), c"snp+ref".as_ptr()) == 0
                    || libc::strcmp(optarg.cast(), c"snps+ref".as_ptr()) == 0
                {
                    pair |= BCF_SR_PAIR_SNPS | BCF_SR_PAIR_SNP_REF;
                } else if libc::strcmp(optarg.cast(), c"indels".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_INDELS;
                } else if libc::strcmp(optarg.cast(), c"indel+ref".as_ptr()) == 0
                    || libc::strcmp(optarg.cast(), c"indels+ref".as_ptr()) == 0
                {
                    pair |= BCF_SR_PAIR_INDELS | BCF_SR_PAIR_INDEL_REF;
                } else if libc::strcmp(optarg.cast(), c"both".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_BOTH;
                } else if libc::strcmp(optarg.cast(), c"both+ref".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_BOTH_REF;
                } else if libc::strcmp(optarg.cast(), c"any".as_ptr()) == 0
                    || libc::strcmp(optarg.cast(), c"all".as_ptr()) == 0
                {
                    pair |= BCF_SR_PAIR_ANY;
                } else if libc::strcmp(optarg.cast(), c"some".as_ptr()) == 0 {
                    pair |= BCF_SR_PAIR_SOME;
                } else if libc::strcmp(optarg.cast(), c"exact".as_ptr()) == 0 {
                    pair = BCF_SR_PAIR_EXACT;
                } else {
                    error!(c"The --pair logic \"%s\" not recognised.\n", optarg);
                }
            }
            x if x == b'r' as i32 => {
                regions = optarg;
            }
            x if x == b't' as i32 => {
                targets = optarg;
            }
            1000 => {
                use_index = 0;
            }
            1001 => {
                use_fofn = 0;
            }
            x if x == b'u' as i32 => {
                usefptr = 1;
            }
            x if x == b'h' as i32 => {
                test_test_bcf_sr_c_54_usage(libc::EXIT_SUCCESS);
            }
            _ => test_test_bcf_sr_c_54_usage(libc::EXIT_FAILURE),
        }
    }
    if pair == 0 {
        pair = BCF_SR_PAIR_EXACT;
    }
    if optind == argc {
        test_test_bcf_sr_c_54_usage(libc::EXIT_FAILURE);
    }

    let mut nvcf = 0;
    let vcfs: *mut *mut u8;
    if use_fofn != 0 {
        vcfs = hts_readlist((*argv.add(optind as usize)).cast(), 1, &mut nvcf).cast();
        if vcfs.is_null() {
            error!(c"Could not parse %s\n", *argv.add(optind as usize));
        }
    } else {
        vcfs = argv.add(optind as usize);
        nvcf = argc - optind;
    }

    let sr = crate::htslib_rs::vcf::bcf_sr_init();
    if sr.is_null() {
        error!(c"bcf_sr_init() failed\n");
    }
    crate::htslib_rs::vcf::bcf_sr_set_opt(&mut *sr, crate::htslib_rs::vcf::BCF_SR_PAIR_LOGIC, pair);
    if use_index != 0 {
        crate::htslib_rs::vcf::bcf_sr_set_opt(&mut *sr, crate::htslib_rs::vcf::BCF_SR_REQUIRE_IDX, 0);
    } else {
        crate::htslib_rs::vcf::bcf_sr_set_opt(&mut *sr, BCF_SR_ALLOW_NO_IDX, 0);
    }

    if !regions.is_null()
        && crate::htslib_rs::vcf::bcf_sr_set_regions(
            &mut *sr,
            std::ffi::CStr::from_ptr(regions.cast()).to_bytes(),
            0,
        ) != 0
    {
        error!(c"Failed to set regions\n");
    }

    if !targets.is_null()
        && crate::htslib_rs::vcf::bcf_sr_set_targets(
            &mut *sr,
            std::ffi::CStr::from_ptr(targets.cast()).to_bytes(),
            0,
            0,
        ) != 0
    {
        error!(c"Failed to set targets\n");
    }

    if usefptr != 0 {
        htsfp = vec![std::ptr::null_mut(); nvcf as usize];
    }

    let mut i = 0;
    while i < nvcf {
        if usefptr == 0 {
            if crate::htslib_rs::vcf::bcf_sr_add_reader(
                &mut *sr,
                std::ffi::CStr::from_ptr((*vcfs.add(i as usize)).cast()).to_bytes(),
            ) == 0
            {
                error!(
                    c"Failed to open %s: %s\n",
                    *vcfs.add(i as usize),
                    crate::htslib_rs::vcf::bcf_sr_strerror((*sr).errnum as i32)
                );
            }
        } else {
            htsfp[i as usize] =
                crate::htslib_rs::hts::hts_open((*vcfs.add(i as usize)).cast(), c"r".as_ptr());
            if htsfp[i as usize].is_null() {
                error!(
                    c"Failed to open %s: %s\n",
                    *vcfs.add(i as usize),
                    crate::htslib_rs::vcf::bcf_sr_strerror((*sr).errnum as i32)
                );
            }

            /*
              with name, index can be anywhere, named as anything
              w/o name it has to be along with file with default naming
            */
            let mut idxname = libc::strstr(
                (*vcfs.add(i as usize)).cast(),
                crate::htslib_rs::hts::HTS_IDX_DELIM.as_ptr().cast(),
            );
            if !idxname.is_null() {
                idxname = idxname.add(libc::strlen(
                    crate::htslib_rs::hts::HTS_IDX_DELIM.as_ptr().cast(),
                ));
            }
            let idxname_arg = if idxname.is_null() {
                None
            } else {
                Some(std::ffi::CStr::from_ptr(idxname).to_bytes())
            };
            if crate::htslib_rs::vcf::bcf_sr_add_hreader(
                &mut *sr,
                &mut *htsfp[i as usize],
                1,
                idxname_arg,
            ) == 0
            {
                error!(
                    c"Failed to add reader %s: %s\n",
                    *vcfs.add(i as usize),
                    crate::htslib_rs::vcf::bcf_sr_strerror((*sr).errnum as i32)
                );
            }
        }
        i += 1;
    }

    if (*sr).readers.is_null() || (*sr).nreaders < 1 {
        error!(c"No readers set, even though one was added\n");
    }

    if out_fmt == crate::htslib_rs::hts::HTS_FORMAT_TEXT_FORMAT {
        let mut out = libc::fdopen(1, c"w".as_ptr());
        if !out_fn.is_null() {
            out = libc::fopen(out_fn.cast(), c"w".as_ptr());
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
        let fmt_type: *const u8 = if out_fmt == crate::htslib_rs::hts::HTS_FORMAT_VCF {
            c"VCF".as_ptr().cast()
        } else {
            c"BCF".as_ptr().cast()
        };

        let hdr = (*(*sr).readers).header;
        if hdr.is_null() {
            error!(c"%s output, but don't have a header\n", fmt_type);
        }

        if out_fn.is_null() {
            out_fn = c"-".as_ptr().cast();
        }
        let vcf_out = crate::htslib_rs::hts::hts_open(
            out_fn.cast(),
            if out_fmt == crate::htslib_rs::hts::HTS_FORMAT_VCF {
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
        if crate::htslib_rs::hts::hts_close(vcf_out) != 0 {
            error!(c"Error on closing \"%s\"\n", out_fn);
        }
    }

    if (*sr).errnum != 0 {
        error!(
            c"Synced reader error: %s\n",
            crate::htslib_rs::vcf::bcf_sr_strerror((*sr).errnum as i32)
        );
    }

    crate::htslib_rs::vcf::bcf_sr_destroy(sr);
    if use_fofn != 0 {
        i = 0;
        while i < nvcf {
            libc::free((*vcfs.add(i as usize)).cast());
            i += 1;
        }
        libc::free(vcfs.cast());
    }
    drop(htsfp);

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    static GETOPT_LOCK: Mutex<()> = Mutex::new(());

    fn fixture(path: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
    }

    fn c_path(path: &Path) -> Vec<u8> {
        let mut v = path.to_string_lossy().as_bytes().to_vec();
        v.push(0);
        v
    }

    fn temp_path(label: &str, ext: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "htslib-rs-test-bcf-sr-main-{label}-{}-{}.{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed"),
            ext
        ))
    }

    unsafe fn run_main(args: &mut [Vec<u8>]) -> i32 {
        // NOTE: callers must already hold `ORIGINAL_MAIN_LOCK` (see
        // src/test/mod.rs). The per-file `GETOPT_LOCK` is retained so direct
        // helper-level callers (none today) remain safe; std `Mutex` is not
        // reentrant, so we never re-acquire `ORIGINAL_MAIN_LOCK` here.
        let _guard = GETOPT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");

        if pid == 0 {
            optarg = std::ptr::null_mut();
            optind = 1;
            let mut argv = args
                .iter_mut()
                .map(|arg| arg.as_ptr().cast_mut())
                .collect::<Vec<*mut u8>>();
            let ret = test_test_bcf_sr_c_126_main(argv.len() as i32, argv.as_mut_ptr());
            libc::fflush(std::ptr::null_mut());
            libc::_exit(ret);
        }

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status), "child did not exit normally");
        libc::WEXITSTATUS(status)
    }

    unsafe fn read_bcf_sites(path: &Path) -> Vec<String> {
        let path = c_path(path);
        let fp = crate::htslib_rs::hts::hts_open(path.as_ptr().cast(), c"r".as_ptr());
        assert!(!fp.is_null());
        let hdr = crate::htslib_rs::vcf::bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = crate::htslib_rs::vcf::bcf_init();
        assert!(!rec.is_null());

        let mut sites = Vec::new();
        while crate::htslib_rs::vcf::bcf_read(fp, hdr, rec) >= 0 {
            assert_eq!(
                crate::htslib_rs::vcf::bcf_unpack(rec, crate::htslib_rs::vcf::BCF_UN_STR as i32),
                0
            );
            let chrom = CStr::from_ptr(crate::htslib_rs::vcf::bcf_seqname(hdr, rec).cast())
                .to_str()
                .unwrap();
            let d = &(*rec).d;
            let ref_allele = std::str::from_utf8(&d.allele[0]).unwrap();
            let alt_allele = if (*rec).n_allele() > 1 {
                let d = &(*rec).d;
                std::str::from_utf8(&d.allele[1]).unwrap()
            } else {
                "."
            };
            sites.push(format!(
                "{chrom}\t{}\t{ref_allele}\t{alt_allele}",
                (*rec).pos + 1
            ));
        }

        crate::htslib_rs::vcf::bcf_destroy(rec);
        crate::htslib_rs::vcf::bcf_hdr_destroy(hdr);
        assert_eq!(crate::htslib_rs::hts::hts_close(fp), 0);
        sites
    }

    unsafe fn make_indexed_bcf_from_vcf(vcf_path: &Path, label: &str) -> PathBuf {
        let bcf_path = temp_path(label, "bcf");
        let csi_path = bcf_path.with_extension("bcf.csi");
        let _ = std::fs::remove_file(&bcf_path);
        let _ = std::fs::remove_file(&csi_path);

        let c_vcf_path = c_path(vcf_path);
        let c_bcf_path = c_path(&bcf_path);
        let c_csi_path = c_path(&csi_path);

        let in_fp = crate::htslib_rs::hts::hts_open(c_vcf_path.as_ptr().cast(), c"r".as_ptr());
        assert!(!in_fp.is_null());
        let hdr = crate::htslib_rs::vcf::bcf_hdr_read(in_fp);
        assert!(!hdr.is_null());

        let out_fp = crate::htslib_rs::hts::hts_open(c_bcf_path.as_ptr().cast(), c"wb".as_ptr());
        assert!(!out_fp.is_null());
        assert_eq!(crate::htslib_rs::vcf::bcf_hdr_write(out_fp, hdr), 0);

        let rec = crate::htslib_rs::vcf::bcf_init();
        assert!(!rec.is_null());
        while crate::htslib_rs::vcf::bcf_read(in_fp, hdr, rec) >= 0 {
            assert_eq!(crate::htslib_rs::vcf::bcf_write(out_fp, hdr, rec), 0);
        }

        crate::htslib_rs::vcf::bcf_destroy(rec);
        assert_eq!(crate::htslib_rs::hts::hts_close(out_fp), 0);
        crate::htslib_rs::vcf::bcf_hdr_destroy(hdr);
        assert_eq!(crate::htslib_rs::hts::hts_close(in_fp), 0);
        assert_eq!(
            crate::htslib_rs::vcf::bcf_index_build2(c_bcf_path.as_ptr().cast(), c_csi_path.as_ptr().cast(), 14),
            0
        );

        bcf_path
    }

    #[test]
    fn original_test_bcf_sr_main_args_summary_matches_no_index_merge_fixture() {
        // Process-wide lock: see src/test/mod.rs. Held for the entire test
        // because the parent does htslib library work outside run_main.
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let out = temp_path("args-summary", "out");
        let _ = std::fs::remove_file(&out);

        let mut args = vec![
            b"test-bcf-sr\0".to_vec(),
            b"--args\0".to_vec(),
            b"--no-index\0".to_vec(),
            b"-O\0".to_vec(),
            b"summary\0".to_vec(),
            b"-p\0".to_vec(),
            b"any\0".to_vec(),
            b"-o\0".to_vec(),
            c_path(&out),
            c_path(&fixture("htslib/test/bcf-sr/merge.noidx.a.vcf")),
            c_path(&fixture("htslib/test/bcf-sr/merge.noidx.b.vcf")),
            c_path(&fixture("htslib/test/bcf-sr/merge.noidx.c.vcf")),
        ];

        unsafe {
            assert_eq!(run_main(&mut args), 0);
        }
        let actual = std::fs::read_to_string(&out).unwrap();
        let expected =
            std::fs::read_to_string(fixture("htslib/test/bcf-sr/merge.noidx.abc.expected.out"))
                .unwrap();
        assert_eq!(actual, expected);
        let _ = std::fs::remove_file(out);
    }

    #[test]
    fn original_test_bcf_sr_main_fofn_vcf_output_matches_weird_chromosome_fixture() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let out = temp_path("fofn-vcf", "vcf");
        let list = temp_path("fofn-vcf", "list");
        let _ = std::fs::remove_file(&out);
        let _ = std::fs::remove_file(&list);
        std::fs::write(
            &list,
            format!(
                "{}\n",
                fixture("htslib/test/bcf-sr/weird-chr-names.vcf").display()
            ),
        )
        .unwrap();

        let mut args = vec![
            b"test-bcf-sr\0".to_vec(),
            b"--no-index\0".to_vec(),
            b"-O\0".to_vec(),
            b"vcf\0".to_vec(),
            b"-o\0".to_vec(),
            c_path(&out),
            c_path(&list),
        ];

        unsafe {
            assert_eq!(run_main(&mut args), 0);
        }
        let actual = std::fs::read_to_string(&out).unwrap();
        assert_eq!(
            actual,
            concat!(
                "##fileformat=VCFv4.3\n",
                "##FILTER=<ID=PASS,Description=\"All filters passed\">\n",
                "##reference=ref.fa\n",
                "##contig=<ID=1>\n",
                "##contig=<ID=1:1>\n",
                "##contig=<ID=1:1-1>\n",
                "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n",
                "1\t1\t.\tC\tT\t.\t.\t.\n",
                "1\t2\t.\tC\tT\t.\t.\t.\n",
                "1:1\t1\t.\tC\tT\t.\t.\t.\n",
                "1:1\t2\t.\tC\tT\t.\t.\t.\n",
                "1:1-1\t1\t.\tC\tT\t.\t.\t.\n",
                "1:1-1\t2\t.\tC\tT\t.\t.\t.\n",
            )
        );
        let _ = std::fs::remove_file(out);
        let _ = std::fs::remove_file(list);
    }

    #[test]
    fn original_test_bcf_sr_main_indexed_regions_and_targets_match_weird_chromosome_outputs() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let bcf = unsafe {
            make_indexed_bcf_from_vcf(
                &fixture("htslib/test/bcf-sr/weird-chr-names.vcf"),
                "indexed-weird",
            )
        };

        let cases = [
            (
                "-r",
                "1:1-2",
                "htslib/test/bcf-sr/weird-chr-names.1.out",
                "region-plain-range",
            ),
            (
                "-r",
                "{1:1}:1-1",
                "htslib/test/bcf-sr/weird-chr-names.4.out",
                "region-braced-range",
            ),
            (
                "-t",
                "{1:1}:1,{1:1}:2",
                "htslib/test/bcf-sr/weird-chr-names.3.out",
                "target-braced-list",
            ),
            (
                "-t",
                "{1:1-1}:1-1",
                "htslib/test/bcf-sr/weird-chr-names.6.out",
                "target-braced-range",
            ),
        ];

        for (flag, range, expected_path, label) in cases {
            let out = temp_path(label, "vcf");
            let _ = std::fs::remove_file(&out);
            let mut args = vec![
                b"test-bcf-sr\0".to_vec(),
                b"--args\0".to_vec(),
                b"-O\0".to_vec(),
                b"vcf\0".to_vec(),
                b"-o\0".to_vec(),
                c_path(&out),
                {
                    let mut v = flag.as_bytes().to_vec();
                    v.push(0);
                    v
                },
                {
                    let mut v = range.as_bytes().to_vec();
                    v.push(0);
                    v
                },
                c_path(&bcf),
            ];

            unsafe {
                assert_eq!(run_main(&mut args), 0, "{label}");
            }
            let actual = std::fs::read_to_string(&out).unwrap();
            let expected = std::fs::read_to_string(fixture(expected_path)).unwrap();
            assert_eq!(actual, expected, "{label}");
            let _ = std::fs::remove_file(out);
        }

        let _ = std::fs::remove_file(&bcf);
        let _ = std::fs::remove_file(bcf.with_extension("bcf.csi"));
    }

    #[test]
    fn original_test_bcf_sr_main_args_bcf_output_round_trips_weird_chromosome_fixture() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let out = temp_path("args-bcf", "bcf");
        let _ = std::fs::remove_file(&out);

        let mut args = vec![
            b"test-bcf-sr\0".to_vec(),
            b"--args\0".to_vec(),
            b"--no-index\0".to_vec(),
            b"-O\0".to_vec(),
            b"bcf\0".to_vec(),
            b"-o\0".to_vec(),
            c_path(&out),
            c_path(&fixture("htslib/test/bcf-sr/weird-chr-names.vcf")),
        ];

        unsafe {
            assert_eq!(run_main(&mut args), 0);
        }

        let bytes = std::fs::read(&out).unwrap();
        assert!(bytes.starts_with(&[0x1f, 0x8b]));
        unsafe {
            assert_eq!(
                read_bcf_sites(&out),
                vec![
                    "1\t1\tC\tT",
                    "1\t2\tC\tT",
                    "1:1\t1\tC\tT",
                    "1:1\t2\tC\tT",
                    "1:1-1\t1\tC\tT",
                    "1:1-1\t2\tC\tT",
                ]
            );
        }
        let _ = std::fs::remove_file(out);
    }
}
