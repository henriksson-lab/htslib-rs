use std::ffi::{c_char, c_int};

use crate::htslib_mini_rs::{hts, sam};

static mut TEST_FIELDARITH_NTESTS: c_int = 0;
static mut TEST_FIELDARITH_NFAILURES: c_int = 0;

// original: check (htslib/test/fieldarith.c:34)
pub unsafe fn test_fieldarith_c_34_check(
    aln: *const sam::bam1_t,
    testname: *const c_char,
    tag: *const c_char,
    value: c_int,
) {
    let aux = sam::bam_aux_get(aln, tag);
    if aux.is_null() {
        return;
    }

    TEST_FIELDARITH_NTESTS += 1;
    let refvalue = sam::bam_aux2i(aux);
    if value as i64 != refvalue {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s FAIL for %s: computed %d != %d expected\n".as_ptr(),
            testname,
            sam::bam_get_qname(aln),
            value,
            refvalue as c_int,
        );
        TEST_FIELDARITH_NFAILURES += 1;
    }
}

// original: main (htslib/test/fieldarith.c:48)
pub unsafe fn test_fieldarith_c_48_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let aln = sam::bam_init1();

    for i in 1..argc {
        let in_ = hts::hts_open(*argv.add(i as usize), c"r".as_ptr());
        if in_.is_null() {
            libc::perror(*argv.add(1));
            return 1;
        }

        let header = sam::sam_hdr_read(in_);
        while sam::sam_read1(in_, header, aln) >= 0 {
            test_fieldarith_c_34_check(
                aln,
                c"cigar2qlen".as_ptr(),
                c"XQ".as_ptr(),
                sam::bam_cigar2qlen((*aln).core.n_cigar as c_int, sam::bam_get_cigar(aln)) as c_int,
            );
            test_fieldarith_c_34_check(
                aln,
                c"cigar2rlen".as_ptr(),
                c"XR".as_ptr(),
                sam::bam_cigar2rlen((*aln).core.n_cigar as c_int, sam::bam_get_cigar(aln)) as c_int,
            );
            test_fieldarith_c_34_check(
                aln,
                c"endpos".as_ptr(),
                c"XE".as_ptr(),
                sam::bam_endpos(aln) as c_int,
            );
        }

        sam::sam_hdr_destroy(header);
        hts::hts_close(in_);
    }

    sam::bam_destroy1(aln);

    (TEST_FIELDARITH_NFAILURES > 0) as c_int
}
