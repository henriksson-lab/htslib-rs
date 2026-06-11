use crate::htslib_rs::vcf;

// original: error (htslib/test/test-bcf_set_variant_type.c:32)
pub unsafe fn test_test_bcf_set_variant_type_c_32_error(format: &[u8]) -> ! {
    eprintln!("{}", String::from_utf8_lossy(format));
    libc::exit(-1);
}

// original: test_bcf_set_variant_type (htslib/test/test-bcf_set_variant_type.c:42)
pub unsafe fn test_test_bcf_set_variant_type_c_42_test_bcf_set_variant_type() {
    // Test SNVs
    let mut var1 = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"A\0".as_ptr().cast(),
        b"T\0".as_ptr().cast(),
        &mut var1,
    );
    if var1.type_ != crate::htslib_rs::vcf::VCF_SNP as i32 {
        test_test_bcf_set_variant_type_c_32_error(b"A -> T was not detected as a SNP");
    }

    // Test INDEL
    let mut var2a = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"A\0".as_ptr().cast(),
        b"AA\0".as_ptr().cast(),
        &mut var2a,
    );
    if var2a.type_ != (crate::htslib_rs::vcf::VCF_INDEL | vcf::VCF_INS) as i32 {
        test_test_bcf_set_variant_type_c_32_error(b"A -> AA was not detected as an INDEL");
    }
    let mut var2b = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"AA\0".as_ptr().cast(),
        b"A\0".as_ptr().cast(),
        &mut var2b,
    );
    if var2b.type_ != (crate::htslib_rs::vcf::VCF_INDEL | vcf::VCF_DEL) as i32 {
        test_test_bcf_set_variant_type_c_32_error(b"AA -> A was not detected as a INDEL");
    }

    // Test breakends
    let mut var3a = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"N\0".as_ptr().cast(),
        b"N]16:33625444]\0".as_ptr().cast(),
        &mut var3a,
    );
    if var3a.type_ != crate::htslib_rs::vcf::VCF_BND as i32 {
        test_test_bcf_set_variant_type_c_32_error(b"N]16:33625444] was not detected as a breakend");
    }

    let mut var3b = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"N\0".as_ptr().cast(),
        b"N[16:33625444[\0".as_ptr().cast(),
        &mut var3b,
    );
    if var3b.type_ != crate::htslib_rs::vcf::VCF_BND as i32 {
        test_test_bcf_set_variant_type_c_32_error(b"N[16:33625444[ was not detected as a breakend");
    }

    let mut var3c = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"N\0".as_ptr().cast(),
        b"]16:33625444]N\0".as_ptr().cast(),
        &mut var3c,
    );
    if var3c.type_ != crate::htslib_rs::vcf::VCF_BND as i32 {
        test_test_bcf_set_variant_type_c_32_error(b"]16:33625444]N was not detected as a breakend");
    }

    let mut var3d = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"N\0".as_ptr().cast(),
        b"[16:33625444[N\0".as_ptr().cast(),
        &mut var3d,
    );
    if var3d.type_ != crate::htslib_rs::vcf::VCF_BND as i32 {
        test_test_bcf_set_variant_type_c_32_error(b"[16:33625444[N was not detected as a breakend");
    }

    vcf::vcf_c_5373_bcf_set_variant_type(
        b"T\0".as_ptr().cast(),
        b"]chrB:123]AGTNNNNNCAT\0".as_ptr().cast(),
        &mut var3d,
    );
    if var3d.type_ != crate::htslib_rs::vcf::VCF_BND as i32 {
        test_test_bcf_set_variant_type_c_32_error(
            b"]chrB:123]AGTNNNNNCAT was not detected as a breakend",
        );
    }
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"C\0".as_ptr().cast(),
        b"CAGTNNNNNCA[chrA:321[\0".as_ptr().cast(),
        &mut var3d,
    );
    if var3d.type_ != crate::htslib_rs::vcf::VCF_BND as i32 {
        test_test_bcf_set_variant_type_c_32_error(
            b"CAGTNNNNNCA[chrA:321[ was not detected as a breakend",
        );
    }

    // Test special reference alleles
    let mut var4a = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"A\0".as_ptr().cast(),
        b"<NON_REF>\0".as_ptr().cast(),
        &mut var4a,
    );
    if var4a.type_ != crate::htslib_rs::vcf::VCF_REF as i32 {
        test_test_bcf_set_variant_type_c_32_error(
            b"<NON_REF> was not detected as a special reference allele",
        );
    }
    let mut var4b = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"A\0".as_ptr().cast(),
        b"<*>\0".as_ptr().cast(),
        &mut var4b,
    );
    if var4b.type_ != crate::htslib_rs::vcf::VCF_REF as i32 {
        test_test_bcf_set_variant_type_c_32_error(
            b"<*> was not detected as a special reference allele",
        );
    }
    // Test MNP
    let mut var5 = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"AA\0".as_ptr().cast(),
        b"TT\0".as_ptr().cast(),
        &mut var5,
    );
    if var5.type_ != crate::htslib_rs::vcf::VCF_MNP as i32 {
        test_test_bcf_set_variant_type_c_32_error(b"AA->TT was not detected as a MNP");
    }
    // Test Overlapping allele
    let mut var6 = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"A\0".as_ptr().cast(),
        b"*\0".as_ptr().cast(),
        &mut var6,
    );
    if var6.type_ != crate::htslib_rs::vcf::VCF_OVERLAP as i32 {
        test_test_bcf_set_variant_type_c_32_error(b"A->* was not detected as an overlap");
    }
    // Test .
    let mut var7 = vcf::bcf_variant_t { type_: 0, n: 0 };
    vcf::vcf_c_5373_bcf_set_variant_type(
        b"A\0".as_ptr().cast(),
        b".\0".as_ptr().cast(),
        &mut var7,
    );
    if var7.type_ != crate::htslib_rs::vcf::VCF_REF as i32 {
        test_test_bcf_set_variant_type_c_32_error(
            b"A->. was not detected as a special reference allele",
        );
    }
}

// original: main (htslib/test/test-bcf_set_variant_type.c:143)
pub unsafe fn test_test_bcf_set_variant_type_c_143_main(
    _argc: i32,
    _argv: *mut *mut u8,
) -> i32 {
    test_test_bcf_set_variant_type_c_42_test_bcf_set_variant_type();
    0
}
