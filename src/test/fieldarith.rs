use crate::htslib_rs::{hts, sam};

static mut TEST_FIELDARITH_NTESTS: i32 = 0;
static mut TEST_FIELDARITH_NFAILURES: i32 = 0;

// original: check (htslib/test/fieldarith.c:34)
pub unsafe fn test_fieldarith_c_34_check(
    aln: *const sam::bam1_t,
    testname: &[u8],
    tag: &[u8],
    value: i32,
) {
    let aux = sam::bam_aux_get(aln, tag.as_ptr());
    if aux.is_null() {
        return;
    }

    TEST_FIELDARITH_NTESTS += 1;
    let refvalue = sam::bam_aux2i(aux);
    if value as i64 != refvalue {
        eprintln!(
            "{} FAIL for {}: computed {} != {} expected",
            String::from_utf8_lossy(testname),
            String::from_utf8_lossy(
                std::ffi::CStr::from_ptr(sam::bam_get_qname(aln).cast()).to_bytes()
            ),
            value,
            refvalue as i32,
        );
        TEST_FIELDARITH_NFAILURES += 1;
    }
}

// original: main (htslib/test/fieldarith.c:48)
pub unsafe fn test_fieldarith_c_48_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let aln = sam::bam_init1();

    for i in 1..argc {
        let in_ = hts::hts_open((*argv.add(i as usize)).cast(), c"r".as_ptr());
        if in_.is_null() {
            libc::perror(*argv.add(1).cast());
            return 1;
        }

        let header = sam::sam_hdr_read(in_);
        while sam::sam_read1(in_, header, aln) >= 0 {
            test_fieldarith_c_34_check(
                aln,
                b"cigar2qlen",
                b"XQ",
                sam::bam_cigar2qlen((*aln).core.n_cigar as i32, sam::bam_get_cigar(aln)) as i32,
            );
            test_fieldarith_c_34_check(
                aln,
                b"cigar2rlen",
                b"XR",
                sam::bam_cigar2rlen((*aln).core.n_cigar as i32, sam::bam_get_cigar(aln)) as i32,
            );
            test_fieldarith_c_34_check(
                aln,
                b"endpos",
                b"XE",
                sam::bam_endpos(aln) as i32,
            );
        }

        sam::sam_hdr_destroy(header);
        hts::hts_close(in_);
    }

    sam::bam_destroy1(aln);

    (TEST_FIELDARITH_NFAILURES > 0) as i32
}
