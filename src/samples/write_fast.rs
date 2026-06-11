use crate::htslib_rs::{hts::htsFile, sam};
use std::io::Write;

// original: print_usage (htslib/samples/write_fast.c:39)
pub fn samples_write_fast_c_39_print_usage() {
    eprint!("Usage: write_fast <file> <sequence> [<qualities]\nAppends a fasta/fastq file.\n");
}

// original: main (htslib/samples/write_fast.c:51)
pub unsafe fn samples_write_fast_c_51_main(argv: &[&[u8]]) -> i32 {
    let mut ret = 1;
    let mut __out = std::io::stdout();
    let mut mode = [b'a', 0, 0, 0];
    let mut qual: &[u8] = &[];
    let mut name = [0u8; 256];

    let argc = argv.len();
    if !(3..=4).contains(&argc) {
        samples_write_fast_c_39_print_usage();
        return ret;
    }

    // argv entries are NUL-terminated C strings; use bytes up to the first NUL.
    let outname = &argv[1][..argv[1].iter().position(|&b| b == 0).unwrap_or(argv[1].len())];
    let data = &argv[2][..argv[2].iter().position(|&b| b == 0).unwrap_or(argv[2].len())];
    if argc == 4 {
        qual = &argv[3][..argv[3].iter().position(|&b| b == 0).unwrap_or(argv[3].len())];
        if data.len() != qual.len() {
            write!(__out, "Incorrect reference and quality data\n").unwrap();
            __out.flush().unwrap();
            return ret;
        }
    }

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
        __out.flush().unwrap();
        return ret;
    }

    // NUL-terminated copies for the still-raw C-ABI callees.
    let mut outname_c = outname.to_vec();
    outname_c.push(0);

    if sam::sam_open_mode(mode.as_mut_ptr().add(1).cast(), outname_c.as_ptr().cast(), std::ptr::null()) < 0 {
        write!(__out, "Invalid file name\n").unwrap();
        __out.flush().unwrap();
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let hfile = crate::htslib_rs::hfile::hopen(outname_c.as_ptr().cast(), mode.as_ptr().cast());
    if hfile.is_null() {
        write!(__out, "Could not open {}\n", String::from_utf8_lossy(outname)).unwrap();
        __out.flush().unwrap();
        sam::bam_destroy1(bamdata);
        return ret;
    }
    let outfile: *mut htsFile = crate::htslib_rs::hts::hts_hopen(hfile, outname_c.as_ptr().cast(), mode.as_ptr().cast());
    if outfile.is_null() {
        write!(__out, "Could not open {}\n", String::from_utf8_lossy(outname)).unwrap();
        __out.flush().unwrap();
        crate::htslib_rs::hfile::hclose(hfile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let label = format!("Test_{}", now);
    let n = label.len().min(name.len() - 1);
    name[..n].copy_from_slice(&label.as_bytes()[..n]);
    name[n] = 0;

    if sam::bam_set1(
        bamdata,
        label.len(),
        name.as_ptr().cast(),
        sam::BAM_FUNMAP as u16,
        -1,
        -1,
        0,
        0,
        std::ptr::null(),
        -1,
        -1,
        0,
        data.len(),
        data.as_ptr().cast(),
        if qual.is_empty() { std::ptr::null() } else { qual.as_ptr().cast() },
        0,
    ) < 0
    {
        write!(__out, "Failed to set data\n").unwrap();
    } else if sam::sam_c_4553_sam_write1(outfile, std::ptr::null(), bamdata) < 0 {
        write!(__out, "Failed to write data\n").unwrap();
    } else {
        ret = 0;
    }

    crate::htslib_rs::hts::hts_close(outfile);
    sam::bam_destroy1(bamdata);
    __out.flush().unwrap();
    ret
}
