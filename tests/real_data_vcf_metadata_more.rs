use htslib_mini_rs::{
    bcf_destroy, bcf_get_format_string, bcf_get_format_values, bcf_get_info, bcf_hdr_destroy,
    bcf_hdr_fmt_text, bcf_hdr_get_version, bcf_hdr_id2int, bcf_hdr_id2name, bcf_hdr_name2id,
    bcf_hdr_read, bcf_hdr_seqnames, bcf_init, bcf_read, bcf_seqname, bcf_unpack, hts_close,
    hts_open, ks_free, kstring_t, vcf_format, vcf_hdr_read,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn fixture_text(path: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(path).unwrap()
}

fn tmp_path(label: &str, extension: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "htslib-mini-rs-{}-{}-{}.{}",
        label,
        std::process::id(),
        std::thread::current().name().unwrap_or("test"),
        extension
    ))
}

unsafe fn header_text(hdr: *const htslib_mini_rs::bcf_hdr_t) -> String {
    let mut len = 0;
    let text = bcf_hdr_fmt_text(hdr, 0, &mut len);
    assert!(!text.is_null());
    let out = CStr::from_ptr(text).to_string_lossy().into_owned();
    assert_eq!(out.len(), len as usize);
    libc::free(text.cast());
    out
}

unsafe fn render_vcf(path: &str) -> String {
    let vcf = c_fixture(path);
    let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null());

    let hdr = vcf_hdr_read(fp);
    assert!(!hdr.is_null());
    let mut out = header_text(hdr);

    let rec = bcf_init();
    assert!(!rec.is_null());
    let mut line = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    while bcf_read(fp, hdr, rec) >= 0 {
        line.l = 0;
        assert_eq!(vcf_format(hdr, rec, &mut line), 0);
        assert!(!line.s.is_null());
        out.push_str(&CStr::from_ptr(line.s).to_string_lossy());
    }

    ks_free(&mut line);
    bcf_destroy(rec);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe fn assert_sample_names(hdr: *const htslib_mini_rs::bcf_hdr_t, expected: &[&[u8]]) {
    assert_eq!(
        (*hdr).n[hts_sys::BCF_DT_SAMPLE as usize] as usize,
        expected.len()
    );
    assert!(!(*hdr).samples.is_null());

    for (i, name) in expected.iter().enumerate() {
        let sample = *(*hdr).samples.add(i);
        assert!(!sample.is_null());
        assert_eq!(CStr::from_ptr(sample).to_bytes(), *name);
        assert_eq!(
            bcf_hdr_id2int(hdr, hts_sys::BCF_DT_SAMPLE as c_int, sample.cast_const()),
            i as c_int
        );
    }
}

unsafe fn assert_alleles(rec: *mut htslib_mini_rs::bcf1_t, expected: &[&CStr]) {
    assert_eq!(bcf_unpack(rec, hts_sys::BCF_UN_STR as c_int), 0);
    assert_eq!((*rec).n_allele() as usize, expected.len());
    for (i, allele) in expected.iter().enumerate() {
        assert_eq!(CStr::from_ptr(*(*rec).d.allele.add(i)), *allele);
    }
}

#[test]
fn formatcols_vcf_renders_exact_original_fixture() {
    unsafe {
        assert_eq!(
            render_vcf("htslib/test/formatcols.vcf"),
            fixture_text("htslib/test/formatcols.vcf")
        );
    }
}

#[test]
fn formatcols_vcf_preserves_utf8_samples_and_string_format_values() {
    unsafe {
        let vcf = c_fixture("htslib/test/formatcols.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(CStr::from_ptr(bcf_hdr_get_version(hdr)), c"VCFv4.3");
        assert_eq!(bcf_hdr_name2id(hdr, c"1".as_ptr()), 0);
        assert_eq!(CStr::from_ptr(bcf_hdr_id2name(hdr, 0)), c"1");
        assert_sample_names(hdr, &[b"S1", b"S\xc2\xb2", b"S3"]);

        let text = header_text(hdr);
        assert!(text.contains("##FORMAT=<ID=S,Number=1,Type=String,Description=\"Text\">\n"));
        assert!(text.contains("#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tS1\tS"));

        let rec = bcf_init();
        assert!(!rec.is_null());
        assert!(bcf_read(fp, hdr, rec) >= 0);
        assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"1");
        assert_eq!((*rec).pos, 99);
        assert_eq!((*rec).rlen, 1);
        assert_eq!((*rec).n_sample(), 3);
        assert_alleles(rec, &[c"A", c"T"]);

        let mut values: *mut *mut c_char = std::ptr::null_mut();
        let mut nvalues = 0;
        assert_eq!(
            bcf_get_format_string(hdr, rec, c"S".as_ptr(), &mut values, &mut nvalues),
            30
        );
        assert!(nvalues >= 3);
        assert_eq!(CStr::from_ptr(*values), c"a");
        assert_eq!(CStr::from_ptr(*values.add(1)), c"bbbbbbb");
        assert_eq!(CStr::from_ptr(*values.add(2)), c"ccccccccc");
        libc::free((*values).cast());
        libc::free(values.cast());

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn test_vcf_hdr_in_keeps_loose_format_metadata_and_first_record_values() {
    unsafe {
        let vcf = c_fixture("htslib/test/test-vcf-hdr-in.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(CStr::from_ptr(bcf_hdr_get_version(hdr)), c"VCFv4.1");
        assert_sample_names(hdr, &[b"NA00001"]);

        let text = header_text(hdr);
        assert!(text.contains("##fileDate=20150126\n"));
        assert!(text.contains("##reference=hs37d5\n"));
        assert!(text.contains(
            "##FORMAT=<ID=GQ,Number=1,Type=Integer,EmptyWithSpace=,Description=\"Genotype Quality\">\n"
        ));
        assert!(text.contains(
            "##FORMAT=<ID=GATK,Number=1,Type=String,Description=\"Genotype as called by GATK. Always a diploid call. All other genotype stats based on this genotype.\">\n"
        ));

        let rec = bcf_init();
        assert!(!rec.is_null());
        assert!(bcf_read(fp, hdr, rec) >= 0);
        assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"1");
        assert_eq!((*rec).pos, 12_065_946);
        assert_eq!((*rec).rlen, 1);
        assert_eq!((*rec).n_sample(), 1);
        assert_alleles(rec, &[c"C", c"T", c"A"]);

        let mut ad = std::ptr::null_mut();
        let mut nad = 0;
        assert_eq!(
            bcf_get_format_values(
                hdr,
                rec,
                c"AD".as_ptr(),
                &mut ad,
                &mut nad,
                hts_sys::BCF_HT_INT as c_int,
            ),
            2
        );
        assert!(nad >= 2);
        assert_eq!(std::slice::from_raw_parts(ad.cast::<i32>(), 2), &[3, 2]);
        libc::free(ad);

        let mut dp = std::ptr::null_mut();
        let mut ndp = 0;
        assert_eq!(
            bcf_get_format_values(
                hdr,
                rec,
                c"DP".as_ptr(),
                &mut dp,
                &mut ndp,
                hts_sys::BCF_HT_INT as c_int,
            ),
            1
        );
        assert!(ndp >= 1);
        assert_eq!(*dp.cast::<i32>(), 5);
        libc::free(dp);

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn vcf_meta_meta_vcf_renders_exact_original_fixture() {
    unsafe {
        assert_eq!(
            render_vcf("htslib/test/vcf_meta_meta.vcf"),
            fixture_text("htslib/test/vcf_meta_meta.vcf")
        );
    }
}

#[test]
fn vcf_meta_meta_header_keeps_meta_records_and_deletion_metadata() {
    unsafe {
        let vcf = c_fixture("htslib/test/vcf_meta_meta.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(CStr::from_ptr(bcf_hdr_get_version(hdr)), c"VCFv4.3");
        assert_eq!((*hdr).n[hts_sys::BCF_DT_SAMPLE as usize], 0);
        assert_eq!(bcf_hdr_name2id(hdr, c"1".as_ptr()), 0);

        let text = header_text(hdr);
        assert!(
            text.contains("##META=<ID=Assay,Number=.,Type=String,Values=[WholeGenome, Exome]>\n")
        );
        assert!(text.contains("##META=<ID=Disease,Number=.,Type=String,Values=[None, Cancer]>\n"));
        assert!(text
            .contains("##META=<ID=Ethnicity,Number=.,Type=String,Values=[AFR, CEU, ASN, MEX]>\n"));
        assert!(text.contains(
            "##META=<ID=Tissue,Number=.,Type=String,Values=[Blood, Breast, Colon, Lung, ?]>\n"
        ));

        let rec = bcf_init();
        assert!(!rec.is_null());
        assert!(bcf_read(fp, hdr, rec) >= 0);
        assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"1");
        assert_eq!((*rec).pos, 122);
        assert_eq!((*rec).rlen, 2);
        assert_eq!((*rec).n_info(), 0);
        assert_alleles(rec, &[c"TC", c"T"]);

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn large_chr_vcf_preserves_contig_order_lengths_and_terminal_record_metadata() {
    unsafe {
        let vcf = c_fixture("htslib/test/tabix/large_chr.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(CStr::from_ptr(bcf_hdr_get_version(hdr)), c"VCFv4.2");
        assert_eq!(bcf_hdr_name2id(hdr, c"chr11".as_ptr()), 0);
        assert_eq!(bcf_hdr_name2id(hdr, c"chr20".as_ptr()), 1);

        let mut nseqs = 0;
        let seqs = bcf_hdr_seqnames(hdr, &mut nseqs);
        assert!(!seqs.is_null());
        assert_eq!(nseqs, 2);
        assert_eq!(CStr::from_ptr(*seqs), c"chr11");
        assert_eq!(CStr::from_ptr(*seqs.add(1)), c"chr20");
        libc::free(seqs.cast());

        let text = header_text(hdr);
        assert!(text.contains("##reference=file:///seq/references/long_chrom.fasta\n"));
        assert!(text.contains("##contig=<ID=chr11,length=116870911>\n"));
        assert!(text.contains("##contig=<ID=chr20,length=2147483647>\n"));

        let rec = bcf_init();
        assert!(!rec.is_null());
        let mut count = 0;
        while bcf_read(fp, hdr, rec) >= 0 {
            count += 1;
            if count == 12 {
                assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"chr20");
                assert_eq!((*rec).pos, 2_147_483_646);
                assert_eq!((*rec).rlen, 1);
                assert_eq!((*rec).n_info(), 0);
                assert_alleles(rec, &[c"A", c"T"]);
            }
        }
        assert_eq!(count, 12);

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn large_info_end_repair_keeps_packed_info_prefix() {
    let input = tmp_path("large-end-input", "vcf");
    let _ = std::fs::remove_file(&input);

    std::fs::write(
        &input,
        concat!(
            "##fileformat=VCFv4.2\n",
            "##contig=<ID=chr1,length=4000000000>\n",
            "##INFO=<ID=END,Number=1,Type=Integer,Description=\"End position\">\n",
            "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n",
            "chr1\t1\t.\tA\t<DEL>\t.\t.\tEND=3000000000\n",
        ),
    )
    .unwrap();

    unsafe {
        let input_c = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let fp = hts_open(input_c.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());
        assert_eq!(bcf_read(fp, hdr, rec), 0);

        let end_info = bcf_get_info(hdr, rec, c"END".as_ptr());
        assert!(!end_info.is_null());
        assert_eq!((*end_info).type_, hts_sys::BCF_BT_INT64 as c_int);
        assert_eq!((*end_info).v1.i, 3_000_000_000);
        assert_eq!((*end_info).vptr_len, std::mem::size_of::<i64>() as u32);
        assert!((*end_info).vptr_off() > 0);
        let packed = std::slice::from_raw_parts(
            (*end_info).vptr.sub((*end_info).vptr_off() as usize),
            (*end_info).vptr_off() as usize + (*end_info).vptr_len as usize,
        );
        assert_eq!(&packed[packed.len() - 8..], &3_000_000_000i64.to_le_bytes());

        let mut line = kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        };
        assert_eq!(vcf_format(hdr, rec, &mut line), 0);
        assert_eq!(
            CStr::from_ptr(line.s),
            c"chr1\t1\t.\tA\t<DEL>\t.\t.\tEND=3000000000\n"
        );

        ks_free(&mut line);
        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }

    let _ = std::fs::remove_file(input);
}
