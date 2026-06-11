use htslib_rs::{
    bcf_destroy, bcf_hdr_destroy, bcf_hdr_fmt_text, bcf_hdr_write, bcf_init, bcf_subset_format,
    bcf_write, hts_close, hts_open, ks_free, kstring_t, vcf_format, vcf_hdr_read, vcf_read,
};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_fixture(path: &str) -> Vec<u8> {
    let mut bytes = fixture(path).to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

unsafe fn render_vcf(path: &str) -> String {
    render_vcf_path(&fixture(path))
}

unsafe fn render_vcf_path(path: &std::path::Path) -> String {
    let mut path_c = path.to_string_lossy().into_owned().into_bytes();
    path_c.push(0);
    let fp = hts_open(path_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.display());

    let hdr = vcf_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read VCF header from {}",
        path.display()
    );

    let mut len = 0;
    let header_text = bcf_hdr_fmt_text(hdr, 0, &mut len);
    assert!(!header_text.is_null());
    let header_bytes = std::ffi::CStr::from_ptr(header_text.cast()).to_bytes();
    let mut out = String::from_utf8_lossy(header_bytes).into_owned();
    assert_eq!(out.len(), len as usize);
    // header_text was C-allocated; ownership handled at the FFI boundary.

    let rec = bcf_init();
    assert!(!rec.is_null());
    let mut line = kstring_t::default();
    loop {
        let ret = vcf_read(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert_eq!(bcf_subset_format(hdr, rec), 0);
        line.data.clear();
        assert_eq!(vcf_format(hdr, rec, &mut line), 0);
        out.push_str(&String::from_utf8_lossy(&line.data));
    }

    ks_free(&mut line);
    bcf_destroy(rec);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe fn write_bcf_copy(vcf_path: &str, label: &str) -> std::path::PathBuf {
    let out_path = std::env::temp_dir().join(format!(
        "htslib_rs-vcf-roundtrip-{}-{}.bcf",
        std::process::id(),
        label
    ));
    let _ = std::fs::remove_file(&out_path);

    let in_path = c_fixture(vcf_path);
    let in_fp = hts_open(in_path.as_ptr().cast(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {vcf_path}");
    let hdr = vcf_hdr_read(in_fp);
    assert!(!hdr.is_null());

    let mut out_path_c = out_path.to_string_lossy().into_owned().into_bytes();
    out_path_c.push(0);
    let out_fp = hts_open(out_path_c.as_ptr().cast(), c"wb".as_ptr());
    assert!(!out_fp.is_null(), "failed to write {}", out_path.display());
    assert_eq!(bcf_hdr_write(out_fp, hdr), 0);

    let rec = bcf_init();
    assert!(!rec.is_null());
    while vcf_read(in_fp, hdr, rec) >= 0 {
        assert_eq!(bcf_write(out_fp, hdr, rec), 0);
    }

    bcf_destroy(rec);
    assert_eq!(hts_close(out_fp), 0);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    out_path
}

#[test]
fn original_formatmissing_vcf_renders_exact_expected_output() {
    unsafe {
        assert_eq!(
            render_vcf("htslib/test/formatmissing.vcf"),
            std::fs::read_to_string(fixture("htslib/test/formatmissing-out.vcf")).unwrap()
        );
    }
}

#[test]
fn original_noroundtrip_vcf_renders_exact_expected_output() {
    unsafe {
        assert_eq!(
            render_vcf("htslib/test/noroundtrip.vcf"),
            std::fs::read_to_string(fixture("htslib/test/noroundtrip-out.vcf")).unwrap()
        );
    }
}

#[test]
fn original_tabix_bcf_renders_exact_vcf_fixture() {
    unsafe {
        assert_eq!(
            render_vcf("htslib/test/tabix/vcf_file.bcf"),
            std::fs::read_to_string(fixture("htslib/test/tabix/vcf_file.vcf")).unwrap()
        );
    }
}

#[test]
fn original_vcf44_1_renders_exact_expected_output() {
    unsafe {
        assert_eq!(
            render_vcf("htslib/test/vcf44_1.vcf"),
            std::fs::read_to_string(fixture("htslib/test/vcf44_1.expected")).unwrap()
        );
    }
}

#[test]
fn original_formatcols_vcf_bcf_roundtrip_preserves_rendered_records() {
    unsafe {
        let bcf = write_bcf_copy("htslib/test/formatcols.vcf", "formatcols");
        assert_eq!(
            render_vcf_path(&bcf),
            std::fs::read_to_string(fixture("htslib/test/formatcols.vcf")).unwrap()
        );
        let _ = std::fs::remove_file(bcf);
    }
}
