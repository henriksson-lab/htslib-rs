use std::ffi::c_int;

const HTS_VERSION: c_int = 102390;

// original: main (htslib/test/test_introspection.c:31)
pub unsafe fn test_test_introspection_c_31_main() -> c_int {
    libc::printf(
        c"Version string: %s\n".as_ptr(),
        crate::htslib_rs::hts::hts_version(),
    );
    libc::printf(c"Version number: %d\n".as_ptr(), HTS_VERSION);
    libc::printf(
        c"\nhtscodecs version: %s\n".as_ptr(),
        crate::htslib_rs::hts::hts_test_feature(crate::htslib_rs::hts::HTS_FEATURE_HTSCODECS),
    );

    libc::printf(
        c"\nCC:             %s\n".as_ptr(),
        crate::htslib_rs::hts::hts_test_feature(crate::htslib_rs::hts::HTS_FEATURE_CC),
    );
    libc::printf(
        c"CPPFLAGS:       %s\n".as_ptr(),
        crate::htslib_rs::hts::hts_test_feature(crate::htslib_rs::hts::HTS_FEATURE_CPPFLAGS),
    );
    libc::printf(
        c"CFLAGS:         %s\n".as_ptr(),
        crate::htslib_rs::hts::hts_test_feature(crate::htslib_rs::hts::HTS_FEATURE_CFLAGS),
    );
    libc::printf(
        c"LDFLAGS:        %s\n".as_ptr(),
        crate::htslib_rs::hts::hts_test_feature(crate::htslib_rs::hts::HTS_FEATURE_LDFLAGS),
    );

    let feat = crate::htslib_rs::hts::hts_features();
    libc::printf(c"\nFeature number: 0x%x\n".as_ptr(), feat);
    if feat & crate::htslib_rs::hts::HTS_FEATURE_CONFIGURE != 0 {
        libc::printf(c"                HTS_FEATURE_CONFIGURE\n".as_ptr());
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_PLUGINS != 0 {
        libc::printf(c"                HTS_FEATURE_PLUGINS\n".as_ptr());
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_LIBCURL != 0 {
        libc::printf(c"                HTS_FEATURE_LIBCURL\n".as_ptr());
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_S3 != 0 {
        libc::printf(c"                HTS_FEATURE_S3\n".as_ptr());
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_GCS != 0 {
        libc::printf(c"                HTS_FEATURE_GCS\n".as_ptr());
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_LIBDEFLATE != 0 {
        libc::printf(c"                HTS_FEATURE_LIBDEFLATE\n".as_ptr());
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_LZMA != 0 {
        libc::printf(c"                HTS_FEATURE_LZMA\n".as_ptr());
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_BZIP2 != 0 {
        libc::printf(c"                HTS_FEATURE_BZIP2\n".as_ptr());
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_HTSCODECS != 0 {
        libc::printf(c"                HTS_FEATURE_HTSCODECS\n".as_ptr());
    }

    libc::printf(
        c"\nFeature string: %s\n".as_ptr(),
        crate::htslib_rs::hts::hts_feature_string(),
    );

    libc::printf(c"\nPlugins present:\n".as_ptr());
    let mut plugins = [std::ptr::null(); 100];
    let mut np = 100;
    if crate::htslib_rs::hfile::hfile_list_plugins(plugins.as_mut_ptr(), &mut np) < 0 {
        return 1;
    }

    for i in 0..np {
        let mut sc_list = [std::ptr::null(); 100];
        let mut nschemes = 100;
        if crate::htslib_rs::hfile::hfile_list_schemes(
            plugins[i as usize],
            sc_list.as_mut_ptr(),
            &mut nschemes,
        ) < 0
        {
            return 1;
        }

        libc::printf(c"    %s:\n".as_ptr(), plugins[i as usize]);
        for j in 0..nschemes {
            libc::printf(c"\t%s\n".as_ptr(), sc_list[j as usize]);
        }
        libc::puts(c"".as_ptr());
    }

    0
}
