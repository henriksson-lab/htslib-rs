const HTS_VERSION: i32 = 102390;

// original: main (htslib/test/test_introspection.c:31)
pub unsafe fn test_test_introspection_c_31_main() -> i32 {
    println!(
        "Version string: {}",
        String::from_utf8_lossy(
            std::ffi::CStr::from_ptr(crate::htslib_rs::hts::hts_version()).to_bytes()
        ),
    );
    println!("Version number: {}", HTS_VERSION);
    println!(
        "\nhtscodecs version: {}",
        String::from_utf8_lossy(
            std::ffi::CStr::from_ptr(crate::htslib_rs::hts::hts_test_feature(
                crate::htslib_rs::hts::HTS_FEATURE_HTSCODECS
            ))
            .to_bytes()
        ),
    );

    println!(
        "\nCC:             {}",
        String::from_utf8_lossy(
            std::ffi::CStr::from_ptr(crate::htslib_rs::hts::hts_test_feature(
                crate::htslib_rs::hts::HTS_FEATURE_CC
            ))
            .to_bytes()
        ),
    );
    println!(
        "CPPFLAGS:       {}",
        String::from_utf8_lossy(
            std::ffi::CStr::from_ptr(crate::htslib_rs::hts::hts_test_feature(
                crate::htslib_rs::hts::HTS_FEATURE_CPPFLAGS
            ))
            .to_bytes()
        ),
    );
    println!(
        "CFLAGS:         {}",
        String::from_utf8_lossy(
            std::ffi::CStr::from_ptr(crate::htslib_rs::hts::hts_test_feature(
                crate::htslib_rs::hts::HTS_FEATURE_CFLAGS
            ))
            .to_bytes()
        ),
    );
    println!(
        "LDFLAGS:        {}",
        String::from_utf8_lossy(
            std::ffi::CStr::from_ptr(crate::htslib_rs::hts::hts_test_feature(
                crate::htslib_rs::hts::HTS_FEATURE_LDFLAGS
            ))
            .to_bytes()
        ),
    );

    let feat = crate::htslib_rs::hts::hts_features();
    println!("\nFeature number: 0x{:x}", feat);
    if feat & crate::htslib_rs::hts::HTS_FEATURE_CONFIGURE != 0 {
        println!("                HTS_FEATURE_CONFIGURE");
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_PLUGINS != 0 {
        println!("                HTS_FEATURE_PLUGINS");
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_LIBCURL != 0 {
        println!("                HTS_FEATURE_LIBCURL");
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_S3 != 0 {
        println!("                HTS_FEATURE_S3");
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_GCS != 0 {
        println!("                HTS_FEATURE_GCS");
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_LIBDEFLATE != 0 {
        println!("                HTS_FEATURE_LIBDEFLATE");
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_LZMA != 0 {
        println!("                HTS_FEATURE_LZMA");
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_BZIP2 != 0 {
        println!("                HTS_FEATURE_BZIP2");
    }
    if feat & crate::htslib_rs::hts::HTS_FEATURE_HTSCODECS != 0 {
        println!("                HTS_FEATURE_HTSCODECS");
    }

    println!(
        "\nFeature string: {}",
        String::from_utf8_lossy(
            std::ffi::CStr::from_ptr(crate::htslib_rs::hts::hts_feature_string()).to_bytes()
        ),
    );

    println!("\nPlugins present:");
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

        println!(
            "    {}:",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(plugins[i as usize].cast()).to_bytes())
        );
        for j in 0..nschemes {
            println!(
                "\t{}",
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(sc_list[j as usize].cast()).to_bytes())
            );
        }
        println!();
    }

    0
}
