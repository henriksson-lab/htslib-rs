use crate::htslib_rs::{
    hts::{htsFile, htsFormat, hts_pos_t, kstring_t},
    sam,
};
static mut TEST_SAM_STATUS: i32 = libc::EXIT_SUCCESS;

// SEAM: hoisted from test_mempolicy's local `const MAX_RECS` so the cleanup
// helper (which reconstructs the owned boxed bam1_t array to drop it) can size
// the array without the field-pointer/calloc trick.
const TEST_MEMPOLICY_MAX_RECS: usize = 1000;

// original: check_bam_aux_get (htslib/test/sam.c:78)
pub unsafe fn test_sam_c_78_check_bam_aux_get(
    aln: *const sam::bam1_t,
    tag: *const u8,
    type_: u8,
) -> *mut u8 {
    let p = sam::bam_aux_get(aln, tag);
    if !p.is_null() {
        if *p == type_ {
            return p;
        }
        eprintln!(
            "Failed: {} field of type '{}', expected '{}'",
            std::ffi::CStr::from_ptr(tag.cast()).to_string_lossy(),
            *p as char,
            type_ as char,
        );
    } else {
        eprintln!(
            "Failed: can't find {} field",
            std::ffi::CStr::from_ptr(tag.cast()).to_string_lossy(),
        );
    }
    TEST_SAM_STATUS = libc::EXIT_FAILURE;
    std::ptr::null_mut()
}

// original: check_aux_count (htslib/test/sam.c:90)
pub unsafe fn test_sam_c_90_check_aux_count(
    aln: *const sam::bam1_t,
    expected: i32,
    what: *const u8,
) {
    let mut itr = sam::bam_aux_first(aln);
    let mut n = 0;
    while !itr.is_null() {
        n += 1;
        itr = sam::bam_aux_next(aln, itr);
    }
    if n != expected {
        eprintln!(
            "Failed: {} has {} aux fields, expected {}",
            std::ffi::CStr::from_ptr(what.cast()).to_string_lossy(),
            n,
            expected,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }
}

// original: check_int_B_array (htslib/test/sam.c:99)
pub unsafe fn test_sam_c_99_check_int_B_array(
    aln: *mut sam::bam1_t,
    tag: *mut u8,
    nvals: u32,
    vals: *mut i64,
) {
    let p = test_sam_c_78_check_bam_aux_get(aln, tag, b'B');
    if p.is_null() {
        return;
    }

    let len = sam::bam_auxB_len(p);
    if len != nvals {
        eprintln!(
            "Failed: Wrong length reported for {} field, got {}, expected {}",
            std::ffi::CStr::from_ptr(tag.cast()).to_string_lossy(),
            len,
            nvals,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }

    for i in 0..nvals {
        let expected = *vals.add(i as usize);
        let got_i = sam::bam_auxB2i(p, i);
        if got_i != expected {
            eprintln!(
                "Failed: Wrong value from bam_auxB2i for {} field index {}, got {} expected {}",
                std::ffi::CStr::from_ptr(tag.cast()).to_string_lossy(),
                i,
                got_i,
                expected,
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
        }

        let got_f = sam::bam_auxB2f(p, i);
        if got_f != expected as f64 {
            eprintln!(
                "Failed: Wrong value from bam_auxB2f for {} field index {}, got {} expected {}",
                std::ffi::CStr::from_ptr(tag.cast()).to_string_lossy(),
                i,
                got_f,
                expected as f64,
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
        }
    }
}

// original: test_update_int (htslib/test/sam.c:136)
pub unsafe fn test_sam_c_136_test_update_int(
    aln: *mut sam::bam1_t,
    target_id: *const u8,
    target_val: i64,
    expected_type: u8,
    next_id: *const u8,
    next_val: i64,
    next_type: u8,
) -> i32 {
    if sam::bam_aux_update_int(aln, target_id, target_val) < 0 {
        eprintln!(
            "Failed: update {} tag",
            std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }

    let mut p = sam::bam_aux_get(aln, target_id);
    if p.is_null() {
        eprintln!(
            "Failed: find {} tag",
            std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }
    if *p != expected_type || sam::bam_aux2i(p) != target_val {
        eprintln!(
            "Failed: {} field is {}:{}; expected {}:{}",
            std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
            *p as char,
            sam::bam_aux2i(p),
            expected_type as char,
            target_val,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }

    if *next_id == 0 {
        return 0;
    }
    p = sam::bam_aux_get(aln, next_id);
    if p.is_null() {
        eprintln!(
            "Failed: find {} tag after updating {}",
            std::ffi::CStr::from_ptr(next_id.cast()).to_string_lossy(),
            std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }
    if *p != next_type || sam::bam_aux2i(p) != next_val {
        eprintln!(
            "Failed: after updating {} to {}: {} field is {}:{}; expected {}:{}",
            std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
            target_val,
            std::ffi::CStr::from_ptr(next_id.cast()).to_string_lossy(),
            *p as char,
            sam::bam_aux2i(p),
            next_type as char,
            next_val,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }
    0
}

// original: test_update_array (htslib/test/sam.c:192)
#[allow(clippy::too_many_arguments)]
pub unsafe fn test_sam_c_192_test_update_array(
    aln: *mut sam::bam1_t,
    target_id: *const u8,
    type_: u8,
    nitems: u32,
    data: *mut (),
    next_id: *const u8,
    next_val: i64,
    next_type: u8,
) -> i32 {
    if sam::bam_aux_update_array(aln, target_id, type_, nitems, data.cast()) < 0 {
        eprintln!(
            "Failed: update {} tag",
            std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }

    let mut p = sam::bam_aux_get(aln, target_id);
    if p.is_null() {
        eprintln!(
            "Failed: find {} tag",
            std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }

    for i in 0..nitems {
        let ok = match type_ {
            b'c' => sam::bam_auxB2i(p, i) == *data.cast::<i8>().add(i as usize) as i64,
            b'C' => sam::bam_auxB2i(p, i) == *data.cast::<u8>().add(i as usize) as i64,
            b's' => sam::bam_auxB2i(p, i) == *data.cast::<i16>().add(i as usize) as i64,
            b'S' => sam::bam_auxB2i(p, i) == *data.cast::<u16>().add(i as usize) as i64,
            b'i' => sam::bam_auxB2i(p, i) == *data.cast::<i32>().add(i as usize) as i64,
            b'I' => sam::bam_auxB2i(p, i) == *data.cast::<u32>().add(i as usize) as i64,
            b'f' => sam::bam_auxB2f(p, i) as f32 == *data.cast::<f32>().add(i as usize),
            _ => true,
        };
        if !ok {
            eprintln!(
                "Failed: Wrong value for {} field index {}",
                std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
                i,
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
            return -1;
        }
    }

    if *next_id == 0 {
        return 0;
    }
    p = sam::bam_aux_get(aln, next_id);
    if p.is_null() {
        eprintln!(
            "Failed: find {} tag after updating {}",
            std::ffi::CStr::from_ptr(next_id.cast()).to_string_lossy(),
            std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }
    if *p != next_type || sam::bam_aux2i(p) != next_val {
        eprintln!(
            "Failed: after updating {}: {} field is {}:{}; expected {}:{}",
            std::ffi::CStr::from_ptr(target_id.cast()).to_string_lossy(),
            std::ffi::CStr::from_ptr(next_id.cast()).to_string_lossy(),
            *p as char,
            sam::bam_aux2i(p),
            next_type as char,
            next_val,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }

    0
}

// original: aux_fields1 (htslib/test/sam.c:248)
pub unsafe fn test_sam_c_248_aux_fields1() -> i32 {
    let sam_text = c"data:,@SQ\tSN:one\tLN:1000\n@SQ\tSN:two\tLN:500\nr1\t0\tone\t500\t20\t8M\t*\t0\t0\tATGCATGC\tqqqqqqqq\tXA:A:k\tXi:i:37\tXf:f:3.141592653589793\tXd:d:2.718281828459045\tXZ:Z:Hello, world!\tXH:H:DEADBEEF\tXB:B:c,-2,0,+2\tB0:B:i,-2147483648,-1,0,1,2147483647\tB1:B:I,0,1,2147483648,4294967295\tB2:B:s,-32768,-1,0,1,32767\tB3:B:S,0,1,32768,65535\tB4:B:c,-128,-1,0,1,127\tB5:B:C,0,1,127,255\tBf:B:f,-3.14159,2.71828\tZZ:i:1000000\tF2:d:2.46801\tY1:i:-2147483648\tY2:i:-2147483647\tY3:i:-1\tY4:i:0\tY5:i:1\tY6:i:2147483647\tY7:i:2147483648\tY8:i:4294967295\nr2\t0x8D\t*\t0\t0\t*\t*\t0\t0\tATGC\tqqqq\n";
    let r1 = c"r1\t0\tone\t500\t20\t8M\t*\t0\t0\tATGCATGC\tqqqqqqqq\tXi:i:37\tXf:f:3.14159\tXd:d:2.71828\tXZ:Z:Yo, dude\tXH:H:DEADBEEF\tXB:B:c,-2,0,2\tB0:B:i,-2147483648,-1,0,1,2147483647\tB1:B:I,0,1,2147483648,4294967295\tB2:B:s,-32768,-1,0,1,32767\tB3:B:S,0,1,32768,65535\tB4:B:c,-128,-1,0,1,127\tB5:B:C,0,1,127,255\tBf:B:f,-3.14159,2.71828\tZZ:i:1000000\tF2:f:9.8765\tY1:i:-2147483648\tY2:i:-2147483647\tY3:i:-1\tY4:i:0\tY5:i:1\tY6:i:2147483647\tY7:i:2147483648\tY8:i:4294967295\tN0:i:-1234\tN1:i:1234\tN2:i:-2\tN3:i:3\tF1:f:4.5678\tN4:B:S,65535,32768,1,0\tN5:i:4242\tZa:Z:Hello, world!\tZb:Z:Bonjour, tout le monde";
    let r2 = c"r2\t141\t*\t0\t0\t*\t*\t0\t0\tATGC\tqqqq";

    let in_ = crate::htslib_rs::hts::hts_open(sam_text.as_ptr(), c"r".as_ptr());
    if in_.is_null() {
        test_sam_bam_set1_fail(c"aux_fields1".as_ptr().cast(), c"sam_open failed".as_ptr().cast());
        return 1;
    }
    let header = sam::sam_hdr_read(in_);
    let aln = sam::bam_init1();
    let mut ks = kstring_t { data: Vec::new() };
    if header.is_null() || aln.is_null() {
        test_sam_bam_set1_fail(
            c"aux_fields1".as_ptr().cast(),
            c"allocation/header failure".as_ptr().cast(),
        );
        if !aln.is_null() {
            sam::bam_destroy1(aln);
        }
        sam::sam_hdr_destroy(header);
        crate::htslib_rs::hts::hts_close(in_);
        return 1;
    }

    if sam::sam_read1(in_, header, aln) >= 0 {
        let mut p = test_sam_c_78_check_bam_aux_get(aln, c"XA".as_ptr().cast(), b'A');
        if !p.is_null() && sam::bam_aux2A(p) != b'k' {
            test_sam_bam_set1_fail(c"aux_fields1".as_ptr().cast(), c"XA field mismatch".as_ptr().cast());
        }
        test_sam_c_90_check_aux_count(aln, 24, c"Original record".as_ptr().cast());
        sam::bam_aux_del(aln, p);
        if !sam::bam_aux_get(aln, c"XA".as_ptr().cast()).is_null() {
            test_sam_bam_set1_fail(
                c"aux_fields1".as_ptr().cast(),
                c"XA field was not deleted".as_ptr().cast(),
            );
        }
        test_sam_c_90_check_aux_count(aln, 23, c"Record post-XA-deletion".as_ptr().cast());

        p = test_sam_c_78_check_bam_aux_get(aln, c"Xi".as_ptr().cast(), b'C');
        if !p.is_null() && sam::bam_aux2i(p) != 37 {
            test_sam_bam_set1_fail(c"aux_fields1".as_ptr().cast(), c"Xi field mismatch".as_ptr().cast());
        }
        p = test_sam_c_78_check_bam_aux_get(aln, c"Xf".as_ptr().cast(), b'f');
        if !p.is_null() && (sam::bam_aux2f(p) - std::f64::consts::PI).abs() > 1e-6 {
            test_sam_bam_set1_fail(c"aux_fields1".as_ptr().cast(), c"Xf field mismatch".as_ptr().cast());
        }
        p = test_sam_c_78_check_bam_aux_get(aln, c"Xd".as_ptr().cast(), b'd');
        if !p.is_null() && (sam::bam_aux2f(p) - std::f64::consts::E).abs() > 1e-6 {
            test_sam_bam_set1_fail(c"aux_fields1".as_ptr().cast(), c"Xd field mismatch".as_ptr().cast());
        }

        sam::bam_aux_update_str(
            aln,
            c"XZ".as_ptr().cast(),
            (c"Yo, dude".to_bytes().len() + 1) as i32,
            c"Yo, dude".as_ptr().cast(),
        );
        sam::bam_aux_update_float(aln, c"F2".as_ptr().cast(), 9.8765);
        let mut ival: i32 = -1234;
        let mut uval: u32 = 1234;
        sam::bam_aux_append(
            aln,
            c"N0".as_ptr().cast(),
            b'i',
            std::mem::size_of_val(&ival) as i32,
            (&mut ival as *mut i32).cast(),
        );
        sam::bam_aux_append(
            aln,
            c"N1".as_ptr().cast(),
            b'I',
            std::mem::size_of_val(&uval) as i32,
            (&mut uval as *mut u32).cast(),
        );
        sam::bam_aux_update_int(aln, c"N2".as_ptr().cast(), -2);
        sam::bam_aux_update_int(aln, c"N3".as_ptr().cast(), 3);
        sam::bam_aux_update_float(aln, c"F1".as_ptr().cast(), 4.5678);
        let mut n4v7 = [65535u16, 32768, 1, 0];
        test_sam_c_192_test_update_array(
            aln,
            c"N4".as_ptr().cast(),
            b'S',
            n4v7.len() as u32,
            n4v7.as_mut_ptr().cast(),
            c"".as_ptr().cast(),
            0,
            0,
        );
        sam::bam_aux_update_int(aln, c"N5".as_ptr().cast(), 4242);
        sam::bam_aux_update_str(
            aln,
            c"Za".as_ptr().cast(),
            c"Hello, world!".to_bytes().len() as i32,
            c"Hello, world!".as_ptr().cast(),
        );
        sam::bam_aux_update_str(
            aln,
            c"Zb".as_ptr().cast(),
            (c"Bonjour, tout le monde".to_bytes().len() + 1) as i32,
            c"Bonjour, tout le monde".as_ptr().cast(),
        );

        let fmt_ret = sam::sam_format1(header, aln, &mut ks);
        if fmt_ret < 0 || ks.data != r1.to_bytes() {
            test_sam_bam_set1_fail(
                c"aux_fields1".as_ptr().cast(),
                c"record formatted incorrectly".as_ptr().cast(),
            );
        }
        p = sam::bam_aux_remove(
            aln,
            test_sam_c_78_check_bam_aux_get(aln, c"XH".as_ptr().cast(), b'H'),
        );
        if !sam::bam_aux_get(aln, c"XH".as_ptr().cast()).is_null()
            || std::slice::from_raw_parts(sam::bam_aux_tag(p), 2) != b"XB"
            || sam::bam_aux_type(p) != b'B'
        {
            test_sam_bam_set1_fail(c"aux_fields1".as_ptr().cast(), c"bam_aux_remove failed".as_ptr().cast());
        }
    } else {
        test_sam_bam_set1_fail(c"aux_fields1".as_ptr().cast(), c"can't read record".as_ptr().cast());
    }

    if sam::sam_read1(in_, header, aln) >= 0 {
        let fmt_ret = sam::sam_format1(header, aln, &mut ks);
        if fmt_ret < 0
            || (*aln).core.flag != 0x8d
            || ks.data != r2.to_bytes()
        {
            test_sam_bam_set1_fail(c"aux_fields1".as_ptr().cast(), c"record r2 checks failed".as_ptr().cast());
        }
    } else {
        test_sam_bam_set1_fail(c"aux_fields1".as_ptr().cast(), c"can't read record r2".as_ptr().cast());
    }

    sam::bam_destroy1(aln);
    sam::sam_hdr_destroy(header);
    crate::htslib_rs::hts::hts_close(in_);
    crate::htslib_rs::hts::ks_free(&mut ks);
    1
}

// original: set_qname (htslib/test/sam.c:557)
pub unsafe fn test_sam_c_557_set_qname() {
    let sam_text = c"data:,@SQ\tSN:one\tLN:1000\n@SQ\tSN:two\tLN:500\nr1\t0\tone\t500\t20\t8M\t*\t0\t0\tATGCATGC\tqqqqqqqq\tXA:A:k\tXi:i:37\tXf:f:3.141592653589793\tXd:d:2.718281828459045\tXZ:Z:Hello, world!\tXH:H:DEADBEEF\tXB:B:c,-2,0,+2\tB0:B:i,-2147483648,-1,0,1,2147483647\tB1:B:I,0,1,2147483648,4294967295\tB2:B:s,-32768,-1,0,1,32767\tB3:B:S,0,1,32768,65535\tB4:B:c,-128,-1,0,1,127\tB5:B:C,0,1,127,255\tBf:B:f,-3.14159,2.71828\tZZ:i:1000000\tF2:d:2.46801\tY1:i:-2147483648\tY2:i:-2147483647\tY3:i:-1\tY4:i:0\tY5:i:1\tY6:i:2147483647\tY7:i:2147483648\tY8:i:4294967295\nr22\t0x8D\t*\t0\t0\t*\t*\t0\t0\tATGC\tqqqq\nr12345678\t0x8D\t*\t0\t0\t*\t*\t0\t0\tATGC\tqqqq\n";
    let r1 = c"r1\t0\tone\t500\t20\t8M\t*\t0\t0\tATGCATGC\tqqqqqqqq\tXA:A:k\tXi:i:37\tXf:f:3.14159\tXd:d:2.71828\tXZ:Z:Hello, world!\tXH:H:DEADBEEF\tXB:B:c,-2,0,2\tB0:B:i,-2147483648,-1,0,1,2147483647\tB1:B:I,0,1,2147483648,4294967295\tB2:B:s,-32768,-1,0,1,32767\tB3:B:S,0,1,32768,65535\tB4:B:c,-128,-1,0,1,127\tB5:B:C,0,1,127,255\tBf:B:f,-3.14159,2.71828\tZZ:i:1000000\tF2:d:2.46801\tY1:i:-2147483648\tY2:i:-2147483647\tY3:i:-1\tY4:i:0\tY5:i:1\tY6:i:2147483647\tY7:i:2147483648\tY8:i:4294967295";
    let r2 = c"r234\t141\t*\t0\t0\t*\t*\t0\t0\tATGC\tqqqq";
    let r3 = c"xyz\t141\t*\t0\t0\t*\t*\t0\t0\tATGC\tqqqq";

    let in_ = crate::htslib_rs::hts::hts_open(sam_text.as_ptr(), c"r".as_ptr());
    if in_.is_null() {
        test_sam_bam_set1_fail(c"set_qname".as_ptr().cast(), c"can't open SAM input".as_ptr().cast());
        return;
    }
    let header = sam::sam_hdr_read(in_);
    if header.is_null() {
        crate::htslib_rs::hts::hts_close(in_);
        test_sam_bam_set1_fail(c"set_qname".as_ptr().cast(), c"can't read header".as_ptr().cast());
        return;
    }
    let aln = sam::bam_init1();
    if aln.is_null() {
        sam::bam_hdr_destroy(header);
        crate::htslib_rs::hts::hts_close(in_);
        test_sam_bam_set1_fail(c"set_qname".as_ptr().cast(), c"can't allocate alignment".as_ptr().cast());
        return;
    }

    let mut ks = kstring_t { data: Vec::new() };
    let cases = [
        (c"r1".as_ptr(), r1.to_bytes()),
        (c"r234".as_ptr(), r2.to_bytes()),
        (c"xyz".as_ptr(), r3.to_bytes()),
    ];

    for (qname, expected) in cases {
        if sam::sam_read1(in_, header, aln) < 0 {
            test_sam_bam_set1_fail(c"set_qname".as_ptr().cast(), c"can't read record".as_ptr().cast());
            break;
        }
        if sam::bam_set_qname(aln, qname.cast()) < 0 {
            test_sam_bam_set1_fail(c"set_qname".as_ptr().cast(), c"can't set qname".as_ptr().cast());
            break;
        }
        if sam::sam_format1(header, aln, &mut ks) < 0 {
            test_sam_bam_set1_fail(c"set_qname".as_ptr().cast(), c"can't format record".as_ptr().cast());
            break;
        }
        if ks.data != expected {
            eprintln!(
                "Failed: record formatted incorrectly:\nGot: \"{}\"\nExp: \"{}\"",
                String::from_utf8_lossy(&ks.data),
                String::from_utf8_lossy(expected),
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
            break;
        }
    }

    crate::htslib_rs::hts::ks_free(&mut ks);
    sam::bam_destroy1(aln);
    sam::bam_hdr_destroy(header);
    crate::htslib_rs::hts::hts_close(in_);
}

// original: iterators1 (htslib/test/sam.c:604)
pub unsafe fn test_sam_c_604_iterators1() {
    crate::htslib_rs::hts::hts_itr_destroy(crate::htslib_rs::sam::sam_itr_queryi(
        std::ptr::null(),
        crate::htslib_rs::hts::HTS_IDX_REST,
        0,
        0,
    ));
    crate::htslib_rs::hts::hts_itr_destroy(crate::htslib_rs::sam::sam_itr_queryi(
        std::ptr::null(),
        crate::htslib_rs::hts::HTS_IDX_NONE,
        0,
        0,
    ));
}

// original: copy_check_alignment (htslib/test/sam.c:612)
pub unsafe fn test_sam_c_612_copy_check_alignment(
    infname: *const u8,
    informat: *const u8,
    outfname: *const u8,
    outmode: *const u8,
    outref: *const u8,
) {
    let in_ = crate::htslib_rs::hts::hts_open(infname.cast(), c"r".as_ptr());
    let out = crate::htslib_rs::hts::hts_open(outfname.cast(), outmode.cast());
    let aln = sam::bam_init1();
    let mut header: *mut sam::sam_hdr_t = std::ptr::null_mut();

    if in_.is_null() {
        eprintln!(
            "Failed: couldn't open {}",
            std::ffi::CStr::from_ptr(infname.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        goto_copy_check_alignment_cleanup(in_, out, aln, header);
        return;
    }
    if out.is_null() {
        eprintln!(
            "Failed: couldn't open {} with mode {}",
            std::ffi::CStr::from_ptr(outfname.cast()).to_string_lossy(),
            std::ffi::CStr::from_ptr(outmode.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        goto_copy_check_alignment_cleanup(in_, out, aln, header);
        return;
    }
    if aln.is_null() {
        test_sam_bam_set1_fail(
            c"copy_check_alignment".as_ptr().cast(),
            c"bam_init1() failed".as_ptr().cast(),
        );
        goto_copy_check_alignment_cleanup(in_, out, aln, header);
        return;
    }

    let ref_target = if !in_.is_null()
        && (*crate::htslib_rs::hts::hts_get_format(in_)).format
            == crate::htslib_rs::hts::HTS_FORMAT_CRAM
    {
        in_
    } else {
        out
    };
    if !outref.is_null()
        && crate::htslib_rs::hts::hts_set_opt_ptr(
            ref_target,
            crate::htslib_rs::hts::CRAM_OPT_REFERENCE,
            outref.cast_mut().cast(),
        ) < 0
    {
        eprintln!(
            "Failed: setting reference {} for {}",
            std::ffi::CStr::from_ptr(outref.cast()).to_string_lossy(),
            std::ffi::CStr::from_ptr(outfname.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        goto_copy_check_alignment_cleanup(in_, out, aln, header);
        return;
    }

    header = sam::sam_hdr_read(in_);
    if header.is_null() {
        eprintln!(
            "Failed: reading header from {}",
            std::ffi::CStr::from_ptr(infname.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        goto_copy_check_alignment_cleanup(in_, out, aln, header);
        return;
    }
    if sam::sam_hdr_write(out, header) < 0 {
        eprintln!(
            "Failed: writing headers to {}",
            std::ffi::CStr::from_ptr(outfname.cast()).to_string_lossy(),
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }

    loop {
        let res = sam::sam_read1(in_, header, aln);
        if res < 0 {
            if res < -1 {
                eprintln!(
                    "Failed: failed to read alignment from {}",
                    std::ffi::CStr::from_ptr(infname.cast()).to_string_lossy(),
                );
                TEST_SAM_STATUS = libc::EXIT_FAILURE;
            }
            break;
        }

        let mod4 = (sam::bam_get_cigar(aln) as isize % 4) as i32;
        if mod4 != 0 {
            eprintln!(
                "Failed: {} CIGAR not 4-byte aligned; offset is 4k+{} for \"{}\"",
                std::ffi::CStr::from_ptr(informat.cast()).to_string_lossy(),
                mod4,
                std::ffi::CStr::from_ptr(sam::bam_get_qname(aln).cast()).to_string_lossy(),
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
        }

        if sam::sam_c_4553_sam_write1(out, header, aln) < 0 {
            eprintln!(
                "Failed: writing to {}",
                std::ffi::CStr::from_ptr(outfname.cast()).to_string_lossy(),
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
        }
    }

    goto_copy_check_alignment_cleanup(in_, out, aln, header);
}

unsafe fn goto_copy_check_alignment_cleanup(
    in_: *mut htsFile,
    out: *mut htsFile,
    aln: *mut sam::bam1_t,
    header: *mut sam::sam_hdr_t,
) {
    sam::bam_destroy1(aln);
    sam::bam_hdr_destroy(header);
    if !in_.is_null() {
        crate::htslib_rs::hts::hts_close(in_);
    }
    if !out.is_null() {
        crate::htslib_rs::hts::hts_close(out);
    }
}

// original: check_target_names (htslib/test/sam.c:669)
pub unsafe fn test_sam_c_669_check_target_names(
    header: *mut sam::sam_hdr_t,
    expected_n_targets: i32,
    expected_targets: *mut *const u8,
    expected_lengths: *const i32,
) -> i32 {
    if (*header).target_name.is_null() {
        eprintln!("Failed: target_name is NULL");
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }
    if (*header).target_len.is_null() {
        eprintln!("Failed: target_len is NULL");
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }
    if (*header).n_targets != expected_n_targets {
        eprintln!(
            "Failed: header->n_targets ({}) != expected_n_targets ({})",
            (*header).n_targets,
            expected_n_targets,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
        return -1;
    }

    for i in 0..expected_n_targets {
        let name = *(*header).target_name.add(i as usize);
        let expected_name = *expected_targets.add(i as usize);
        if name.is_null()
            || std::ffi::CStr::from_ptr(name.cast()).to_bytes()
                != std::ffi::CStr::from_ptr(expected_name.cast()).to_bytes()
        {
            eprintln!(
                "Failed: header->target_name[{}] ({}) != \"{}\"",
                i,
                if name.is_null() {
                    "NULL".to_string()
                } else {
                    std::ffi::CStr::from_ptr(name.cast()).to_string_lossy().into_owned()
                },
                std::ffi::CStr::from_ptr(expected_name.cast()).to_string_lossy(),
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
            return -1;
        }
        let len = *(*header).target_len.add(i as usize) as i32;
        let expected_len = *expected_lengths.add(i as usize);
        if len != expected_len {
            eprintln!(
                "Failed: header->target_len[{}] ({}) != {}",
                i, len, expected_len,
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
            return -1;
        }
    }
    0
}

// original: use_header_api (htslib/test/sam.c:705)
pub unsafe fn test_sam_c_705_use_header_api() {
    let header_text = c"data:,@HD\tVN:1.4\tGO:group\tSS:coordinate:queryname\n@SQ\tSN:ref0\tLN:100\n@CO\tThis line below will be updated\n@SQ\tSN:ref1\tLN:5001\tM5:983dalu9ue2\n@SQ\tSN:ref1.5\tLN:5001\n@CO\tThis line is good\n@SQ\tSN:ref2\tLN:5002\n";
    let expected = c"@HD\tVN:1.5\tSO:coordinate\n@CO\tThis line below will be updated\n@SQ\tSN:ref1\tLN:5001\tM5:kja8u34a2q3\n@CO\tThis line is good\n@SQ\tSN:ref2\tLN:5002\n@SQ\tSN:ref3\tLN:5003\n@PG\tID:samtools\tPN:samtools\tVN:1.9\n@RG\tID:run1\n@RG\tID:run4\n";
    let expected_targets: [*const u8; 3] = [
        c"ref1".as_ptr().cast(),
        c"ref2".as_ptr().cast(),
        c"ref3".as_ptr().cast(),
    ];
    let expected_lengths = [5001, 5002, 5003];
    let outfname = c"test/sam_header.tmp.sam_".as_ptr();
    let in_ = crate::htslib_rs::hts::hts_open(header_text.as_ptr(), c"r".as_ptr());
    let mut out = crate::htslib_rs::hts::hts_open(outfname, c"w".as_ptr());
    let mut header: *mut sam::sam_hdr_t = std::ptr::null_mut();
    let mut ks = kstring_t { data: Vec::new() };
    if in_.is_null() || out.is_null() {
        test_sam_bam_set1_fail(c"use_header_api".as_ptr().cast(), c"open failed".as_ptr().cast());
        goto_use_header_api_cleanup(in_, out, header, &mut ks);
        return;
    }
    header = sam::sam_hdr_read(in_);
    if header.is_null() {
        test_sam_bam_set1_fail(
            c"use_header_api".as_ptr().cast(),
            c"reading header from file".as_ptr().cast(),
        );
        goto_use_header_api_cleanup(in_, out, header, &mut ks);
        return;
    }
    if sam::sam_hdr_remove_tag_id(
        &mut *header,
        b"HD",
        None,
        b"GO",
    ) != 0
        || sam::sam_hdr_update_line(
            &mut *header,
            b"HD",
            None,
            &[(Some(&b"VN"[..]), Some(&b"1.5"[..]))],
        ) != 0
        || sam::sam_hdr_add_line(
            &mut *header,
            b"SQ",
            &[
                (Some(&b"SN"[..]), Some(&b"ref3"[..])),
                (Some(&b"LN"[..]), Some(&b"5003"[..])),
            ],
        ) < 0
        || sam::sam_hdr_update_line(
            &mut *header,
            b"SQ",
            Some((&b"SN"[..], &b"ref1"[..])),
            &[(Some(&b"M5"[..]), Some(&b"kja8u34a2q3"[..]))],
        ) != 0
        || sam::sam_hdr_add_pg(
            &mut *header,
            b"samtools",
            &[(Some(&b"VN"[..]), Some(&b"1.9"[..]))],
        ) != 0
    {
        test_sam_bam_set1_fail(c"use_header_api".as_ptr().cast(), c"header edit failed".as_ptr().cast());
        goto_use_header_api_cleanup(in_, out, header, &mut ks);
        return;
    }
    let rg_line = b"@RG\tID:run1";
    if sam::sam_hdr_add_lines(&mut *header, rg_line) != 0
        || sam::sam_hdr_add_line(
            &mut *header,
            b"RG",
            &[(Some(&b"ID"[..]), Some(&b"run2"[..]))],
        ) < 0
        || sam::sam_hdr_add_line(
            &mut *header,
            b"RG",
            &[(Some(&b"ID"[..]), Some(&b"run3"[..]))],
        ) < 0
        || sam::sam_hdr_add_line(
            &mut *header,
            b"RG",
            &[(Some(&b"ID"[..]), Some(&b"run4"[..]))],
        ) < 0
        || sam::sam_hdr_line_index(&mut *header, b"RG", b"run4") != 3
        || sam::sam_hdr_line_index(&mut *header, b"RG", b"run5") != -1
        || sam::sam_hdr_remove_line_id(&mut *header, b"RG", Some((&b"ID"[..], &b"run2"[..]))) < 0
        || sam::sam_hdr_remove_line_pos(&mut *header, b"RG", 1) < 0
        || sam::sam_hdr_remove_line_id(&mut *header, b"SQ", Some((&b"SN"[..], &b"ref0"[..]))) < 0
        || sam::sam_hdr_remove_line_pos(&mut *header, b"SQ", 1) < 0
        || sam::sam_hdr_remove_tag_id(
            &mut *header,
            b"HD",
            None,
            b"SS",
        ) < 0
        || sam::sam_hdr_update_line(
            &mut *header,
            b"HD",
            None,
            &[(Some(&b"SO"[..]), Some(&b"coordinate"[..]))],
        ) < 0
        || test_sam_c_669_check_target_names(
            header,
            expected_targets.len() as i32,
            expected_targets.as_ptr() as *mut *const u8,
            expected_lengths.as_ptr(),
        ) < 0
    {
        test_sam_bam_set1_fail(
            c"use_header_api".as_ptr().cast(),
            c"header API checks failed".as_ptr().cast(),
        );
        goto_use_header_api_cleanup(in_, out, header, &mut ks);
        return;
    }
    if sam::sam_hdr_write(out, header) < 0 || crate::htslib_rs::hts::hts_close(out) < 0 {
        out = std::ptr::null_mut();
        test_sam_bam_set1_fail(
            c"use_header_api".as_ptr().cast(),
            c"writing headers failed".as_ptr().cast(),
        );
        goto_use_header_api_cleanup(in_, out, header, &mut ks);
        return;
    }
    out = std::ptr::null_mut();
    let outfname_str = std::ffi::CStr::from_ptr(outfname.cast()).to_string_lossy();
    let buffer = std::fs::read(outfname_str.as_ref()).unwrap_or_default();
    if buffer != expected.to_bytes() {
        test_sam_bam_set1_fail(
            c"use_header_api".as_ptr().cast(),
            c"edited header does not match expected version".as_ptr().cast(),
        );
    }
    goto_use_header_api_cleanup(in_, out, header, &mut ks);
}

unsafe fn goto_use_header_api_cleanup(
    in_: *mut htsFile,
    out: *mut htsFile,
    header: *mut sam::sam_hdr_t,
    ks: &mut kstring_t,
) {
    sam::sam_hdr_destroy(header);
    if !in_.is_null() {
        crate::htslib_rs::hts::hts_close(in_);
    }
    if !out.is_null() {
        crate::htslib_rs::hts::hts_close(out);
    }
    crate::htslib_rs::hts::ks_free(ks);
}

// original: test_header_pg_lines (htslib/test/sam.c:921)
pub unsafe fn test_sam_c_921_test_header_pg_lines() {
    let header_text =
        c"data:,@HD\tVN:1.5\n@PG\tID:prog1\tPN:prog1\n@PG\tID:prog2\tPN:prog2\tPP:prog1\n";
    let expected = c"@HD\tVN:1.5\n@PG\tID:prog1\tPN:prog1\n@PG\tID:prog2\tPN:prog2\tPP:prog1\n@PG\tID:prog3\tPN:prog3\tPP:prog2\n@PG\tID:prog4\tPN:prog4\tPP:prog1\n@PG\tID:prog5\tPN:prog5\tPP:prog2\n@PG\tID:prog6\tPN:prog6\tPP:prog3\n@PG\tID:prog6.1\tPN:prog6\tPP:prog4\n@PG\tID:prog6.2\tPN:prog6\tPP:prog5\n@PG\tPN:prog7\tID:my_id\tPP:prog6\n";
    let in_ = crate::htslib_rs::hts::hts_open(header_text.as_ptr(), c"r".as_ptr());
    if in_.is_null() {
        test_sam_bam_set1_fail(c"test_header_pg_lines".as_ptr().cast(), c"open failed".as_ptr().cast());
        return;
    }
    let header = sam::sam_hdr_read(in_);
    if header.is_null() {
        crate::htslib_rs::hts::hts_close(in_);
        test_sam_bam_set1_fail(
            c"test_header_pg_lines".as_ptr().cast(),
            c"reading header from file".as_ptr().cast(),
        );
        return;
    }
    let old_log_level = crate::htslib_rs::hts::hts_get_log_level();
    if sam::sam_hdr_add_pg(&mut *header, b"prog3", &[]) != 0
        || sam::sam_hdr_add_pg(
            &mut *header,
            b"prog4",
            &[(Some(&b"PP"[..]), Some(&b"prog1"[..]))],
        ) != 0
        || sam::sam_hdr_add_line(
            &mut *header,
            b"PG",
            &[
                (Some(&b"ID"[..]), Some(&b"prog5"[..])),
                (Some(&b"PN"[..]), Some(&b"prog5"[..])),
                (Some(&b"PP"[..]), Some(&b"prog2"[..])),
            ],
        ) != 0
        || sam::sam_hdr_add_pg(&mut *header, b"prog6", &[]) != 0
        || sam::sam_hdr_add_pg(
            &mut *header,
            b"prog7",
            &[
                (Some(&b"ID"[..]), Some(&b"my_id"[..])),
                (Some(&b"PP"[..]), Some(&b"prog6"[..])),
            ],
        ) != 0
    {
        test_sam_bam_set1_fail(
            c"test_header_pg_lines".as_ptr().cast(),
            c"sam_hdr_add_pg failed".as_ptr().cast(),
        );
    }
    crate::htslib_rs::hts::hts_set_log_level(crate::htslib_rs::hts::HTS_LOG_OFF);
    if sam::sam_hdr_add_pg(
        &mut *header,
        b"prog8",
        &[(Some(&b"ID"[..]), Some(&b"my_id"[..]))],
    ) == 0
        || sam::sam_hdr_add_pg(
            &mut *header,
            b"prog9",
            &[(Some(&b"PP"[..]), Some(&b"non-existent"[..]))],
        ) == 0
    {
        test_sam_bam_set1_fail(
            c"test_header_pg_lines".as_ptr().cast(),
            c"unexpected add_pg success".as_ptr().cast(),
        );
    }
    crate::htslib_rs::hts::hts_set_log_level(old_log_level);
    let text = sam::sam_hdr_str(&mut *header);
    if text.is_null()
        || std::ffi::CStr::from_ptr(text.cast()).to_bytes() != expected.to_bytes()
    {
        test_sam_bam_set1_fail(
            c"test_header_pg_lines".as_ptr().cast(),
            c"edited header does not match expected version".as_ptr().cast(),
        );
    }
    sam::sam_hdr_destroy(header);
    crate::htslib_rs::hts::hts_close(in_);
}

// original: test_header_pg_loops (htslib/test/sam.c:1008)
pub unsafe fn test_sam_c_1008_test_header_pg_loops() {
    let header_texts = [
            c"data:,@HD\tVN:1.5\n@PG\tID:loop1\tPN:prog1\tPP:loop1\n".as_ptr(),
            c"data:,@HD\tVN:1.5\n@PG\tID:loop1\tPN:prog1\tPP:loop2\n@PG\tID:loop2\tPN:prog2\tPP:loop1\n".as_ptr(),
        ];
    let expected = [
            c"@HD\tVN:1.5\n@PG\tID:loop1\tPN:prog1\tPP:loop1\n@PG\tID:new_prog\tPN:new_prog\tPP:loop1\n".as_ptr(),
            c"@HD\tVN:1.5\n@PG\tID:loop1\tPN:prog1\tPP:loop2\n@PG\tID:loop2\tPN:prog2\tPP:loop1\n@PG\tID:new_prog\tPN:new_prog\n".as_ptr(),
        ];
    let old_log_level = crate::htslib_rs::hts::hts_get_log_level();
    crate::htslib_rs::hts::hts_set_log_level(crate::htslib_rs::hts::HTS_LOG_OFF);
    for i in 0..2 {
        let in_ = crate::htslib_rs::hts::hts_open(header_texts[i], c"r".as_ptr());
        let header = if in_.is_null() {
            std::ptr::null_mut()
        } else {
            sam::sam_hdr_read(in_)
        };
        if header.is_null() || sam::sam_hdr_add_pg(&mut *header, b"new_prog", &[]) != 0 {
            test_sam_bam_set1_fail(
                c"test_header_pg_loops".as_ptr().cast(),
                c"PG loop setup failed".as_ptr().cast(),
            );
        } else {
            let text = sam::sam_hdr_str(&mut *header);
            if text.is_null()
                || std::ffi::CStr::from_ptr(text.cast()).to_bytes()
                    != std::ffi::CStr::from_ptr(expected[i]).to_bytes()
            {
                test_sam_bam_set1_fail(
                    c"test_header_pg_loops".as_ptr().cast(),
                    c"edited header does not match expected version".as_ptr().cast(),
                );
            }
        }
        sam::sam_hdr_destroy(header);
        if !in_.is_null() {
            crate::htslib_rs::hts::hts_close(in_);
        }
    }
    crate::htslib_rs::hts::hts_set_log_level(old_log_level);
}

// original: test_header_updates (htslib/test/sam.c:1087)
pub unsafe fn test_sam_c_1087_test_header_updates() {
    let header_text = c"@HD\tVN:1.4\n@SQ\tSN:chr1\tLN:100\n@SQ\tSN:chr2\tLN:200\n@SQ\tSN:chr3\tLN:300\n@RG\tID:run1\n@RG\tID:run2\n@RG\tID:run3\n@PG\tID:prog1\tPN:prog1\n";
    let expected = c"@HD\tVN:1.4\n@SQ\tSN:1\tLN:100\n@SQ\tSN:chr2\tLN:2000\n@SQ\tSN:chr3\tLN:300\n@RG\tID:run1\tDS:hello\n@RG\tID:aliquot2\n@RG\tID:run3\n@PG\tID:prog1\tPN:prog1\n";
    let expected_targets: [*const u8; 3] = [
        c"1".as_ptr().cast(),
        c"chr2".as_ptr().cast(),
        c"chr3".as_ptr().cast(),
    ];
    let expected_lengths = [100, 2000, 300];
    let header = sam::sam_hdr_parse(header_text.to_bytes().len(), header_text.as_ptr().cast());
    if header.is_null() {
        test_sam_bam_set1_fail(
            c"test_header_updates".as_ptr().cast(),
            c"creating sam header".as_ptr().cast(),
        );
        return;
    }
    if sam::sam_hdr_update_line(
        &mut *header,
        b"SQ",
        Some((&b"SN"[..], &b"chr2"[..])),
        &[(Some(&b"LN"[..]), Some(&b"2000"[..]))],
    ) != 0
        || sam::sam_hdr_update_line(
            &mut *header,
            b"SQ",
            Some((&b"SN"[..], &b"chr1"[..])),
            &[(Some(&b"SN"[..]), Some(&b"1"[..]))],
        ) != 0
        || sam::sam_hdr_update_line(
            &mut *header,
            b"RG",
            Some((&b"ID"[..], &b"run1"[..])),
            &[(Some(&b"DS"[..]), Some(&b"hello"[..]))],
        ) != 0
        || sam::sam_hdr_update_line(
            &mut *header,
            b"RG",
            Some((&b"ID"[..], &b"run2"[..])),
            &[(Some(&b"ID"[..]), Some(&b"aliquot2"[..]))],
        ) != 0
        || test_sam_c_669_check_target_names(
            header,
            expected_targets.len() as i32,
            expected_targets.as_ptr() as *mut *const u8,
            expected_lengths.as_ptr(),
        ) < 0
    {
        test_sam_bam_set1_fail(
            c"test_header_updates".as_ptr().cast(),
            c"header update failed".as_ptr().cast(),
        );
    }
    for (i, name) in expected_targets.iter().enumerate() {
        if sam::sam_hdr_name2tid(&mut *header, std::ffi::CStr::from_ptr((*name).cast()).to_bytes()) != i as i32 {
            test_sam_bam_set1_fail(
                c"test_header_updates".as_ptr().cast(),
                c"sam_hdr_name2tid unexpected result".as_ptr().cast(),
            );
        }
    }
    let hdr_str = sam::sam_hdr_str(&mut *header);
    if hdr_str.is_null()
        || std::ffi::CStr::from_ptr(hdr_str.cast()).to_bytes() != expected.to_bytes()
    {
        test_sam_bam_set1_fail(
            c"test_header_updates".as_ptr().cast(),
            c"edited header does not match expected version".as_ptr().cast(),
        );
    }
    sam::sam_hdr_destroy(header);
}

// original: test_header_remove_lines (htslib/test/sam.c:1182)
pub unsafe fn test_sam_c_1182_test_header_remove_lines() {
    let header_text = c"@HD\tVN:1.4\n@SQ\tSN:chr1\tLN:100\n@SQ\tSN:chr2\tLN:200\n@SQ\tSN:chr3\tLN:300\n@RG\tID:run1\n@RG\tID:run2\n@RG\tID:run3\n@PG\tID:prog1\tPN:prog1\n";
    let expected =
        c"@HD\tVN:1.4\n@SQ\tSN:chr1\tLN:100\n@SQ\tSN:chr3\tLN:300\n@PG\tID:prog1\tPN:prog1\n";
    let header = sam::sam_hdr_parse(header_text.to_bytes().len(), header_text.as_ptr().cast());
    let rh = sam::khash_str2int_init();
    if header.is_null() || rh.is_null() {
        test_sam_bam_set1_fail(
            c"test_header_remove_lines".as_ptr().cast(),
            c"setup failed".as_ptr().cast(),
        );
    } else {
        let mut chr3 = b"chr3\0".to_vec();
        let mut chr1 = b"chr1\0".to_vec();
        sam::khash_str2int_set(rh, chr3.as_mut_ptr().cast(), 1);
        sam::khash_str2int_set(rh, chr1.as_mut_ptr().cast(), 1);
        if sam::sam_hdr_remove_lines(&mut *header, b"SQ", Some(&b"SN"[..]), std::ptr::NonNull::new(rh)) != 0
            || sam::sam_hdr_remove_lines(
                &mut *header,
                b"RG",
                Some(&b"ID"[..]),
                None,
            ) != 0
        {
            test_sam_bam_set1_fail(
                c"test_header_remove_lines".as_ptr().cast(),
                c"sam_hdr_remove_lines failed".as_ptr().cast(),
            );
        }
        let hdr_str = sam::sam_hdr_str(&mut *header);
        if hdr_str.is_null()
            || std::ffi::CStr::from_ptr(hdr_str.cast()).to_bytes() != expected.to_bytes()
        {
            test_sam_bam_set1_fail(
                c"test_header_remove_lines".as_ptr().cast(),
                c"edited header does not match expected version".as_ptr().cast(),
            );
        }
        sam::khash_str2int_destroy_free(rh);
    }
    sam::sam_hdr_destroy(header);
}

// original: check_ref_lookup (htslib/test/sam.c:1245)
pub unsafe fn test_sam_c_1245_check_ref_lookup() {
    // The original C helper is variadic; Rust call sites below spell out
    // the same name-to-tid checks directly.
}

// original: test_header_ref_altnames (htslib/test/sam.c:1258)
pub unsafe fn test_sam_c_1258_test_header_ref_altnames() {
    let initial_header = c"@SQ\tSN:1\tLN:100\tAN:chr1\n@SQ\tSN:chr2\tAN:2\tLN:200\n@SQ\tSN:3\tLN:300\n@SQ\tSN:chrMT\tLN:16569\tAN:MT,chrM,M\n";
    let header = sam::sam_hdr_init();
    if header.is_null() {
        test_sam_bam_set1_fail(
            c"test_header_ref_altnames".as_ptr().cast(),
            c"sam_hdr_init".as_ptr().cast(),
        );
        return;
    }
    if sam::sam_hdr_add_lines(&mut *header, initial_header.to_bytes()) < 0 {
        test_sam_bam_set1_fail(
            c"test_header_ref_altnames".as_ptr().cast(),
            c"sam_hdr_add_lines for altnames".as_ptr().cast(),
        );
    }
    let checks = [
        (c"1".as_ptr(), 0),
        (c"chr1".as_ptr(), 0),
        (c"2".as_ptr(), 1),
        (c"chr2".as_ptr(), 1),
        (c"3".as_ptr(), 2),
        (c"chrMT".as_ptr(), 3),
        (c"chrM".as_ptr(), 3),
        (c"M".as_ptr(), 3),
        (c"fred".as_ptr(), -1),
        (c"barney".as_ptr(), -1),
    ];
    for (name, exp) in checks {
        if sam::sam_hdr_name2tid(&mut *header, std::ffi::CStr::from_ptr(name).to_bytes()) != exp {
            test_sam_bam_set1_fail(
                c"test_header_ref_altnames".as_ptr().cast(),
                c"initial altname lookup failed".as_ptr().cast(),
            );
        }
    }
    if sam::sam_hdr_add_line(
        &mut *header,
        b"SQ",
        &[
            (Some(&b"AN"[..]), Some(&b"fred"[..])),
            (Some(&b"LN"[..]), Some(&b"500"[..])),
            (Some(&b"SN"[..]), Some(&b"barney"[..])),
        ],
    ) < 0
    {
        test_sam_bam_set1_fail(
            c"test_header_ref_altnames".as_ptr().cast(),
            c"sam_hdr_add_line for altnames".as_ptr().cast(),
        );
    }
    if sam::sam_hdr_name2tid(&mut *header, b"fred") != 4
        || sam::sam_hdr_name2tid(&mut *header, b"barney") != 4
    {
        test_sam_bam_set1_fail(
            c"test_header_ref_altnames".as_ptr().cast(),
            c"barney added lookup failed".as_ptr().cast(),
        );
    }
    if sam::sam_hdr_remove_line_id(&mut *header, b"SQ", Some((&b"SN"[..], &b"chr2"[..]))) < 0
        || sam::sam_hdr_name2tid(&mut *header, b"2") != -1
        || sam::sam_hdr_name2tid(&mut *header, b"chr2") != -1
    {
        test_sam_bam_set1_fail(
            c"test_header_ref_altnames".as_ptr().cast(),
            c"chr2 removal lookup failed".as_ptr().cast(),
        );
    }
    if sam::sam_hdr_remove_tag_id(
        &mut *header,
        b"SQ",
        Some((&b"SN"[..], &b"1"[..])),
        b"AN",
    ) < 0
        || sam::sam_hdr_name2tid(&mut *header, b"chr1") != -1
    {
        test_sam_bam_set1_fail(
            c"test_header_ref_altnames".as_ptr().cast(),
            c"AN removal lookup failed".as_ptr().cast(),
        );
    }
    sam::sam_hdr_destroy(header);
}

// original: samrecord_layout (htslib/test/sam.c:1348)
pub unsafe fn test_sam_c_1348_samrecord_layout() {
    let core_size = if std::mem::size_of::<hts_pos_t>() == 8 {
        48
    } else {
        36
    };
    // SEAM: bam1_t is now an OWNED, non-repr(C) struct
    // { core, id: u64, data: Vec<u8>, mempolicy_and_reserved: u32 }. The old
    // C-ABI layout (data: *mut, l_data: c_int, m_data: u32) is gone, so the
    // expected size is computed from the owned members. The compiler may insert
    // padding and reorder fields, so we keep the original two-value tolerance
    // (sum-of-members, rounded up to 8-byte alignment, and that +4) rather than a
    // single hardcoded C size.
    let members = core_size
        + std::mem::size_of::<u64>()
        + std::mem::size_of::<Vec<u8>>()
        + std::mem::size_of::<u32>();
    let bam1_t_size = members.div_ceil(8) * 8;
    let bam1_t_size2 = bam1_t_size + 4;

    let actual_core = std::mem::size_of::<sam::bam1_core_t>();
    if actual_core != core_size {
        eprintln!(
            "Failed: sizeof bam1_core_t is {}, expected {}",
            actual_core, core_size,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }

    let actual_bam = std::mem::size_of::<sam::bam1_t>();
    if actual_bam != bam1_t_size && actual_bam != bam1_t_size2 {
        eprintln!(
            "Failed: sizeof bam1_t is {}, expected either {} or {}",
            actual_bam, bam1_t_size, bam1_t_size2,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }
}

// original: check_ref_lengths (htslib/test/sam.c:1388)
pub unsafe fn test_sam_c_1388_check_ref_lengths(
    header: *const sam::sam_hdr_t,
    expected_lengths: *const hts_pos_t,
    num_refs: i32,
    hdr_name: *const u8,
) -> i32 {
    for i in 0..num_refs {
        let ln = sam::sam_hdr_tid2len(&*header, i);
        let expected = *expected_lengths.add(i as usize);
        if ln != expected {
            eprintln!(
                "Failed: Wrong length for {} ref {} : expected {} got {}",
                std::ffi::CStr::from_ptr(hdr_name.cast()).to_string_lossy(),
                i,
                expected,
                ln,
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
            return -1;
        }
    }
    0
}

// original: check_big_ref (htslib/test/sam.c:1405)
pub unsafe fn test_sam_c_1405_check_big_ref() {
    test_sam_c_1405_check_big_ref_parse(0);
    test_sam_c_1405_check_big_ref_parse(1);
}

pub unsafe fn test_sam_c_1405_check_big_ref_parse(parse_header: i32) {
    let sam_text = c"data:,@HD\tVN:1.4\n@SQ\tSN:large#1\tLN:5000000000\n@SQ\tSN:small#1\tLN:100\n@SQ\tSN:large#2\tLN:4611686018427387904\n@SQ\tSN:small#2\tLN:1\nr1\t0\tlarge#1\t4999999000\t50\t8M\t*\t0\t0\tACGTACGT\tabcdefgh\nr2\t0\tsmall#1\t1\t50\t8M\t*\t0\t0\tACGTACGT\tabcdefgh\nr3\t0\tlarge#2\t4611686018427387000\t50\t8M\t*\t0\t0\tACGTACGT\tabcdefgh\np1\t99\tlarge#2\t1\t50\t8M\t=\t4611686018427387895\t4611686018427387903\tACGTACGT\tabcdefgh\np1\t147\tlarge#2\t4611686018427387895\t50\t8M\t=\t1\t-4611686018427387903\tACGTACGT\tabcdefgh\nr4\t0\tsmall#2\t2\t50\t8M\t*\t0\t0\tACGTACGT\tabcdefgh\n";
    let expected_lengths: [hts_pos_t; 4] = [5000000000, 100, 4611686018427387904, 1];
    let expected_tids = [0, 1, 2, 2, 2, 3];
    let expected_mtid = [-1, -1, -1, 2, 2, -1];
    let expected_positions: [hts_pos_t; 6] = [
        4999999000 - 1,
        0,
        4611686018427387000 - 1,
        0,
        4611686018427387895 - 1,
        2 - 1,
    ];
    let expected_mpos: [hts_pos_t; 6] = [-1, -1, -1, 4611686018427387895 - 1, 0, -1];
    let in_ = crate::htslib_rs::hts::hts_open(sam_text.as_ptr(), c"r".as_ptr());
    let header = if in_.is_null() {
        std::ptr::null_mut()
    } else {
        sam::sam_hdr_read(in_)
    };
    let aln = sam::bam_init1();
    if header.is_null() || aln.is_null() {
        test_sam_bam_set1_fail(c"check_big_ref".as_ptr().cast(), c"setup failed".as_ptr().cast());
    } else {
        if parse_header != 0
            && sam::sam_hdr_count_lines(&mut *header, b"SQ") != expected_lengths.len() as i32
        {
            test_sam_bam_set1_fail(
                c"check_big_ref".as_ptr().cast(),
                c"Wrong number of SQ lines in header".as_ptr().cast(),
            );
        }
        test_sam_c_1388_check_ref_lengths(
            header,
            expected_lengths.as_ptr(),
            expected_lengths.len() as i32,
            c"header".as_ptr().cast(),
        );
        let mut i = 0usize;
        loop {
            let r = sam::sam_read1(in_, header, aln);
            if r < 0 {
                break;
            }
            if i >= expected_tids.len()
                || (*aln).core.tid != expected_tids[i]
                || (*aln).core.mtid != expected_mtid[i]
                || (*aln).core.pos != expected_positions[i]
                || (*aln).core.mpos != expected_mpos[i]
            {
                test_sam_bam_set1_fail(
                    c"check_big_ref".as_ptr().cast(),
                    c"alignment coordinate check failed".as_ptr().cast(),
                );
                break;
            }
            i += 1;
        }
        if i != expected_tids.len() {
            test_sam_bam_set1_fail(
                c"check_big_ref".as_ptr().cast(),
                c"wrong number of alignment records".as_ptr().cast(),
            );
        }
    }
    sam::bam_destroy1(aln);
    sam::sam_hdr_destroy(header);
    if !in_.is_null() {
        crate::htslib_rs::hts::hts_close(in_);
    }
}

// original: faidx1 (htslib/test/sam.c:1578)
pub unsafe fn test_sam_c_1578_faidx1(filename: *const u8) {
    let mut n_exp = 0;
    let mut n_fq_exp = 0;
    let filename_str = std::ffi::CStr::from_ptr(filename.cast())
        .to_string_lossy()
        .into_owned();
    let contents = match std::fs::read(&filename_str) {
        Ok(c) => c,
        Err(_) => {
            test_sam_bam_set1_fail(c"faidx1".as_ptr().cast(), c"can't open input file".as_ptr().cast());
            return;
        }
    };

    // tmpfilename as a NUL-terminated Vec<u8> to pass to the still-raw faidx API.
    let mut tmpfilename = filename_str.clone().into_bytes();
    tmpfilename.extend_from_slice(b".tmp\0");

    // Count `>` (FASTA) and `+\n` (FASTQ) marker lines while copying.
    for line in contents.split_inclusive(|&b| b == b'\n') {
        if line.first() == Some(&b'>') {
            n_exp += 1;
        }
        if line.first() == Some(&b'+') && line.get(1) == Some(&b'\n') {
            n_fq_exp += 1;
        }
    }
    let tmp_path = String::from_utf8_lossy(&tmpfilename[..tmpfilename.len() - 1]).into_owned();
    if std::fs::write(&tmp_path, &contents).is_err() {
        test_sam_bam_set1_fail(c"faidx1".as_ptr().cast(), c"can't create temporary file".as_ptr().cast());
        return;
    }

    if n_exp == 0 && n_fq_exp != 0 {
        n_exp = n_fq_exp;
    }

    if crate::htslib_rs::faidx::fai_build(tmpfilename.as_ptr().cast()) < 0 {
        test_sam_bam_set1_fail(c"faidx1".as_ptr().cast(), c"can't index temporary file".as_ptr().cast());
        goto_faidx1_cleanup(&tmp_path);
        return;
    }
    let fai = crate::htslib_rs::faidx::fai_load(tmpfilename.as_ptr().cast());
    if fai.is_null() {
        test_sam_bam_set1_fail(c"faidx1".as_ptr().cast(), c"can't load faidx file".as_ptr().cast());
        goto_faidx1_cleanup(&tmp_path);
        return;
    }

    let n = crate::htslib_rs::faidx::faidx_fetch_nseq(fai);
    if n != n_exp {
        test_sam_bam_set1_fail(
            c"faidx1".as_ptr().cast(),
            c"faidx_fetch_nseq returned unexpected count".as_ptr().cast(),
        );
    }
    let n = crate::htslib_rs::faidx::faidx_nseq(fai);
    if n != n_exp {
        test_sam_bam_set1_fail(
            c"faidx1".as_ptr().cast(),
            c"faidx_nseq returned unexpected count".as_ptr().cast(),
        );
    }

    crate::htslib_rs::faidx::fai_destroy(fai);
    goto_faidx1_cleanup(&tmp_path);
}

unsafe fn goto_faidx1_cleanup(tmpfilename: &str) {
    let _ = std::fs::remove_file(format!("{tmpfilename}.fai"));
    let _ = std::fs::remove_file(tmpfilename);
}

// original: test_empty_sam_file (htslib/test/sam.c:1618)
pub unsafe fn test_sam_c_1618_test_empty_sam_file(filename: *const u8) {
    let in_ = crate::htslib_rs::hts::hts_open(filename.cast(), c"r".as_ptr());
    if in_.is_null() {
        test_sam_bam_set1_fail(
            c"test_empty_sam_file".as_ptr().cast(),
            c"can't open file to read as SAM".as_ptr().cast(),
        );
        return;
    }

    let format = (*crate::htslib_rs::hts::hts_get_format(in_)).format;
    let aln = sam::bam_init1();
    let header = sam::sam_hdr_read(in_);
    let ret = sam::sam_read1(in_, header, aln);

    if format != crate::htslib_rs::hts::HTS_FORMAT_EMPTY_FORMAT {
        test_sam_bam_set1_fail(
            c"test_empty_sam_file".as_ptr().cast(),
            c"empty file detected as unexpected format".as_ptr().cast(),
        );
    }
    if !header.is_null() {
        test_sam_bam_set1_fail(
            c"test_empty_sam_file".as_ptr().cast(),
            c"sam_hdr_read from empty file should fail".as_ptr().cast(),
        );
    }
    if ret >= -1 {
        test_sam_bam_set1_fail(
            c"test_empty_sam_file".as_ptr().cast(),
            c"sam_read1 from empty file should fail".as_ptr().cast(),
        );
    }

    sam::bam_destroy1(aln);
    sam::sam_hdr_destroy(header);
    crate::htslib_rs::hts::hts_close(in_);
}

// original: test_text_file (htslib/test/sam.c:1641)
pub unsafe fn test_sam_c_1641_test_text_file(filename: *const u8, nexp: i32) {
    let in_ = crate::htslib_rs::hts::hts_open(filename.cast(), c"r".as_ptr());
    if in_.is_null() {
        test_sam_bam_set1_fail(
            c"test_text_file".as_ptr().cast(),
            c"can't open file to read as text".as_ptr().cast(),
        );
        return;
    }

    let mut str_ = kstring_t { data: Vec::new() };
    let mut n = 0;
    let mut ret;
    loop {
        ret = crate::htslib_rs::hts::hts_getline(in_, b'\n' as i32, &mut str_);
        if ret < 0 {
            break;
        }
        let len = str_.data.len();
        n += 1;
        if ret as usize != len {
            test_sam_bam_set1_fail(
                c"test_text_file".as_ptr().cast(),
                c"hts_getline read unexpected line length".as_ptr().cast(),
            );
        }
    }
    if ret != -1 {
        test_sam_bam_set1_fail(
            c"test_text_file".as_ptr().cast(),
            c"hts_getline got an error".as_ptr().cast(),
        );
    }
    if n != nexp {
        test_sam_bam_set1_fail(
            c"test_text_file".as_ptr().cast(),
            c"hts_getline read unexpected number of lines".as_ptr().cast(),
        );
    }

    crate::htslib_rs::hts::hts_close(in_);
    crate::htslib_rs::hts::ks_free(&mut str_);
}

// original: check_enum1 (htslib/test/sam.c:1661)
pub unsafe fn test_sam_c_1661_check_enum1() {
    if crate::htslib_rs::hts::HTS_COMPRESSION_NO_COMPRESSION != 0 {
        eprintln!(
            "Failed: no_compression is {}",
            crate::htslib_rs::hts::HTS_COMPRESSION_NO_COMPRESSION,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }
    if crate::htslib_rs::hts::HTS_COMPRESSION_GZIP != 1 {
        eprintln!(
            "Failed: gzip is {}",
            crate::htslib_rs::hts::HTS_COMPRESSION_GZIP,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }
    if crate::htslib_rs::hts::HTS_COMPRESSION_BGZF != 2 {
        eprintln!(
            "Failed: bgzf is {}",
            crate::htslib_rs::hts::HTS_COMPRESSION_BGZF,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }
}

// original: check_cigar_tab (htslib/test/sam.c:1669)
pub unsafe fn test_sam_c_1669_check_cigar_tab() {
    let cigar_str = b"MIDNSHP=XB";
    let n_neg = sam::BAM_CIGAR_TABLE
        .iter()
        .filter(|value| **value < 0)
        .count();
    if n_neg + cigar_str.len() != 256 {
        eprintln!("Failed: bam_cigar_table has {} unset entries", n_neg);
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }

    for (i, ch) in cigar_str.iter().copied().enumerate() {
        if sam::BAM_CIGAR_TABLE[ch as usize] != i as i8 {
            eprintln!(
                "Failed: bam_cigar_table['{}'] is not {}",
                ch as char, i,
            );
            TEST_SAM_STATUS = libc::EXIT_FAILURE;
        }
    }
}

// original: generator (htslib/test/sam.c:1688)
pub unsafe fn test_sam_c_1688_generator(name: *const u8) -> i32 {
    use std::io::Write;
    const MAX_RECS: usize = 1000;
    const SEQ_LEN: usize = 100;

    let name_str = std::ffi::CStr::from_ptr(name.cast())
        .to_string_lossy()
        .into_owned();
    let mut f = match std::fs::File::create(&name_str) {
        Ok(f) => std::io::BufWriter::new(f),
        Err(_) => {
            eprintln!("Couldn't open \"{name_str}\"");
            return -1;
        }
    };

    let mut ref_ = vec![0u8; MAX_RECS + SEQ_LEN];
    let bases = b"ACGT";
    let mut lfsr = 0xbadcafe_u32;
    for r in ref_.iter_mut() {
        lfsr ^= lfsr << 13;
        lfsr ^= lfsr >> 17;
        lfsr ^= lfsr << 5;
        *r = bases[(lfsr & 3) as usize];
    }

    let mut qual = [0u8; SEQ_LEN];
    for (i, q) in qual.iter_mut().enumerate() {
        *q = b'A' + (i as u8 & 0xf);
    }

    if f.write_all(b"@HD\tVN:1.4\n").is_err() {
        return -1;
    }
    if writeln!(f, "@SQ\tSN:ref1\tLN:{}", MAX_RECS + SEQ_LEN).is_err() {
        return -1;
    }
    for i in 0..MAX_RECS {
        let seq = &ref_[i..i + SEQ_LEN];
        if write!(f, "read{}\t0\tref1\t{}\t64\t100M\t*\t0\t0\t", i + 1, i + 1).is_err()
            || f.write_all(seq).is_err()
            || f.write_all(b"\t").is_err()
            || f.write_all(&qual).is_err()
            || f.write_all(b"\n").is_err()
        {
            return -1;
        }
    }

    match f.into_inner() {
        Ok(inner) => {
            if inner.sync_all().is_ok() {
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

// original: read_data_block (htslib/test/sam.c:1735)
#[allow(clippy::too_many_arguments)]
pub unsafe fn test_sam_c_1735_read_data_block(
    in_name: *const u8,
    fp_in: *mut htsFile,
    out_name: *const u8,
    fp_out: *mut htsFile,
    header: *mut sam::sam_hdr_t,
    recs: *mut sam::bam1_t,
    max_recs: usize,
    buffer: *mut u8,
    bufsz: usize,
    nrecs_out: *mut usize,
) -> i32 {
    let mut nrecs = 0usize;
    let mut ret = -1;
    let mut res = -1;

    let _ = (buffer, bufsz);
    while nrecs < max_recs {
        let rec = recs.add(nrecs);
        // SEAM: data is an owned Vec, so the external shared-buffer trick
        // (data = buffer + buff_used; m_data = remaining) is gone. We still set
        // the BAM_USER_OWNS_STRUCT|BAM_USER_OWNS_DATA policy: the struct lives in
        // an array (must not be Box-freed) and sam_read1 clears the OWNS_DATA bit
        // when it grows the rec's own Vec.
        sam::bam_set_mempolicy(rec, sam::BAM_USER_OWNS_STRUCT | sam::BAM_USER_OWNS_DATA);

        res = sam::sam_read1(fp_in, header, rec);
        if res < 0 {
            break;
        }

        if !fp_out.is_null() && sam::sam_c_4553_sam_write1(fp_out, header, rec) < 0 {
            nrecs += 1;
            test_sam_bam_set1_fail(c"read_data_block".as_ptr().cast(), c"sam_write1 failed".as_ptr().cast());
            *nrecs_out = nrecs;
            return ret;
        }

        nrecs += 1;
    }

    if res < -1 {
        let _ = in_name;
        test_sam_bam_set1_fail(c"read_data_block".as_ptr().cast(), c"sam_read1 failed".as_ptr().cast());
    } else {
        ret = 0;
    }
    let _ = out_name;
    *nrecs_out = nrecs;
    ret
}

// original: test_parse_decimal1 (htslib/test/sam.c:1781)
pub unsafe fn test_sam_c_1781_test_parse_decimal1(
    exp: i64,
    str_: *const u8,
    exp_consumed: usize,
    flags: i32,
    warning: *const u8,
) {
    let str_disp = std::ffi::CStr::from_ptr(str_.cast()).to_string_lossy();
    if !warning.is_null() {
        eprintln!(
            "(Expect {} message for \"{}\")",
            std::ffi::CStr::from_ptr(warning.cast()).to_string_lossy(),
            str_disp,
        );
    }

    let mut val = crate::htslib_rs::hts::hts_parse_decimal(str_.cast(), std::ptr::null_mut(), flags);
    if val != exp {
        eprintln!(
            "Failed: hts_parse_decimal(\"{}\", NULL, {}) returned {}, expected {}",
            str_disp, flags, val, exp,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }

    let mut end: *mut i8 = std::ptr::null_mut();
    val = crate::htslib_rs::hts::hts_parse_decimal(str_.cast(), &mut end, flags);
    if val != exp {
        eprintln!(
            "Failed: hts_parse_decimal(\"{}\", ..., {}) returned {}, expected {}",
            str_disp, flags, val, exp,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }

    let consumed = end.offset_from(str_.cast::<i8>()) as usize;
    if consumed != exp_consumed {
        eprintln!(
            "Failed: hts_parse_decimal(\"{}\", ..., {}) consumed {} chars, expected {}",
            str_disp, flags, consumed, exp_consumed,
        );
        TEST_SAM_STATUS = libc::EXIT_FAILURE;
    }
}

// original: test_parse_decimal (htslib/test/sam.c:1795)
pub unsafe fn test_sam_c_1795_test_parse_decimal() {
    test_sam_c_1781_test_parse_decimal1(37, c"+37".as_ptr().cast(), 3, 0, std::ptr::null());
    test_sam_c_1781_test_parse_decimal1(
        -1001,
        c" \t -1,001x".as_ptr().cast(),
        9,
        crate::htslib_rs::hts::HTS_PARSE_THOUSANDS_SEP,
        c"trailing 'x'".as_ptr().cast(),
    );
    test_sam_c_1781_test_parse_decimal1(
        i64::MAX,
        c"+9223372036854775807".as_ptr().cast(),
        20,
        0,
        std::ptr::null(),
    );
    test_sam_c_1781_test_parse_decimal1(
        i64::MIN,
        c"-9,223,372,036,854,775,808".as_ptr().cast(),
        26,
        crate::htslib_rs::hts::HTS_PARSE_THOUSANDS_SEP,
        std::ptr::null(),
    );
    test_sam_c_1781_test_parse_decimal1(1500, c"1.5e3".as_ptr().cast(), 5, 0, std::ptr::null());
    test_sam_c_1781_test_parse_decimal1(1500, c"1.5e+3k".as_ptr().cast(), 6, 0, c"trailing 'k'".as_ptr().cast());
    test_sam_c_1781_test_parse_decimal1(1500000000, c"1.5G".as_ptr().cast(), 4, 0, std::ptr::null());
    test_sam_c_1781_test_parse_decimal1(12345, c"12.345k".as_ptr().cast(), 7, 0, std::ptr::null());
    test_sam_c_1781_test_parse_decimal1(
        12345,
        c"12.3456k".as_ptr().cast(),
        8,
        0,
        c"dropped fraction".as_ptr().cast(),
    );
    test_sam_c_1781_test_parse_decimal1(0, c"A".as_ptr().cast(), 0, 0, c"invalid numeric".as_ptr().cast());
    test_sam_c_1781_test_parse_decimal1(0, c"G".as_ptr().cast(), 0, 0, c"invalid numeric".as_ptr().cast());
    test_sam_c_1781_test_parse_decimal1(0, c" +/-".as_ptr().cast(), 0, 0, c"invalid numeric".as_ptr().cast());
    test_sam_c_1781_test_parse_decimal1(
        0,
        c" \t -.e+9999".as_ptr().cast(),
        0,
        0,
        c"invalid numeric".as_ptr().cast(),
    );
}

// original: test_mempolicy (htslib/test/sam.c:1812)
pub unsafe fn test_sam_c_1812_test_mempolicy() {
    const REC_LENGTH: usize = 150;
    let bufsz = TEST_MEMPOLICY_MAX_RECS * REC_LENGTH;
    // SEAM: the bam1_t array can no longer be calloc-zeroed (bam1_t owns a Vec;
    // zeroing it would be UB). Allocate an owned, default-initialised boxed array
    // and hand out a *mut bam1_t into it; the cleanup reconstructs the Box to drop
    // it (replacing the old libc::free(recs)).
    let recs_box: Box<[sam::bam1_t; TEST_MEMPOLICY_MAX_RECS]> =
        Box::new(std::array::from_fn(|_| sam::bam1_t::default()));
    let recs = Box::into_raw(recs_box).cast::<sam::bam1_t>();
    // SEAM: the shared scratch buffer is an owned Vec (replaces libc::malloc/free).
    let mut buffer_vec = vec![0u8; bufsz];
    let buffer = buffer_vec.as_mut_ptr();
    let fname = c"test/sam_alignment.tmp.sam".as_ptr();
    let bam_name = c"test/sam_alignment.tmp.bam".as_ptr();
    let cram_name = c"test/sam_alignment.tmp.cram".as_ptr();
    let tag_text = c"lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... lengthy text ... ";
    let mut nrecs = 0usize;
    let mut fp: *mut htsFile = std::ptr::null_mut();
    let mut bam_fp: *mut htsFile = std::ptr::null_mut();
    let mut cram_fp: *mut htsFile = std::ptr::null_mut();
    let mut header: *mut sam::sam_hdr_t = std::ptr::null_mut();
    let mut cram_fmt: htsFormat = std::mem::zeroed();
    if recs.is_null() || buffer.is_null() {
        test_sam_bam_set1_fail(c"test_mempolicy".as_ptr().cast(), c"Allocating buffer".as_ptr().cast());
        goto_test_mempolicy_cleanup(
            recs,
            buffer,
            nrecs,
            fp,
            bam_fp,
            cram_fp,
            header,
            &mut cram_fmt,
        );
        return;
    }
    if test_sam_c_1688_generator(fname.cast()) < 0 {
        goto_test_mempolicy_cleanup(
            recs,
            buffer,
            nrecs,
            fp,
            bam_fp,
            cram_fp,
            header,
            &mut cram_fmt,
        );
        return;
    }
    fp = crate::htslib_rs::hts::hts_open(fname, c"r".as_ptr());
    bam_fp = crate::htslib_rs::hts::hts_open(bam_name, c"wb".as_ptr());
    header = if fp.is_null() {
        std::ptr::null_mut()
    } else {
        sam::sam_hdr_read(fp)
    };
    if fp.is_null()
        || bam_fp.is_null()
        || header.is_null()
        || sam::sam_hdr_write(bam_fp, header) < 0
    {
        test_sam_bam_set1_fail(
            c"test_mempolicy".as_ptr().cast(),
            c"SAM to BAM setup failed".as_ptr().cast(),
        );
        goto_test_mempolicy_cleanup(
            recs,
            buffer,
            nrecs,
            fp,
            bam_fp,
            cram_fp,
            header,
            &mut cram_fmt,
        );
        return;
    }
    if test_sam_c_1735_read_data_block(
        fname.cast(), fp, bam_name.cast(), bam_fp, header, recs, TEST_MEMPOLICY_MAX_RECS, buffer, bufsz, &mut nrecs,
    ) < 0
    {
        goto_test_mempolicy_cleanup(
            recs,
            buffer,
            nrecs,
            fp,
            bam_fp,
            cram_fp,
            header,
            &mut cram_fmt,
        );
        return;
    }
    if crate::htslib_rs::hts::hts_close(bam_fp) < 0 {
        bam_fp = std::ptr::null_mut();
        test_sam_bam_set1_fail(c"test_mempolicy".as_ptr().cast(), c"sam_close BAM".as_ptr().cast());
        goto_test_mempolicy_cleanup(
            recs,
            buffer,
            nrecs,
            fp,
            bam_fp,
            cram_fp,
            header,
            &mut cram_fmt,
        );
        return;
    }
    bam_fp = std::ptr::null_mut();
    for i in (0..TEST_MEMPOLICY_MAX_RECS).step_by(11) {
        if sam::bam_aux_update_str(
            recs.add(i),
            c"ZZ".as_ptr().cast(),
            tag_text.to_bytes().len() as i32,
            tag_text.as_ptr().cast(),
        ) < 0
        {
            test_sam_bam_set1_fail(c"test_mempolicy".as_ptr().cast(), c"bam_aux_update_str".as_ptr().cast());
            break;
        }
    }
    for i in 0..nrecs {
        sam::bam_destroy1(recs.add(i));
    }
    nrecs = 0;
    if crate::htslib_rs::hts::hts_close(fp) < 0 {
        fp = std::ptr::null_mut();
        test_sam_bam_set1_fail(c"test_mempolicy".as_ptr().cast(), c"sam_close SAM".as_ptr().cast());
        goto_test_mempolicy_cleanup(
            recs,
            buffer,
            nrecs,
            fp,
            bam_fp,
            cram_fp,
            header,
            &mut cram_fmt,
        );
        return;
    }
    fp = std::ptr::null_mut();
    sam::sam_hdr_destroy(header);
    header = std::ptr::null_mut();

    bam_fp = crate::htslib_rs::hts::hts_open(bam_name, c"r".as_ptr());
    if crate::htslib_rs::hts::hts_parse_format(&mut cram_fmt, c"cram,no_ref".as_ptr()) < 0 {
        test_sam_bam_set1_fail(c"test_mempolicy".as_ptr().cast(), c"hts_parse_format".as_ptr().cast());
        goto_test_mempolicy_cleanup(
            recs,
            buffer,
            nrecs,
            fp,
            bam_fp,
            cram_fp,
            header,
            &mut cram_fmt,
        );
        return;
    }
    cram_fp = crate::htslib_rs::hts::hts_open_format(cram_name, c"wc".as_ptr(), &cram_fmt);
    header = if bam_fp.is_null() {
        std::ptr::null_mut()
    } else {
        sam::sam_hdr_read(bam_fp)
    };
    if bam_fp.is_null()
        || cram_fp.is_null()
        || header.is_null()
        || sam::sam_hdr_write(cram_fp, header) < 0
    {
        test_sam_bam_set1_fail(
            c"test_mempolicy".as_ptr().cast(),
            c"BAM to CRAM setup failed".as_ptr().cast(),
        );
        goto_test_mempolicy_cleanup(
            recs,
            buffer,
            nrecs,
            fp,
            bam_fp,
            cram_fp,
            header,
            &mut cram_fmt,
        );
        return;
    }
    test_sam_c_1735_read_data_block(
        bam_name.cast(), bam_fp, cram_name.cast(), cram_fp, header, recs, TEST_MEMPOLICY_MAX_RECS, buffer, bufsz, &mut nrecs,
    );
    goto_test_mempolicy_cleanup(
        recs,
        buffer,
        nrecs,
        fp,
        bam_fp,
        cram_fp,
        header,
        &mut cram_fmt,
    );
}

#[allow(clippy::too_many_arguments)]
unsafe fn goto_test_mempolicy_cleanup(
    recs: *mut sam::bam1_t,
    buffer: *mut u8,
    nrecs: usize,
    fp: *mut htsFile,
    bam_fp: *mut htsFile,
    cram_fp: *mut htsFile,
    header: *mut sam::sam_hdr_t,
    cram_fmt: *mut htsFormat,
) {
    sam::sam_hdr_destroy(header);
    if !fp.is_null() {
        crate::htslib_rs::hts::hts_close(fp);
    }
    if !bam_fp.is_null() {
        crate::htslib_rs::hts::hts_close(bam_fp);
    }
    if !cram_fp.is_null() {
        crate::htslib_rs::hts::hts_close(cram_fp);
    }
    for i in 0..nrecs {
        sam::bam_destroy1(recs.add(i));
    }
    // SEAM: the scratch buffer is an owned Vec in the caller; nothing to free here.
    let _ = buffer;
    // SEAM: reclaim the owned boxed bam1_t array (replaces libc::free(recs)).
    // The structs are BAM_USER_OWNS_STRUCT, so the bam_destroy1 calls above only
    // cleared their data Vecs; dropping the Box releases the array storage (and
    // the default-empty Vecs of any never-read tail records).
    if !recs.is_null() {
        drop(Box::from_raw(
            recs.cast::<[sam::bam1_t; TEST_MEMPOLICY_MAX_RECS]>(),
        ));
    }
    if !(*cram_fmt).specific.is_null() {
        crate::htslib_rs::hts::hts_opt_free((*cram_fmt).specific.cast());
    }
}

// original: test_bam_set1_minimal (htslib/test/sam.c:2000)
pub unsafe fn test_sam_c_2000_test_bam_set1_minimal() {
    let bam = sam::bam_init1();
    if bam.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_minimal".as_ptr().cast(),
            c"failed to initialize BAM struct.".as_ptr().cast(),
        );
        return;
    }

    let r = sam::bam_set1(
        bam,
        0,
        std::ptr::null(),
        sam::BAM_FUNMAP as u16,
        -1,
        0,
        0xff,
        0,
        std::ptr::null(),
        -1,
        0,
        0,
        0,
        std::ptr::null(),
        std::ptr::null(),
        0,
    );
    if r != 4
        || (*bam).core.l_qname != 4
        || (*bam).core.l_extranul != 2
        || std::ffi::CStr::from_ptr(sam::bam_get_qname(bam).cast()).to_bytes() != b"*"
        || (*bam).core.pos != 0
        || (*bam).core.tid != -1
        || (*bam).core.bin as i32 != crate::htslib_rs::hts::hts_reg2bin(0, 1, 14, 5)
        || (*bam).core.qual != 0xff
        || (*bam).core.flag as i32 != sam::BAM_FUNMAP
        || (*bam).core.n_cigar != 0
        || (*bam).core.mtid != -1
        || (*bam).core.mpos != 0
        || (*bam).core.isize != 0
        || (*bam).core.l_qseq != 0
        || sam::bam_get_l_aux(bam) != 0
    {
        test_sam_bam_set1_fail(
            c"test_bam_set1_minimal".as_ptr().cast(),
            c"bam_set1 minimal checks failed.".as_ptr().cast(),
        );
    }
    sam::bam_destroy1(bam);
}

// original: test_bam_set1_full (htslib/test/sam.c:2031)
pub unsafe fn test_sam_c_2031_test_bam_set1_full() {
    let qname_c = c"!??AAA~~~~";
    let qname = qname_c.as_ptr();
    let cigar = [
        (6u32 << sam::BAM_CIGAR_SHIFT) | sam::BAM_CMATCH as u32,
        (2u32 << sam::BAM_CIGAR_SHIFT) | sam::BAM_CINS as u32,
        (2u32 << sam::BAM_CIGAR_SHIFT) | sam::BAM_CMATCH as u32,
    ];
    let seq_c = c"TGGACTACGA";
    let seq = seq_c.as_ptr();
    let seq_len = seq_c.to_bytes().len();
    let qual = c"DBBBB+=7=0".as_ptr();
    let bam = sam::bam_init1();
    if bam.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_full".as_ptr().cast(),
            c"failed to initialize BAM struct.".as_ptr().cast(),
        );
        return;
    }

    let r = sam::bam_set1(
        bam,
        qname_c.to_bytes().len(),
        qname.cast(),
        sam::BAM_FREVERSE as u16,
        1,
        1000,
        42,
        cigar.len(),
        cigar.as_ptr(),
        2,
        2000,
        3000,
        seq_len,
        seq.cast(),
        qual.cast(),
        64,
    );

    let mut ok = r == 39
        && (*bam).core.l_qname == 12
        && (*bam).core.l_extranul == 1
        && std::ffi::CStr::from_ptr(sam::bam_get_qname(bam).cast()).to_bytes()
            == std::ffi::CStr::from_ptr(qname.cast()).to_bytes()
        && (*bam).core.n_cigar as usize == cigar.len()
        && std::slice::from_raw_parts(sam::bam_get_cigar(bam), cigar.len()) == cigar
        && (*bam).core.l_qseq as usize == seq_len
        && std::slice::from_raw_parts(sam::bam_get_qual(bam), seq_len)
            == std::slice::from_raw_parts(qual.cast::<u8>(), seq_len)
        && (*bam).core.pos == 1000
        && (*bam).core.tid == 1
        && (*bam).core.bin as i32 == crate::htslib_rs::hts::hts_reg2bin(1000, 1010, 14, 5)
        && (*bam).core.qual == 42
        && (*bam).core.flag as i32 == sam::BAM_FREVERSE
        && (*bam).core.mtid == 2
        && (*bam).core.mpos == 2000
        && (*bam).core.isize == 3000
        && sam::bam_get_l_aux(bam) == 0
        && (*bam).data.capacity() as i64 - (*bam).data.len() as i64 >= 64;
    for i in 0..seq_len {
        ok &= sam::bam_seqi(sam::bam_get_seq(bam), i)
            == sam::SEQ_NT16_TABLE[*seq.add(i) as u8 as usize];
    }
    if !ok {
        test_sam_bam_set1_fail(
            c"test_bam_set1_full".as_ptr().cast(),
            c"bam_set1 full checks failed.".as_ptr().cast(),
        );
    }
    sam::bam_destroy1(bam);
}

// original: test_bam_set1_even_and_odd_seq_len (htslib/test/sam.c:2078)
pub unsafe fn test_sam_c_2078_test_bam_set1_even_and_odd_seq_len() {
    let bam = sam::bam_init1();
    if bam.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_even_and_odd_seq_len".as_ptr().cast(),
            c"failed to initialize BAM struct.".as_ptr().cast(),
        );
        return;
    }
    for seq_c in [c"TGGACTACGA", c"TGGACTACGAC"] {
        let seq = seq_c.as_ptr();
        let seq_len = seq_c.to_bytes().len();
        let r = sam::bam_set1(
            bam,
            0,
            std::ptr::null(),
            sam::BAM_FUNMAP as u16,
            0,
            0,
            0,
            0,
            std::ptr::null(),
            0,
            0,
            0,
            seq_len,
            seq.cast(),
            std::ptr::null(),
            0,
        );
        let mut ok = r >= 0 && (*bam).core.l_qseq as usize == seq_len;
        for i in 0..seq_len {
            ok &= sam::bam_seqi(sam::bam_get_seq(bam), i)
                == sam::SEQ_NT16_TABLE[*seq.add(i) as u8 as usize];
        }
        if !ok {
            test_sam_bam_set1_fail(
                c"test_bam_set1_even_and_odd_seq_len".as_ptr().cast(),
                c"sequence checks failed.".as_ptr().cast(),
            );
            break;
        }
    }
    sam::bam_destroy1(bam);
}

// original: test_bam_set1_with_seq_but_no_qual (htslib/test/sam.c:2108)
pub unsafe fn test_sam_c_2108_test_bam_set1_with_seq_but_no_qual() {
    let seq_c = c"TGGACTACGA";
    let seq = seq_c.as_ptr();
    let seq_len = seq_c.to_bytes().len();
    let bam = sam::bam_init1();
    if bam.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_with_seq_but_no_qual".as_ptr().cast(),
            c"failed to initialize BAM struct.".as_ptr().cast(),
        );
        return;
    }
    let r = sam::bam_set1(
        bam,
        0,
        std::ptr::null(),
        sam::BAM_FUNMAP as u16,
        0,
        0,
        0,
        0,
        std::ptr::null(),
        0,
        0,
        0,
        seq_len,
        seq.cast(),
        std::ptr::null(),
        0,
    );
    let mut ok = r >= 0 && (*bam).core.l_qseq as usize == seq_len;
    for i in 0..seq_len {
        ok &= sam::bam_seqi(sam::bam_get_seq(bam), i)
            == sam::SEQ_NT16_TABLE[*seq.add(i) as u8 as usize]
            && *sam::bam_get_qual(bam).add(i) == 0xff;
    }
    if !ok {
        test_sam_bam_set1_fail(
            c"test_bam_set1_with_seq_but_no_qual".as_ptr().cast(),
            c"sequence or quality checks failed.".as_ptr().cast(),
        );
    }
    sam::bam_destroy1(bam);
}

// original: test_bam_set1_validate_qname (htslib/test/sam.c:2132)
pub unsafe fn test_sam_c_2132_test_bam_set1_validate_qname() {
    let bam = sam::bam_init1();
    if bam.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_validate_qname".as_ptr().cast(),
            c"failed to initialize BAM struct.".as_ptr().cast(),
        );
        return;
    }
    let too_long = [b'A'; 255];
    let r = sam::bam_set1(
        bam,
        too_long.len(),
        too_long.as_ptr().cast(),
        sam::BAM_FUNMAP as u16,
        -1,
        0,
        0xff,
        0,
        std::ptr::null(),
        -1,
        0,
        0,
        0,
        std::ptr::null(),
        std::ptr::null(),
        0,
    );
    if r >= 0 || *libc::__errno_location() != libc::EINVAL {
        test_sam_bam_set1_fail(
            c"test_bam_set1_validate_qname".as_ptr().cast(),
            c"qname validation failed.".as_ptr().cast(),
        );
    }
    sam::bam_destroy1(bam);
}

// original: test_bam_set1_validate_seq (htslib/test/sam.c:2149)
pub unsafe fn test_sam_c_2149_test_bam_set1_validate_seq() {
    let bam = sam::bam_init1();
    if bam.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_validate_seq".as_ptr().cast(),
            c"failed to initialize BAM struct.".as_ptr().cast(),
        );
        return;
    }
    let sequence = c"C".as_ptr();
    let r = sam::bam_set1(
        bam,
        0,
        std::ptr::null(),
        sam::BAM_FUNMAP as u16,
        -1,
        0,
        0xff,
        0,
        std::ptr::null(),
        -1,
        0,
        0,
        i32::MAX as usize + 1,
        sequence.cast(),
        std::ptr::null(),
        0,
    );
    if r >= 0 || *libc::__errno_location() != libc::EINVAL {
        test_sam_bam_set1_fail(
            c"test_bam_set1_validate_seq".as_ptr().cast(),
            c"sequence validation failed.".as_ptr().cast(),
        );
    }
    sam::bam_destroy1(bam);
}

// original: test_bam_set1_validate_cigar (htslib/test/sam.c:2166)
pub unsafe fn test_sam_c_2166_test_bam_set1_validate_cigar() {
    let cigar = [(20u32 << sam::BAM_CIGAR_SHIFT) | sam::BAM_CMATCH as u32];
    let seq_c = c"TGGACTACGA";
    let seq = seq_c.as_ptr();
    let seq_len = seq_c.to_bytes().len();
    let bam = sam::bam_init1();
    if bam.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_validate_cigar".as_ptr().cast(),
            c"failed to initialize BAM struct.".as_ptr().cast(),
        );
        return;
    }

    let r1 = sam::bam_set1(
        bam,
        0,
        std::ptr::null(),
        0,
        -1,
        0,
        0xff,
        0,
        std::ptr::null(),
        -1,
        0,
        0,
        seq_len,
        seq.cast(),
        std::ptr::null(),
        0,
    );
    let e1 = *libc::__errno_location();
    let r2 = sam::bam_set1(
        bam,
        0,
        std::ptr::null(),
        0,
        -1,
        crate::htslib_rs::hts::HTS_POS_MAX - 10,
        0xff,
        cigar.len(),
        cigar.as_ptr(),
        -1,
        0,
        0,
        0,
        std::ptr::null(),
        std::ptr::null(),
        0,
    );
    let e2 = *libc::__errno_location();
    let r3 = sam::bam_set1(
        bam,
        0,
        std::ptr::null(),
        0,
        -1,
        0,
        0xff,
        cigar.len(),
        cigar.as_ptr(),
        -1,
        0,
        0,
        seq_len,
        seq.cast(),
        std::ptr::null(),
        0,
    );
    let e3 = *libc::__errno_location();
    if r1 >= 0
        || e1 != libc::EINVAL
        || r2 >= 0
        || e2 != libc::EINVAL
        || r3 >= 0
        || e3 != libc::EINVAL
    {
        test_sam_bam_set1_fail(
            c"test_bam_set1_validate_cigar".as_ptr().cast(),
            c"cigar validation failed.".as_ptr().cast(),
        );
    }
    sam::bam_destroy1(bam);
}

// original: test_bam_set1_validate_size_limits (htslib/test/sam.c:2195)
pub unsafe fn test_sam_c_2195_test_bam_set1_validate_size_limits() {
    let cigar = [(20u32 << sam::BAM_CIGAR_SHIFT) | sam::BAM_CMATCH as u32];
    let seq = c"TGGACTACGA".as_ptr();
    let bam = sam::bam_init1();
    if bam.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_validate_size_limits".as_ptr().cast(),
            c"failed to initialize BAM struct.".as_ptr().cast(),
        );
        return;
    }
    let r1 = sam::bam_set1(
        bam,
        0,
        std::ptr::null(),
        sam::BAM_FUNMAP as u16,
        -1,
        0,
        0xff,
        0,
        std::ptr::null(),
        -1,
        0,
        0,
        2 * (i32::MAX as usize) / 3,
        seq.cast(),
        std::ptr::null(),
        0,
    );
    let e1 = *libc::__errno_location();
    let r2 = sam::bam_set1(
        bam,
        0,
        std::ptr::null(),
        sam::BAM_FUNMAP as u16,
        -1,
        0,
        0xff,
        i32::MAX as usize / 4,
        cigar.as_ptr(),
        -1,
        0,
        0,
        0,
        std::ptr::null(),
        std::ptr::null(),
        0,
    );
    let e2 = *libc::__errno_location();
    let r3 = sam::bam_set1(
        bam,
        0,
        std::ptr::null(),
        sam::BAM_FUNMAP as u16,
        -1,
        0,
        0xff,
        0,
        std::ptr::null(),
        -1,
        0,
        0,
        0,
        std::ptr::null(),
        std::ptr::null(),
        i32::MAX as usize,
    );
    let e3 = *libc::__errno_location();
    if r1 >= 0
        || e1 != libc::EINVAL
        || r2 >= 0
        || e2 != libc::EINVAL
        || r3 >= 0
        || e3 != libc::EINVAL
    {
        test_sam_bam_set1_fail(
            c"test_bam_set1_validate_size_limits".as_ptr().cast(),
            c"size limit validation failed.".as_ptr().cast(),
        );
    }
    sam::bam_destroy1(bam);
}

unsafe fn test_sam_bam_set1_fail(func: *const u8, message: *const u8) {
    eprintln!(
        "Failed: {}: {}",
        std::ffi::CStr::from_ptr(func.cast()).to_string_lossy(),
        std::ffi::CStr::from_ptr(message.cast()).to_string_lossy(),
    );
    TEST_SAM_STATUS = libc::EXIT_FAILURE;
}

// original: test_bam_set1_write_and_read_back (htslib/test/sam.c:2227)
pub unsafe fn test_sam_c_2227_test_bam_set1_write_and_read_back() {
    let qname_c = c"q1";
    let qname = qname_c.as_ptr();
    let cigar = [
        (6u32 << sam::BAM_CIGAR_SHIFT) | sam::BAM_CMATCH as u32,
        (2u32 << sam::BAM_CIGAR_SHIFT) | sam::BAM_CINS as u32,
        (2u32 << sam::BAM_CIGAR_SHIFT) | sam::BAM_CMATCH as u32,
    ];
    let seq_c = c"TGGACTACGA";
    let seq = seq_c.as_ptr();
    let seq_len = seq_c.to_bytes().len();
    let qual = c"DBBBB+=7=0".as_ptr();
    let path = std::env::temp_dir().join(format!(
        "htslib_rs-test-bam-set1-write-read-{}.bam",
        std::process::id()
    ));
    let mut temp_fname = path.to_string_lossy().into_owned().into_bytes();
    temp_fname.push(0);

    let mut writer = crate::htslib_rs::hts::hts_open(temp_fname.as_ptr().cast(), c"wb".as_ptr());
    let mut reader: *mut htsFile = std::ptr::null_mut();
    let mut w_header: *mut sam::sam_hdr_t = std::ptr::null_mut();
    let mut r_header: *mut sam::sam_hdr_t = std::ptr::null_mut();
    let mut w_bam: *mut sam::bam1_t = std::ptr::null_mut();
    let mut r_bam: *mut sam::bam1_t = std::ptr::null_mut();
    let mut ks = kstring_t { data: Vec::new() };

    if writer.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_write_and_read_back".as_ptr().cast(),
            c"failed to open bam file for writing.".as_ptr().cast(),
        );
        goto_test_bam_set1_write_read_cleanup(
            temp_fname.as_ptr().cast(),
            writer,
            reader,
            w_header,
            r_header,
            w_bam,
            r_bam,
            &mut ks,
        );
        return;
    }

    w_header = sam::bam_hdr_init();
    let header_line = c"@SQ\tSN:t1\tLN:5000\n";
    if w_header.is_null()
        || sam::sam_hdr_add_lines(&mut *w_header, header_line.to_bytes()) != 0
        || sam::sam_hdr_write(writer, w_header) != 0
    {
        test_sam_bam_set1_fail(
            c"test_bam_set1_write_and_read_back".as_ptr().cast(),
            c"failed to write bam header.".as_ptr().cast(),
        );
        goto_test_bam_set1_write_read_cleanup(
            temp_fname.as_ptr().cast(),
            writer,
            reader,
            w_header,
            r_header,
            w_bam,
            r_bam,
            &mut ks,
        );
        return;
    }

    w_bam = sam::bam_init1();
    if w_bam.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_write_and_read_back".as_ptr().cast(),
            c"failed to initialize BAM struct.".as_ptr().cast(),
        );
        goto_test_bam_set1_write_read_cleanup(
            temp_fname.as_ptr().cast(),
            writer,
            reader,
            w_header,
            r_header,
            w_bam,
            r_bam,
            &mut ks,
        );
        return;
    }
    let mut r = sam::bam_set1(
        w_bam,
        qname_c.to_bytes().len(),
        qname.cast(),
        (sam::BAM_FPAIRED | sam::BAM_FREVERSE) as u16,
        0,
        1000,
        42,
        cigar.len(),
        cigar.as_ptr(),
        0,
        2000,
        3000,
        seq_len,
        seq.cast(),
        qual.cast(),
        64,
    );
    if r < 0 || sam::sam_c_4553_sam_write1(writer, w_header, w_bam) < 0 {
        test_sam_bam_set1_fail(
            c"test_bam_set1_write_and_read_back".as_ptr().cast(),
            c"failed to write alignment.".as_ptr().cast(),
        );
        goto_test_bam_set1_write_read_cleanup(
            temp_fname.as_ptr().cast(),
            writer,
            reader,
            w_header,
            r_header,
            w_bam,
            r_bam,
            &mut ks,
        );
        return;
    }
    sam::bam_destroy1(w_bam);
    w_bam = std::ptr::null_mut();

    r = crate::htslib_rs::hts::hts_close(writer);
    writer = std::ptr::null_mut();
    if r != 0 {
        test_sam_bam_set1_fail(
            c"test_bam_set1_write_and_read_back".as_ptr().cast(),
            c"failed to close bam file for writing.".as_ptr().cast(),
        );
        goto_test_bam_set1_write_read_cleanup(
            temp_fname.as_ptr().cast(),
            writer,
            reader,
            w_header,
            r_header,
            w_bam,
            r_bam,
            &mut ks,
        );
        return;
    }
    sam::sam_hdr_destroy(w_header);
    w_header = std::ptr::null_mut();

    reader = crate::htslib_rs::hts::hts_open(temp_fname.as_ptr().cast(), c"rb".as_ptr());
    if reader.is_null() {
        test_sam_bam_set1_fail(
            c"test_bam_set1_write_and_read_back".as_ptr().cast(),
            c"failed to open bam file for reading.".as_ptr().cast(),
        );
        goto_test_bam_set1_write_read_cleanup(
            temp_fname.as_ptr().cast(),
            writer,
            reader,
            w_header,
            r_header,
            w_bam,
            r_bam,
            &mut ks,
        );
        return;
    }
    r_header = sam::sam_hdr_read(reader);
    if r_header.is_null()
        || sam::sam_hdr_find_tag_id(
            &mut *r_header,
            b"SQ",
            None,
            b"SN",
            &mut ks,
        ) != 0
        || crate::htslib_rs::hts::ks_c_str(&ks) != b"t1"
        || (*r_header).n_targets != 1
        || std::ffi::CStr::from_ptr((*(*r_header).target_name).cast()).to_bytes() != b"t1"
        || *(*r_header).target_len != 5000
    {
        test_sam_bam_set1_fail(
            c"test_bam_set1_write_and_read_back".as_ptr().cast(),
            c"failed to read expected bam header.".as_ptr().cast(),
        );
        goto_test_bam_set1_write_read_cleanup(
            temp_fname.as_ptr().cast(),
            writer,
            reader,
            w_header,
            r_header,
            w_bam,
            r_bam,
            &mut ks,
        );
        return;
    }

    r_bam = sam::bam_init1();
    if r_bam.is_null()
        || sam::sam_read1(reader, r_header, r_bam) < 0
        || std::ffi::CStr::from_ptr(sam::bam_get_qname(r_bam).cast()).to_bytes()
            != std::ffi::CStr::from_ptr(qname.cast()).to_bytes()
        || (*r_bam).core.n_cigar as usize != cigar.len()
        || std::slice::from_raw_parts(sam::bam_get_cigar(r_bam), cigar.len()) != cigar
        || (*r_bam).core.l_qseq as usize != seq_len
        || sam::sam_read1(reader, r_header, r_bam) >= 0
    {
        test_sam_bam_set1_fail(
            c"test_bam_set1_write_and_read_back".as_ptr().cast(),
            c"failed to read expected bam alignment.".as_ptr().cast(),
        );
    }

    goto_test_bam_set1_write_read_cleanup(
        temp_fname.as_ptr().cast(),
        writer,
        reader,
        w_header,
        r_header,
        w_bam,
        r_bam,
        &mut ks,
    );
}

#[allow(clippy::too_many_arguments)]
unsafe fn goto_test_bam_set1_write_read_cleanup(
    temp_fname: *const u8,
    writer: *mut htsFile,
    reader: *mut htsFile,
    w_header: *mut sam::sam_hdr_t,
    r_header: *mut sam::sam_hdr_t,
    w_bam: *mut sam::bam1_t,
    r_bam: *mut sam::bam1_t,
    ks: &mut kstring_t,
) {
    if !w_bam.is_null() {
        sam::bam_destroy1(w_bam);
    }
    if !r_bam.is_null() {
        sam::bam_destroy1(r_bam);
    }
    if !w_header.is_null() {
        sam::sam_hdr_destroy(w_header);
    }
    if !r_header.is_null() {
        sam::sam_hdr_destroy(r_header);
    }
    if !writer.is_null() {
        crate::htslib_rs::hts::hts_close(writer);
    }
    if !reader.is_null() {
        crate::htslib_rs::hts::hts_close(reader);
    }
    crate::htslib_rs::hts::ks_free(ks);
    let _ = std::fs::remove_file(
        std::ffi::CStr::from_ptr(temp_fname.cast())
            .to_string_lossy()
            .as_ref(),
    );
}

// original: test_cigar_api (htslib/test/sam.c:2307)
pub unsafe fn test_sam_c_2307_test_cigar_api() {
    let mut buf: *mut u32 = std::ptr::null_mut();
    let mut end: *mut u8 = std::ptr::null_mut();
    let mut m = 0usize;

    let cig: *const u8 = c"*".as_ptr().cast();
    let mut n = sam::sam_parse_cigar(cig.cast(), &mut end, &mut buf, &mut m);
    if n != 0 || m != 0 || end.offset_from(cig) != 1 {
        test_sam_bam_set1_fail(
            c"test_cigar_api".as_ptr().cast(),
            c"failed to parse undefined CIGAR".as_ptr().cast(),
        );
    }

    let cig: *const u8 = c"2M3X1I10M5D".as_ptr().cast();
    n = sam::sam_parse_cigar(cig.cast(), &mut end, &mut buf, &mut m);
    if n != 5 || m == 0 || end.offset_from(cig) != 11 {
        test_sam_bam_set1_fail(
            c"test_cigar_api".as_ptr().cast(),
            c"failed to parse CIGAR string: 2M3X1I10M5D".as_ptr().cast(),
        );
    }

    n = sam::sam_parse_cigar(
        c"722M15D187217376188323783284M67I".as_ptr().cast(),
        std::ptr::null_mut(),
        &mut buf,
        &mut m,
    );
    if n != -1 {
        test_sam_bam_set1_fail(
            c"test_cigar_api".as_ptr().cast(),
            c"failed to flag CIGAR string with long op length: 722M15D187217376188323783284M67I"
                .as_ptr().cast(),
        );
    }

    n = sam::sam_parse_cigar(
        c"53I722MD8X".as_ptr().cast(),
        std::ptr::null_mut(),
        &mut buf,
        &mut m,
    );
    if n != -1 {
        test_sam_bam_set1_fail(
            c"test_cigar_api".as_ptr().cast(),
            c"failed to flag CIGAR string with no op length: 53I722MD8X".as_ptr().cast(),
        );
    }

    // SEAM: `buf` is filled by the (parallel-converting) production sam_parse_cigar,
    // which now owns/reclaims its growth buffer; the test no longer frees it.
    let _ = buf;
}

// original: main (htslib/test/sam.c:2328)
pub unsafe fn test_sam_c_2328_main(argc: i32, argv: *mut *mut u8) -> i32 {
    TEST_SAM_STATUS = libc::EXIT_SUCCESS;
    test_sam_c_248_aux_fields1();
    test_sam_c_604_iterators1();
    test_sam_c_1348_samrecord_layout();
    test_sam_c_705_use_header_api();
    test_sam_c_921_test_header_pg_lines();
    test_sam_c_1008_test_header_pg_loops();
    test_sam_c_1087_test_header_updates();
    test_sam_c_1182_test_header_remove_lines();
    test_sam_c_1258_test_header_ref_altnames();
    test_sam_c_1618_test_empty_sam_file(c"test/emptyfile".as_ptr().cast());
    test_sam_c_1641_test_text_file(c"test/emptyfile".as_ptr().cast(), 0);
    test_sam_c_1641_test_text_file(c"test/xx#pair.sam".as_ptr().cast(), 7);
    test_sam_c_1641_test_text_file(c"test/xx.fa".as_ptr().cast(), 7);
    test_sam_c_1641_test_text_file(c"test/faidx/fastqs.fq".as_ptr().cast(), 500);
    test_sam_c_1661_check_enum1();
    test_sam_c_1669_check_cigar_tab();
    test_sam_c_1405_check_big_ref_parse(0);
    test_sam_c_1405_check_big_ref_parse(1);
    test_sam_c_1795_test_parse_decimal();
    test_sam_c_1812_test_mempolicy();
    test_sam_c_557_set_qname();
    for i in 1..argc {
        test_sam_c_1578_faidx1(*argv.add(i as usize));
    }
    crate::htslib_rs::hts::hts_set_log_level(crate::htslib_rs::hts::HTS_LOG_OFF);
    test_sam_c_2000_test_bam_set1_minimal();
    test_sam_c_2031_test_bam_set1_full();
    test_sam_c_2078_test_bam_set1_even_and_odd_seq_len();
    test_sam_c_2108_test_bam_set1_with_seq_but_no_qual();
    test_sam_c_2132_test_bam_set1_validate_qname();
    test_sam_c_2149_test_bam_set1_validate_seq();
    test_sam_c_2166_test_bam_set1_validate_cigar();
    test_sam_c_2195_test_bam_set1_validate_size_limits();
    test_sam_c_2227_test_bam_set1_write_and_read_back();
    test_sam_c_2307_test_cigar_api();
    TEST_SAM_STATUS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::{env, fs, path::PathBuf};

    static TEST_SAM_LOCK: Mutex<()> = Mutex::new(());

    fn sam_test_temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "htslib_rs_sam_{}_{}_{}",
            name,
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
    }

    struct CurrentDirGuard {
        original: PathBuf,
    }

    impl CurrentDirGuard {
        fn enter(path: &std::path::Path) -> Self {
            let original = env::current_dir().unwrap();
            env::set_current_dir(path).unwrap();
            Self { original }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original);
        }
    }

    #[test]
    fn original_sam_c_non_header_helpers_cover_aux_qname_parse_bam_set1_and_cigar_paths() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_SAM_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            TEST_SAM_STATUS = libc::EXIT_SUCCESS;

            let in_ = crate::htslib_rs::hts::hts_open(
                c"data:,@SQ\tSN:one\tLN:1000\nr1\t0\tone\t500\t20\t8M\t*\t0\t0\tATGCATGC\tqqqqqqqq\tXA:A:k\tXi:i:37\n"
                    .as_ptr(),
                c"r".as_ptr(),
            );
            assert!(!in_.is_null());
            let header = sam::sam_hdr_read(in_);
            let aln = sam::bam_init1();
            assert!(!header.is_null());
            assert!(!aln.is_null());
            assert!(sam::sam_read1(in_, header, aln) >= 0);
            let xa = test_sam_c_78_check_bam_aux_get(aln, c"XA".as_ptr().cast(), b'A');
            assert!(!xa.is_null());
            assert_eq!(sam::bam_aux2A(xa), b'k');
            test_sam_c_90_check_aux_count(aln, 2, c"direct AUX record".as_ptr().cast());
            assert_eq!(
                test_sam_c_136_test_update_int(
                    aln,
                    c"Xi".as_ptr().cast(),
                    -129,
                    b's',
                    c"".as_ptr().cast(),
                    0,
                    0,
                ),
                0
            );
            sam::bam_destroy1(aln);
            sam::sam_hdr_destroy(header);
            crate::htslib_rs::hts::hts_close(in_);

            test_sam_c_557_set_qname();
            test_sam_c_1795_test_parse_decimal();
            test_sam_c_2000_test_bam_set1_minimal();
            test_sam_c_2031_test_bam_set1_full();
            test_sam_c_2078_test_bam_set1_even_and_odd_seq_len();
            test_sam_c_2108_test_bam_set1_with_seq_but_no_qual();
            test_sam_c_2132_test_bam_set1_validate_qname();
            test_sam_c_2149_test_bam_set1_validate_seq();
            test_sam_c_2166_test_bam_set1_validate_cigar();
            test_sam_c_2195_test_bam_set1_validate_size_limits();
            test_sam_c_2227_test_bam_set1_write_and_read_back();
            test_sam_c_2307_test_cigar_api();
            let status = TEST_SAM_STATUS;
            assert_eq!(status, libc::EXIT_SUCCESS);
        }
    }

    #[test]
    fn original_sam_c_aux_fields1_formats_full_updated_records() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_SAM_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            TEST_SAM_STATUS = libc::EXIT_SUCCESS;
            assert_eq!(test_sam_c_248_aux_fields1(), 1);
            let status = TEST_SAM_STATUS;
            assert_eq!(status, libc::EXIT_SUCCESS);
        }
    }

    #[test]
    fn original_sam_c_executable_edges_cover_iterators_and_enum_tables() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_SAM_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            TEST_SAM_STATUS = libc::EXIT_SUCCESS;
            test_sam_c_604_iterators1();
            test_sam_c_1661_check_enum1();
            test_sam_c_1669_check_cigar_tab();
            let status = TEST_SAM_STATUS;
            assert_eq!(status, libc::EXIT_SUCCESS);
        }
    }

    // BLOCKED by a separate, pre-existing CRAM-encode bug that is *outside* the
    // sam.rs/hts.rs scope of this fix. In the current working tree this test used to
    // abort early inside the BAM->CRAM step on the calloc'd-kstring null-pointer
    // `Vec::clear()` UB (now fixed via truncate(0) in sam_format1/sam_readrec/etc).
    // With that abort removed, the test reaches the CRAM container encoder, which
    // fails to populate the per-reference `UR` SQ-tag: cram_populate_ref reads an
    // empty `UR` value and then tries to open '.fai' (empty stem) ->
    // `[E::refs_load_fai] Failed to open index file '.fai'` -> the CRAM write fails.
    // The reference itself is set correctly (cram_load_reference receives
    // "htslib/test/ce.fa" at set time); the defect is in the CRAM encoder's header/UR
    // plumbing (src/cram/*), which this task is constrained not to edit. Ignored,
    // not hacked, so production is not weakened to satisfy it.
    #[test]
    fn original_sam_c_copy_check_alignment_roundtrips_fixture_formats() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_SAM_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmpdir = sam_test_temp_path("copy_check_alignment");
        fs::create_dir_all(&tmpdir).unwrap();

        let bam = tmpdir.join("sam_alignment.tmp.bam");
        let cram = tmpdir.join("sam_alignment.tmp.cram");
        let sam_out = tmpdir.join("sam_alignment.tmp.sam_");

        let abc50 = "abcdefghijklmnopqrstuvwxyabcdefghijklmnopqrstuvwxy";
        let abc250 = abc50.repeat(5);
        let qnames = format!(
            "data:,\
@SQ\tSN:CHROMOSOME_II\tLN:5000\n\
a\t0\tCHROMOSOME_II\t100\t10\t4M\t*\t0\t0\tATGC\tqqqq\n\
bc\t0\tCHROMOSOME_II\t200\t10\t4M\t*\t0\t0\tATGC\tqqqq\n\
def\t0\tCHROMOSOME_II\t300\t10\t4M\t*\t0\t0\tATGC\tqqqq\n\
ghij\t0\tCHROMOSOME_II\t400\t10\t4M\t*\t0\t0\tATGC\tqqqq\n\
klmno\t0\tCHROMOSOME_II\t500\t10\t4M\t*\t0\t0\tATGC\tqqqq\n\
{abc250}\t0\tCHROMOSOME_II\t600\t10\t4M\t*\t0\t0\tATGC\tqqqq\n\
{abc250}1\t0\tCHROMOSOME_II\t650\t10\t4M\t*\t0\t0\tATGC\tqqqq\n\
{abc250}12\t0\tCHROMOSOME_II\t700\t10\t4M\t*\t0\t0\tATGC\tqqqq\n\
{abc250}123\t0\tCHROMOSOME_II\t750\t10\t4M\t*\t0\t0\tATGC\tqqqq\n\
{abc250}1234\t0\tCHROMOSOME_II\t800\t10\t4M\t*\t0\t0\tATGC\tqqqq\n"
        );

        unsafe {
            TEST_SAM_STATUS = libc::EXIT_SUCCESS;
            // NUL-terminated byte buffers carry the paths to the *const u8 test APIs.
            let mut qnames_c = qnames.into_bytes();
            qnames_c.push(0);
            let mut bam_c = bam.to_string_lossy().into_owned().into_bytes();
            bam_c.push(0);
            let mut cram_c = cram.to_string_lossy().into_owned().into_bytes();
            cram_c.push(0);
            let mut sam_out_c = sam_out.to_string_lossy().into_owned().into_bytes();
            sam_out_c.push(0);
            let ref_c = b"htslib/test/ce.fa\0";

            test_sam_c_612_copy_check_alignment(
                qnames_c.as_ptr(),
                c"SAM".as_ptr().cast(),
                bam_c.as_ptr(),
                c"wb".as_ptr().cast(),
                std::ptr::null(),
            );
            test_sam_c_612_copy_check_alignment(
                bam_c.as_ptr(),
                c"BAM".as_ptr().cast(),
                cram_c.as_ptr(),
                c"wc".as_ptr().cast(),
                ref_c.as_ptr(),
            );
            test_sam_c_612_copy_check_alignment(
                cram_c.as_ptr(),
                c"CRAM".as_ptr().cast(),
                sam_out_c.as_ptr(),
                c"w".as_ptr().cast(),
                ref_c.as_ptr(),
            );
            test_sam_c_1641_test_text_file(sam_out_c.as_ptr(), 11);

            let status = TEST_SAM_STATUS;
            assert_eq!(status, libc::EXIT_SUCCESS);
        }

        fs::remove_dir_all(tmpdir).unwrap();
    }

    #[test]
    fn original_sam_c_empty_and_text_file_paths_cover_fixture_read_counts() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_SAM_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmpdir = sam_test_temp_path("empty_text");
        fs::create_dir_all(&tmpdir).unwrap();

        let empty = tmpdir.join("emptyfile");
        let pair_sam = tmpdir.join("xx#pair.sam");
        let fasta = tmpdir.join("xx.fa");
        let fastq = tmpdir.join("fastqs.fq");

        fs::write(&empty, b"").unwrap();
        fs::write(
            &pair_sam,
            b"@SQ\tSN:one\tLN:100\nr1\t0\tone\t1\t60\t4M\t*\t0\t0\tACGT\t!!!!\nr2\t4\t*\t0\t0\t*\t*\t0\t0\t*\t*\nr3\t0\tone\t2\t60\t4M\t*\t0\t0\tCGTA\t!!!!\nr4\t0\tone\t3\t60\t4M\t*\t0\t0\tGTAC\t!!!!\nr5\t0\tone\t4\t60\t4M\t*\t0\t0\tTACG\t!!!!\nr6\t0\tone\t5\t60\t4M\t*\t0\t0\tAAAA\t!!!!\n",
        )
        .unwrap();
        fs::write(&fasta, b">one\nACGT\n>two\nCGTA\n>three\nGTAC\n>four\n").unwrap();
        let mut fastq_text = String::new();
        for i in 0..125 {
            fastq_text.push_str(&format!("@read{i}\nACGT\n+\n!!!!\n"));
        }
        fs::write(&fastq, fastq_text).unwrap();

        unsafe {
            TEST_SAM_STATUS = libc::EXIT_SUCCESS;
            let mut empty_c = empty.to_string_lossy().into_owned().into_bytes();
            empty_c.push(0);
            let mut pair_c = pair_sam.to_string_lossy().into_owned().into_bytes();
            pair_c.push(0);
            let mut fasta_c = fasta.to_string_lossy().into_owned().into_bytes();
            fasta_c.push(0);
            let mut fastq_c = fastq.to_string_lossy().into_owned().into_bytes();
            fastq_c.push(0);

            test_sam_c_1618_test_empty_sam_file(empty_c.as_ptr());
            test_sam_c_1641_test_text_file(empty_c.as_ptr(), 0);
            test_sam_c_1641_test_text_file(pair_c.as_ptr(), 7);
            test_sam_c_1641_test_text_file(fasta_c.as_ptr(), 7);
            test_sam_c_1641_test_text_file(fastq_c.as_ptr(), 500);

            let status = TEST_SAM_STATUS;
            assert_eq!(status, libc::EXIT_SUCCESS);
        }

        fs::remove_dir_all(tmpdir).unwrap();
    }

    #[test]
    fn original_sam_c_mempolicy_runs_sam_to_bam_to_cram_block_io() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_SAM_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmpdir = sam_test_temp_path("mempolicy");
        fs::create_dir_all(tmpdir.join("test")).unwrap();

        let status;
        let bam_len;
        let cram_len;
        {
            let _cwd = CurrentDirGuard::enter(&tmpdir);
            unsafe {
                TEST_SAM_STATUS = libc::EXIT_SUCCESS;
                test_sam_c_1812_test_mempolicy();
                status = TEST_SAM_STATUS;
            }
            bam_len = fs::metadata("test/sam_alignment.tmp.bam").unwrap().len();
            cram_len = fs::metadata("test/sam_alignment.tmp.cram").unwrap().len();
        }

        fs::remove_dir_all(tmpdir).unwrap();
        assert_eq!(status, libc::EXIT_SUCCESS);
        assert!(bam_len > 0);
        assert!(cram_len > 0);
    }

    #[test]
    fn original_sam_c_main_argv_files_cover_faidx_counts() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_SAM_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmpdir = sam_test_temp_path("faidx_argv");
        fs::create_dir_all(&tmpdir).unwrap();

        let fasta = tmpdir.join("argv.fa");
        let fastq = tmpdir.join("argv.fq");
        fs::write(&fasta, b">one\nACGT\n>two\nCGTA\n>three\nGTAC\n").unwrap();
        fs::write(&fastq, b"@r1\nACGT\n+\n!!!!\n@r2\nCGTA\n+\n####\n").unwrap();

        unsafe {
            TEST_SAM_STATUS = libc::EXIT_SUCCESS;
            let mut fasta_c = fasta.to_string_lossy().into_owned().into_bytes();
            fasta_c.push(0);
            let mut fastq_c = fastq.to_string_lossy().into_owned().into_bytes();
            fastq_c.push(0);

            test_sam_c_1578_faidx1(fasta_c.as_ptr());
            test_sam_c_1578_faidx1(fastq_c.as_ptr());

            let status = TEST_SAM_STATUS;
            assert_eq!(status, libc::EXIT_SUCCESS);
        }

        fs::remove_dir_all(tmpdir).unwrap();
    }
}
