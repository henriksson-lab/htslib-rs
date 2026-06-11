use crate::htslib_rs::{hfile, hts, sam, vcf};
use std::ffi::c_char;

unsafe extern "C" {
    #[link_name = "hopen"]
    fn htslib_hopen_variadic(fname: *const c_char, mode: *const c_char, ...) -> *mut hts::hFILE;
}

// original: hts_close_or_abort (htslib/test/fuzz/hts_open_fuzzer.c:40)
pub unsafe fn test_fuzz_hts_open_fuzzer_c_40_hts_close_or_abort(file: *mut hts::htsFile) {
    if unsafe { hts::hts_close(file) } != 0 {
        unsafe { libc::abort() };
    }
}

// original: view_sam (htslib/test/fuzz/hts_open_fuzzer.c:46)
pub unsafe fn test_fuzz_hts_open_fuzzer_c_46_view_sam(
    data: *const u8,
    size: usize,
    mode: &[u8],
    close_abort: i32,
) {
    let mut copy = vec![0u8; size];
    copy.copy_from_slice(unsafe { std::slice::from_raw_parts(data, size) });

    let memfile = unsafe {
        htslib_hopen_variadic(c"mem:".as_ptr(), c"rb:".as_ptr(), copy.as_ptr(), size)
    };
    if memfile.is_null() {
        return;
    }

    let in_ = unsafe { hts::hts_hopen(memfile, c"data".as_ptr(), c"rb".as_ptr()) };
    if in_.is_null() {
        if unsafe { hfile::hclose(memfile) } != 0 {
            unsafe { libc::abort() };
        }
        return;
    }

    let out = unsafe { hts::hts_open(c"/dev/null".as_ptr(), mode.as_ptr().cast()) };
    if out.is_null() {
        unsafe { libc::abort() };
    }

    /*
    #ifdef FUZZ_FAI
        // Not critical if this doesn't work, but can test more if
        // we're in the right location.
        //
        // We can't rely on what the pwd is for the OSS-fuzz so we don't enable
        // this by default.
        if (hts_set_fai_filename(out, "../c2.fa") < 0) {
            static int warned = 0;
            if (!warned) {
                warned = 1;
                fprintf(stderr, "Warning couldn't find the c2.fa file\n");
            }
        }
    #endif
    */

    let hdr = unsafe { sam::sam_hdr_read(in_) };
    if hdr.is_null() {
        if close_abort != 0 {
            unsafe { test_fuzz_hts_open_fuzzer_c_40_hts_close_or_abort(out) };
        } else {
            unsafe { hts::hts_close(out) };
        }
        unsafe { hts::hts_close(in_) };
        return;
    }

    // This will force the header to be parsed.
    unsafe { sam::sam_hdr_count_lines(&mut *hdr, b"SQ") };

    if unsafe { sam::sam_hdr_write(out, hdr) } != 0 {
        unsafe { sam::sam_hdr_destroy(hdr) };
        if close_abort != 0 {
            unsafe { test_fuzz_hts_open_fuzzer_c_40_hts_close_or_abort(out) };
        } else {
            unsafe { hts::hts_close(out) };
        }
        unsafe { hts::hts_close(in_) };
        return;
    }

    let b = unsafe { sam::bam_init1() };
    if !b.is_null() {
        while unsafe { sam::sam_read1(in_, hdr, b) } >= 0 {
            if unsafe { sam::sam_c_4553_sam_write1(out, hdr, b) } < 0 {
                break;
            }
        }
        unsafe { sam::bam_destroy1(b) };
    }

    unsafe { sam::sam_hdr_destroy(hdr) };
    if close_abort != 0 {
        unsafe { test_fuzz_hts_open_fuzzer_c_40_hts_close_or_abort(out) };
    } else {
        unsafe { hts::hts_close(out) };
    }
    unsafe { hts::hts_close(in_) };
}

// original: view_vcf (htslib/test/fuzz/hts_open_fuzzer.c:121)
pub unsafe fn test_fuzz_hts_open_fuzzer_c_121_view_vcf(
    data: *const u8,
    size: usize,
    mode: &[u8],
) {
    let mut copy = vec![0u8; size];
    copy.copy_from_slice(unsafe { std::slice::from_raw_parts(data, size) });

    let memfile = unsafe {
        htslib_hopen_variadic(c"mem:".as_ptr(), c"rb:".as_ptr(), copy.as_ptr(), size)
    };
    if memfile.is_null() {
        return;
    }

    let in_ = unsafe { hts::hts_hopen(memfile, c"data".as_ptr(), c"rb".as_ptr()) };
    if in_.is_null() {
        if unsafe { hfile::hclose(memfile) } != 0 {
            unsafe { libc::abort() };
        }
        return;
    }

    let out = unsafe { hts::hts_open(c"/dev/null".as_ptr(), mode.as_ptr().cast()) };
    if out.is_null() {
        unsafe { libc::abort() };
    }

    let hdr = unsafe { vcf::bcf_hdr_read(in_) };
    if hdr.is_null() {
        unsafe { test_fuzz_hts_open_fuzzer_c_40_hts_close_or_abort(out) };
        unsafe { hts::hts_close(in_) };
        return;
    }

    if unsafe { vcf::bcf_hdr_write(out, hdr) } != 0 {
        unsafe { vcf::bcf_hdr_destroy(hdr) };
        unsafe { test_fuzz_hts_open_fuzzer_c_40_hts_close_or_abort(out) };
        unsafe { hts::hts_close(in_) };
        return;
    }

    let rec = unsafe { vcf::bcf_init() };
    if !rec.is_null() {
        while unsafe { vcf::bcf_read(in_, hdr, rec) } >= 0 {
            if unsafe { vcf::bcf_write(out, hdr, rec) } < 0 {
                break;
            }
        }
        unsafe { vcf::bcf_destroy(rec) };
    }

    unsafe { vcf::bcf_hdr_destroy(hdr) };
    unsafe { test_fuzz_hts_open_fuzzer_c_40_hts_close_or_abort(out) };
    unsafe { hts::hts_close(in_) };
}

// original: LLVMFuzzerTestOneInput (htslib/test/fuzz/hts_open_fuzzer.c:171)
pub unsafe fn test_fuzz_hts_open_fuzzer_c_171_LLVMFuzzerTestOneInput(
    data: *const u8,
    size: usize,
) -> i32 {
    // Only data as a mem file purely for purposes of determining format
    let mut copy = vec![0u8; size];
    copy.copy_from_slice(unsafe { std::slice::from_raw_parts(data, size) });
    // hopen does not take ownership of `copy`, but hts_hopen does.
    let memfile = unsafe {
        htslib_hopen_variadic(c"mem:".as_ptr(), c"rb:".as_ptr(), copy.as_ptr(), size)
    };
    if memfile.is_null() {
        return 0;
    }

    let ht_file = unsafe { hts::hts_hopen(memfile, c"data".as_ptr(), c"rb".as_ptr()) };
    if ht_file.is_null() {
        if unsafe { hfile::hclose(memfile) } != 0 {
            unsafe { libc::abort() };
        }
        return 0;
    }
    let ftype = unsafe { (*ht_file).format.category };
    unsafe { hts::hts_close(ht_file) };

    // Now repeat a read-write loop multiple times per input, testing
    // encoding in all output formats.
    // (Although we could just ignore ftype and do all 5 for all inputs)
    match ftype {
        hts::HTS_FORMAT_SEQUENCE_DATA => {
            unsafe { test_fuzz_hts_open_fuzzer_c_46_view_sam(data, size, b"w\0", 1) };
            unsafe { test_fuzz_hts_open_fuzzer_c_46_view_sam(data, size, b"wb\0", 1) };
            unsafe { test_fuzz_hts_open_fuzzer_c_46_view_sam(data, size, b"wc\0", 0) };
        }
        hts::HTS_FORMAT_VARIANT_DATA => {
            unsafe { test_fuzz_hts_open_fuzzer_c_121_view_vcf(data, size, b"w\0") };
            unsafe { test_fuzz_hts_open_fuzzer_c_121_view_vcf(data, size, b"wb\0") };
        }
        _ => {}
    }
    0
}
