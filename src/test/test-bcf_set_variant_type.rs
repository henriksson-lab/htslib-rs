use std::ffi::{c_char, c_int};

use crate::htslib_rs::vcf;

// original: error (htslib/test/test-bcf_set_variant_type.c:32)
pub unsafe fn test_test_bcf_set_variant_type_c_32_error(format: *const c_char) -> ! {
    libc::fputs(format, hts_sys::stderr.cast());
    if libc::strrchr(format, b'\n' as c_int).is_null() {
        libc::fputc(b'\n' as c_int, hts_sys::stderr.cast());
    }
    libc::exit(-1);
}

// original: test_bcf_set_variant_type (htslib/test/test-bcf_set_variant_type.c:42)
pub unsafe fn test_test_bcf_set_variant_type_c_42_test_bcf_set_variant_type() {
    // Test SNVs
    let mut var1 = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"T".as_ptr(), &mut var1);
    if var1.type_ != hts_sys::VCF_SNP as c_int {
        test_test_bcf_set_variant_type_c_32_error(c"A -> T was not detected as a SNP".as_ptr());
    }

    // Test INDEL
    let mut var2a = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"AA".as_ptr(), &mut var2a);
    if var2a.type_ != (hts_sys::VCF_INDEL | vcf::VCF_INS) as c_int {
        test_test_bcf_set_variant_type_c_32_error(c"A -> AA was not detected as an INDEL".as_ptr());
    }
    let mut var2b = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"AA".as_ptr(), c"A".as_ptr(), &mut var2b);
    if var2b.type_ != (hts_sys::VCF_INDEL | vcf::VCF_DEL) as c_int {
        test_test_bcf_set_variant_type_c_32_error(c"AA -> A was not detected as a INDEL".as_ptr());
    }

    // Test breakends
    let mut var3a = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"N".as_ptr(), c"N]16:33625444]".as_ptr(), &mut var3a);
    if var3a.type_ != hts_sys::VCF_BND as c_int {
        test_test_bcf_set_variant_type_c_32_error(
            c"N]16:33625444] was not detected as a breakend".as_ptr(),
        );
    }

    let mut var3b = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"N".as_ptr(), c"N[16:33625444[".as_ptr(), &mut var3b);
    if var3b.type_ != hts_sys::VCF_BND as c_int {
        test_test_bcf_set_variant_type_c_32_error(
            c"N[16:33625444[ was not detected as a breakend".as_ptr(),
        );
    }

    let mut var3c = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"N".as_ptr(), c"]16:33625444]N".as_ptr(), &mut var3c);
    if var3c.type_ != hts_sys::VCF_BND as c_int {
        test_test_bcf_set_variant_type_c_32_error(
            c"]16:33625444]N was not detected as a breakend".as_ptr(),
        );
    }

    let mut var3d = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"N".as_ptr(), c"[16:33625444[N".as_ptr(), &mut var3d);
    if var3d.type_ != hts_sys::VCF_BND as c_int {
        test_test_bcf_set_variant_type_c_32_error(
            c"[16:33625444[N was not detected as a breakend".as_ptr(),
        );
    }

    vcf::vcf_c_5373_bcf_set_variant_type(
        c"T".as_ptr(),
        c"]chrB:123]AGTNNNNNCAT".as_ptr(),
        &mut var3d,
    );
    if var3d.type_ != hts_sys::VCF_BND as c_int {
        test_test_bcf_set_variant_type_c_32_error(
            c"]chrB:123]AGTNNNNNCAT was not detected as a breakend".as_ptr(),
        );
    }
    vcf::vcf_c_5373_bcf_set_variant_type(
        c"C".as_ptr(),
        c"CAGTNNNNNCA[chrA:321[".as_ptr(),
        &mut var3d,
    );
    if var3d.type_ != hts_sys::VCF_BND as c_int {
        test_test_bcf_set_variant_type_c_32_error(
            c"CAGTNNNNNCA[chrA:321[ was not detected as a breakend".as_ptr(),
        );
    }

    // Test special reference alleles
    let mut var4a = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"<NON_REF>".as_ptr(), &mut var4a);
    if var4a.type_ != hts_sys::VCF_REF as c_int {
        test_test_bcf_set_variant_type_c_32_error(
            c"<NON_REF> was not detected as a special reference allele".as_ptr(),
        );
    }
    let mut var4b = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"<*>".as_ptr(), &mut var4b);
    if var4b.type_ != hts_sys::VCF_REF as c_int {
        test_test_bcf_set_variant_type_c_32_error(
            c"<*> was not detected as a special reference allele".as_ptr(),
        );
    }
    // Test MNP
    let mut var5 = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"AA".as_ptr(), c"TT".as_ptr(), &mut var5);
    if var5.type_ != hts_sys::VCF_MNP as c_int {
        test_test_bcf_set_variant_type_c_32_error(c"AA->TT was not detected as a MNP".as_ptr());
    }
    // Test Overlapping allele
    let mut var6 = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"*".as_ptr(), &mut var6);
    if var6.type_ != hts_sys::VCF_OVERLAP as c_int {
        test_test_bcf_set_variant_type_c_32_error(c"A->* was not detected as an overlap".as_ptr());
    }
    // Test .
    let mut var7 = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c".".as_ptr(), &mut var7);
    if var7.type_ != hts_sys::VCF_REF as c_int {
        test_test_bcf_set_variant_type_c_32_error(
            c"A->. was not detected as a special reference allele".as_ptr(),
        );
    }
}

// original: main (htslib/test/test-bcf_set_variant_type.c:143)
pub unsafe fn test_test_bcf_set_variant_type_c_143_main(
    _argc: c_int,
    _argv: *mut *mut c_char,
) -> c_int {
    test_test_bcf_set_variant_type_c_42_test_bcf_set_variant_type();
    0
}
