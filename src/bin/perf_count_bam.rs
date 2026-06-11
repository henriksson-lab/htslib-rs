use std::ffi::CString;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: perf_count_bam <bam> [region]");
        std::process::exit(2);
    };
    let region = args.next();
    let path = CString::new(path).unwrap();
    let region = region.map(|r| CString::new(r).unwrap());

    let count = unsafe {
        let fp = htslib_rs::hts_open(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open input");
        let hdr = htslib_rs::sam_hdr_read(fp);
        assert!(!hdr.is_null(), "failed to read header");
        let rec = htslib_rs::bam_init1();
        assert!(!rec.is_null(), "failed to allocate record");

        let mut count = 0u64;
        if let Some(region) = &region {
            let idx = htslib_rs::sam_index_load(fp, path.as_ptr().cast());
            assert!(!idx.is_null(), "failed to load index");
            let itr = htslib_rs::sam_itr_querys(idx, hdr, region.as_ptr().cast());
            assert!(!itr.is_null(), "failed to build iterator");
            while htslib_rs::sam_itr_next(fp, itr, rec) >= 0 {
                count += 1;
            }
            htslib_rs::hts_itr_destroy(itr);
            htslib_rs::hts_idx_destroy(idx);
        } else {
            while htslib_rs::sam_read1(fp, hdr, rec) >= 0 {
                count += 1;
            }
        }

        htslib_rs::bam_destroy1(rec);
        htslib_rs::sam_hdr_destroy(hdr);
        assert_eq!(htslib_rs::hts_close(fp), 0);
        count
    };

    println!("{count}");
}
