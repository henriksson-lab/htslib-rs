include!("../bin_alloc_for_bins.rs");

use std::ffi::CString;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: bam_to_cram <in.bam> <out.cram>");
        std::process::exit(2);
    }
    unsafe {
        let in_path = CString::new(args[1].as_str()).unwrap();
        let mode_r = CString::new("rb").unwrap();
        let out_path = CString::new(args[2].as_str()).unwrap();
        let mode_w = CString::new("wc").unwrap();
        let in_fp = htslib_rs::hts::hts_open(in_path.as_ptr(), mode_r.as_ptr());
        if in_fp.is_null() {
            panic!("hts_open in");
        }
        let hdr = htslib_rs::sam::sam_hdr_read(in_fp);
        if hdr.is_null() {
            panic!("sam_hdr_read");
        }
        // Open with explicit format options: no_ref (so we don't need a reference)
        let mut fmt: htslib_rs::hts::htsFormat = std::mem::zeroed();
        let fmt_str = CString::new("cram,no_ref").unwrap();
        if htslib_rs::hts::hts_parse_format(&mut fmt, fmt_str.as_ptr()) < 0 {
            panic!("parse_format");
        }
        let out_fp = htslib_rs::hts::hts_open_format(out_path.as_ptr(), mode_w.as_ptr(), &fmt);
        if out_fp.is_null() {
            panic!("open out");
        }
        if htslib_rs::sam::sam_hdr_write(out_fp, hdr) < 0 {
            panic!("sam_hdr_write");
        }
        let rec = htslib_rs::sam::bam_init1();
        let mut n = 0u64;
        while htslib_rs::sam::sam_read1(in_fp, hdr, rec) >= 0 {
            if htslib_rs::sam::sam_c_4553_sam_write1(out_fp, hdr, rec) < 0 {
                panic!("sam_write1 at record {}", n);
            }
            n += 1;
            if n.is_multiple_of(1_000_000) {
                eprintln!("  wrote {n}");
            }
        }
        htslib_rs::sam::bam_destroy1(rec);
        htslib_rs::sam::sam_hdr_destroy(hdr);
        htslib_rs::hts::hts_close(in_fp);
        htslib_rs::hts::hts_close(out_fp);
        eprintln!("done, {n} records");
    }
}
