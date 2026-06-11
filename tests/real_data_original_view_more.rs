use htslib_rs::{
    bam_destroy1, bam_init1, bcf_destroy, bcf_hdr_destroy, bcf_hdr_fmt_text, bcf_init,
    htsExactFormat, hts_c_2208_hts_check_EOF, hts_close, hts_get_format, hts_idx_destroy,
    hts_itr_destroy, hts_open, hts_opt_add, hts_opt_apply, hts_opt_free, hts_set_fai_filename,
    hts_set_threads, kstring_t, sam_c_1768_sam_itr_regarray, sam_c_4553_sam_write1, sam_format1,
    sam_hdr_destroy, sam_hdr_length, sam_hdr_read, sam_hdr_str, sam_hdr_write, sam_idx_init,
    sam_idx_save, sam_index_load, sam_itr_next, sam_itr_querys, sam_read1, vcf_format,
    vcf_hdr_read, vcf_read, HTS_FORMAT_BAM, HTS_FORMAT_CRAM, HTS_FORMAT_VCF,
};
use std::ffi::{CStr, CString};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_fixture(path: &str) -> CString {
    CString::new(fixture(path).to_string_lossy().as_bytes()).unwrap()
}

unsafe fn sam_header_text(hdr: *mut htslib_rs::sam_hdr_t) -> String {
    let len = sam_hdr_length(&mut *hdr);
    if len == 0 {
        return String::new();
    }
    assert!(!sam_hdr_str(&mut *hdr).is_null());
    String::from_utf8(std::slice::from_raw_parts(sam_hdr_str(&mut *hdr).cast::<u8>(), len).to_vec())
        .unwrap()
}

unsafe fn append_formatted_record(
    hdr: *mut htslib_rs::sam_hdr_t,
    rec: *const htslib_rs::bam1_t,
    line: &mut kstring_t,
    out: &mut String,
) {
    line.data.clear();
    assert!(sam_format1(hdr, rec, line) >= 0);
    out.push_str(&String::from_utf8_lossy(&line.data));
    out.push('\n');
}

fn temp_output_path(name: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "htslib_rs-{name}-{}-{nanos}.sam",
        std::process::id()
    ))
}

unsafe fn render_alignment_through_sam_writer(path: &str) -> String {
    render_alignment_path_through_sam_writer(&fixture(path), None)
}

unsafe fn render_alignment_path_through_sam_writer(
    path: &std::path::Path,
    reference: Option<&str>,
) -> String {
    let in_path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
    let in_fp = hts_open(in_path.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {}", path.display());
    if let Some(reference) = reference {
        let reference_c = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(in_fp, reference_c.as_ptr()), 0);
    }
    let hdr = sam_hdr_read(in_fp);
    assert!(
        !hdr.is_null(),
        "failed to read SAM header from {}",
        path.display()
    );

    let out_path = temp_output_path("view");
    let out_path_c = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
    let out_fp = hts_open(out_path_c.as_ptr(), c"w".as_ptr());
    assert!(!out_fp.is_null(), "failed to open temp SAM output");
    assert_eq!(sam_hdr_write(out_fp, hdr), 0);

    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert!(sam_c_4553_sam_write1(out_fp, hdr, rec) >= 0);
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);
    let out = std::fs::read_to_string(&out_path).unwrap();
    let _ = std::fs::remove_file(out_path);
    out
}

unsafe fn copy_alignment_to_cram_with_index(
    input_path: &str,
    output_name: &str,
    output_opts: &[&str],
) -> std::path::PathBuf {
    let in_path = c_fixture(input_path);
    let in_fp = hts_open(in_path.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {input_path}");
    let hdr = sam_hdr_read(in_fp);
    assert!(
        !hdr.is_null(),
        "failed to read SAM header from {input_path}"
    );

    let out_path = temp_output_path(output_name).with_extension("cram");
    let out_index_path = out_path.with_extension("cram.crai");
    let out_path_c = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
    let out_index_path_c = CString::new(out_index_path.to_string_lossy().as_bytes()).unwrap();
    let out_fp = hts_open(out_path_c.as_ptr(), c"wc".as_ptr());
    assert!(!out_fp.is_null(), "failed to open temp CRAM output");

    let mut opts: *mut htslib_rs::hts_opt = std::ptr::null_mut();
    for opt in output_opts {
        let opt_c = CString::new(*opt).unwrap();
        assert_eq!(
            hts_opt_add(&mut opts, opt_c.as_ptr()),
            0,
            "bad option {opt}"
        );
    }
    assert_eq!(hts_opt_apply(out_fp, opts), 0);
    hts_opt_free(opts);

    assert_eq!(sam_hdr_write(out_fp, hdr), 0);
    assert_eq!(sam_idx_init(out_fp, hdr, 0, out_index_path_c.as_ptr()), 0);

    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            assert_eq!(ret, -1);
            break;
        }
        assert!(sam_c_4553_sam_write1(out_fp, hdr, rec) >= 0);
    }

    assert_eq!(sam_idx_save(out_fp), 0);
    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);
    out_path
}

unsafe fn copy_alignment_to_cram(
    input_path: &str,
    output_name: &str,
    output_opts: &[&str],
    reference: Option<&str>,
) -> std::path::PathBuf {
    let in_path = c_fixture(input_path);
    let in_fp = hts_open(in_path.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {input_path}");
    let hdr = sam_hdr_read(in_fp);
    assert!(
        !hdr.is_null(),
        "failed to read SAM header from {input_path}"
    );
    if output_opts.is_empty() && reference.is_none() {
        let max_ref_ends = alignment_max_ref_ends(input_path);
        extend_header_reference_lengths(hdr, &max_ref_ends);
    }

    let out_path = temp_output_path(output_name).with_extension("cram");
    let out_path_c = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
    let out_fp = hts_open(out_path_c.as_ptr(), c"wc".as_ptr());
    assert!(!out_fp.is_null(), "failed to open temp CRAM output");
    if let Some(reference) = reference {
        let reference_c = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(out_fp, reference_c.as_ptr()), 0);
    }

    let mut opts: *mut htslib_rs::hts_opt = std::ptr::null_mut();
    for opt in output_opts {
        let opt_c = CString::new(*opt).unwrap();
        assert_eq!(
            hts_opt_add(&mut opts, opt_c.as_ptr()),
            0,
            "bad option {opt}"
        );
    }
    assert_eq!(hts_opt_apply(out_fp, opts), 0);
    hts_opt_free(opts);

    assert_eq!(sam_hdr_write(out_fp, hdr), 0);

    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            assert_eq!(ret, -1);
            break;
        }
        assert!(sam_c_4553_sam_write1(out_fp, hdr, rec) >= 0);
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);
    out_path
}

unsafe fn alignment_max_ref_ends(input_path: &str) -> Vec<i64> {
    let in_path = c_fixture(input_path);
    let in_fp = hts_open(in_path.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {input_path}");
    let hdr = sam_hdr_read(in_fp);
    assert!(
        !hdr.is_null(),
        "failed to read SAM header from {input_path}"
    );

    let n_targets = (*hdr).n_targets.max(0) as usize;
    let mut max_ref_ends = vec![0i64; n_targets];
    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            assert_eq!(ret, -1);
            break;
        }
        let tid = (*rec).core.tid;
        if tid >= 0 && (tid as usize) < max_ref_ends.len() {
            let pos = (*rec).core.pos;
            let cigar = htslib_rs::bam_get_cigar(rec);
            let rlen = htslib_rs::bam_cigar2rlen((*rec).core.n_cigar as i32, cigar);
            if rlen > 0 {
                max_ref_ends[tid as usize] = max_ref_ends[tid as usize].max(pos + rlen);
            }
        }
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    max_ref_ends
}

unsafe fn extend_header_reference_lengths(hdr: *mut htslib_rs::sam_hdr_t, max_ref_ends: &[i64]) {
    let mut changed = false;
    for (tid, &end) in max_ref_ends.iter().enumerate() {
        if end <= 0 || tid >= (*hdr).n_targets as usize {
            continue;
        }
        if !(*hdr).target_len.is_null() {
            let target_len = (*hdr).target_len.add(tid);
            if (*target_len as i64) < end {
                *target_len = end as u32;
                changed = true;
            }
        }
        if !(*hdr).hrecs.is_null() && (tid as i32) < (*(*hdr).hrecs).nref {
            let ref_ = (*(*hdr).hrecs).ref_.add(tid);
            if (*ref_).len < end {
                (*ref_).len = end;
                changed = true;
            }
        }
    }
    if changed {
        rewrite_header_text_reference_lengths(hdr);
    }
}

unsafe fn rewrite_header_text_reference_lengths(hdr: *mut htslib_rs::sam_hdr_t) {
    if (*hdr).text.is_null() {
        return;
    }

    let text = std::slice::from_raw_parts((*hdr).text.cast::<u8>(), (*hdr).l_text);
    let text = String::from_utf8(text.to_vec()).unwrap();
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        if !line.starts_with("@SQ\t") {
            out.push_str(line);
            continue;
        }

        let has_newline = line.ends_with('\n');
        let bare = line.trim_end_matches('\n');
        let mut fields = bare.split('\t').map(str::to_owned).collect::<Vec<_>>();
        let Some(sn) = fields
            .iter()
            .find_map(|field| field.strip_prefix("SN:").map(str::to_owned))
        else {
            out.push_str(line);
            continue;
        };
        let mut tid = None;
        for i in 0..(*hdr).n_targets {
            let name = CStr::from_ptr(*(*hdr).target_name.add(i as usize)).to_string_lossy();
            if name == sn {
                tid = Some(i as usize);
                break;
            }
        }
        let Some(tid) = tid else {
            out.push_str(line);
            continue;
        };

        let len = *(*hdr).target_len.add(tid);
        for field in &mut fields {
            if field.starts_with("LN:") {
                *field = format!("LN:{len}");
            }
        }
        out.push_str(&fields.join("\t"));
        if has_newline {
            out.push('\n');
        }
    }

    let bytes = out.into_bytes();
    let new_text = libc::malloc(bytes.len() + 1).cast::<u8>();
    assert!(!new_text.is_null());
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), new_text, bytes.len());
    *new_text.add(bytes.len()) = 0;
    libc::free((*hdr).text.cast());
    (*hdr).text = new_text.cast();
    (*hdr).l_text = bytes.len();
}

fn compare_sam_record_key(line: &str) -> Vec<String> {
    let mut fields = line
        .trim_end_matches(['\r', '\n'])
        .split('\t')
        .collect::<Vec<_>>();
    assert!(fields.len() >= 11, "malformed SAM record: {line}");

    let flag = fields[1].parse::<u32>().unwrap();
    if flag & 4 != 0 {
        fields[4] = "0";
        fields[5] = "*";
    }

    let mut key = fields
        .iter()
        .take(11)
        .map(|field| (*field).to_owned())
        .collect::<Vec<_>>();
    let mut aux = fields
        .iter()
        .skip(11)
        .map(|field| {
            let bytes = field.as_bytes();
            if bytes.len() > 5 && bytes[2] == b':' && bytes[3] == b'f' && bytes[4] == b':' {
                let value = &field[5..];
                format!("{}{}", &field[..5], value.parse::<f64>().unwrap())
            } else {
                (*field).to_owned()
            }
        })
        .collect::<Vec<_>>();
    aux.sort();
    key.extend(aux);

    key[9] = key[9].to_ascii_uppercase().replace('U', "T");
    key[5] = compare_sam_cigar_key(&key[5]);
    key
}

fn compare_sam_cigar_key(cigar: &str) -> String {
    if cigar == "*" {
        return "*".to_owned();
    }

    let mut ops = Vec::new();
    let mut n = String::new();
    for ch in cigar.chars() {
        if ch.is_ascii_digit() {
            n.push(ch);
        } else {
            let len = n.parse::<u64>().unwrap();
            if len != 0 {
                if let Some((last_len, last_op)) = ops.last_mut() {
                    if *last_op == ch {
                        *last_len += len;
                    } else {
                        ops.push((len, ch));
                    }
                } else {
                    ops.push((len, ch));
                }
            }
            n.clear();
        }
    }

    if ops.is_empty() {
        "*".to_owned()
    } else {
        ops.into_iter()
            .map(|(len, op)| format!("{len}{op}"))
            .collect::<String>()
    }
}

fn assert_compare_sam_records(expected: &str, actual: &str, label: &str) {
    let expected_records = expected.lines().filter(|line| !line.starts_with('@'));
    let actual_records = actual.lines().filter(|line| !line.starts_with('@'));
    let expected_keys = expected_records
        .map(compare_sam_record_key)
        .collect::<Vec<_>>();
    let mut actual_keys = actual_records
        .map(compare_sam_record_key)
        .collect::<Vec<_>>();
    for (expected, actual) in expected_keys.iter().zip(actual_keys.iter_mut()) {
        if expected[9] == "*" {
            actual[9] = "*".to_owned();
        }
    }
    assert_eq!(actual_keys, expected_keys, "{label}");
}

unsafe fn render_alignment_regions(
    path: &str,
    reference: Option<&str>,
    regions: &[&str],
) -> (String, htsExactFormat) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {path}");
    if let Some(reference) = reference {
        let reference_c = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference_c.as_ptr()), 0);
    }

    let format = (*hts_get_format(fp)).format;
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null(), "failed to read SAM header from {path}");
    let mut out = sam_header_text(hdr);

    let idx = sam_index_load(fp, path_c.as_ptr());
    assert!(!idx.is_null(), "failed to load index for {path}");
    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut line: kstring_t = kstring_t::default();

    for region in regions {
        let region_c = CString::new(*region).unwrap();
        let iter = sam_itr_querys(idx, hdr, region_c.as_ptr());
        assert!(!iter.is_null(), "failed to query {path}:{region}");
        loop {
            let ret = sam_itr_next(fp, iter, rec);
            if ret < 0 {
                assert_eq!(ret, -1, "iterator error for {path}:{region}");
                break;
            }
            append_formatted_record(hdr, rec, &mut line, &mut out);
        }
        hts_itr_destroy(iter);
    }

    bam_destroy1(rec);
    hts_idx_destroy(idx);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    (out, format)
}

unsafe fn render_alignment_multiregion(
    path: &str,
    reference: Option<&str>,
    regions: &[&str],
) -> (String, htsExactFormat) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {path}");
    if let Some(reference) = reference {
        let reference_c = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference_c.as_ptr()), 0);
    }

    let format = (*hts_get_format(fp)).format;
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null(), "failed to read SAM header from {path}");

    let out_path = temp_output_path("multiregion");
    let out_path_c = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
    let out_fp = hts_open(out_path_c.as_ptr(), c"w".as_ptr());
    assert!(!out_fp.is_null(), "failed to open temp SAM output");
    assert_eq!(sam_hdr_write(out_fp, hdr), 0);

    let idx = sam_index_load(fp, path_c.as_ptr());
    assert!(!idx.is_null(), "failed to load index for {path}");
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut region_cstrings: Vec<CString> = regions
        .iter()
        .map(|region| CString::new(*region).unwrap())
        .collect();
    let mut region_ptrs: Vec<*mut libc::c_char> = region_cstrings
        .iter_mut()
        .map(|region| region.as_ptr().cast_mut())
        .collect();
    let iter =
        sam_c_1768_sam_itr_regarray(idx, hdr, region_ptrs.as_mut_ptr(), region_ptrs.len() as u32);
    assert!(!iter.is_null(), "failed to multi-query {path}");
    loop {
        let ret = sam_itr_next(fp, iter, rec);
        if ret < 0 {
            assert_eq!(ret, -1, "multi iterator error for {path}");
            break;
        }
        assert!(sam_c_4553_sam_write1(out_fp, hdr, rec) >= 0);
    }

    hts_itr_destroy(iter);
    bam_destroy1(rec);
    hts_idx_destroy(idx);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    assert_eq!(hts_close(out_fp), 0);
    let out = std::fs::read_to_string(&out_path).unwrap();
    let _ = std::fs::remove_file(out_path);
    (out, format)
}

unsafe fn render_vcf(path: &str) -> (String, htsExactFormat) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {path}");
    let format = (*hts_get_format(fp)).format;

    let hdr = vcf_hdr_read(fp);
    assert!(!hdr.is_null(), "failed to read VCF header from {path}");

    let mut len = 0;
    let header_text = bcf_hdr_fmt_text(hdr, 0, &mut len);
    assert!(!header_text.is_null());
    let mut out = CStr::from_ptr(header_text).to_string_lossy().into_owned();
    assert_eq!(out.len(), len as usize);
    libc::free(header_text.cast());

    let rec = bcf_init();
    assert!(!rec.is_null());
    let mut line: kstring_t = kstring_t::default();
    loop {
        let ret = vcf_read(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        line.data.clear();
        assert_eq!(vcf_format(hdr, rec, &mut line), 0);
        out.push_str(&String::from_utf8_lossy(&line.data));
    }

    bcf_destroy(rec);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    (out, format)
}

#[test]
fn original_view_range_bam_regions_match_expected_output() {
    unsafe {
        let regions = [
            "CHROMOSOME_II:2980-2980",
            "CHROMOSOME_IV:1500-1500",
            "CHROMOSOME_II:2980-2980",
            "CHROMOSOME_I:1000-1100",
        ];
        let (actual, format) = render_alignment_regions("htslib/test/range.bam", None, &regions);
        assert_eq!(format, HTS_FORMAT_BAM);
        assert_eq!(
            actual,
            std::fs::read_to_string(fixture("htslib/test/range.out")).unwrap()
        );
    }
}

#[test]
fn original_view_no_hdr_sq_bam_writer_synthesizes_expected_sq_lines() {
    unsafe {
        let actual = render_alignment_through_sam_writer("htslib/test/no_hdr_sq_1.bam");
        assert_eq!(
            actual,
            std::fs::read_to_string(fixture("htslib/test/no_hdr_sq_1.expected.sam")).unwrap()
        );
    }
}

#[test]
fn original_view_sam_hdr_read_caches_bam_and_cram_headers_on_file() {
    unsafe {
        for (path, reference) in [
            ("htslib/test/range.bam", None),
            ("htslib/test/range.cram", Some("htslib/test/ce.fa")),
        ] {
            let path_c = c_fixture(path);
            let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null(), "failed to open {path}");
            if let Some(reference) = reference {
                let reference_c = c_fixture(reference);
                assert_eq!(hts_set_fai_filename(fp, reference_c.as_ptr()), 0);
            }

            let hdr = sam_hdr_read(fp);
            assert!(!hdr.is_null(), "failed to read header from {path}");
            assert_eq!((*fp).bam_header, hdr.cast());

            sam_hdr_destroy(hdr);
            assert_eq!(hts_close(fp), 0);
        }
    }
}

#[test]
fn original_view_plain_sam_header_write_is_flushed_before_close() {
    unsafe {
        let in_path = c_fixture("htslib/test/range.bam");
        let in_fp = hts_open(in_path.as_ptr(), c"r".as_ptr());
        assert!(!in_fp.is_null());
        let hdr = sam_hdr_read(in_fp);
        assert!(!hdr.is_null());

        let out_path = temp_output_path("hdr-flush");
        let out_path_c = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
        let out_fp = hts_open(out_path_c.as_ptr(), c"w".as_ptr());
        assert!(!out_fp.is_null());
        assert_eq!(sam_hdr_write(out_fp, hdr), 0);

        let visible = std::fs::read_to_string(&out_path).unwrap();
        assert!(visible.starts_with("@HD\tVN:1.4\tSO:coordinate\n"));
        assert!(visible.contains("@SQ\tSN:CHROMOSOME_I\tLN:1009800\t"));

        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(in_fp), 0);
        assert_eq!(hts_close(out_fp), 0);
        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn original_view_range_cram_regions_restore_md_nm_with_reference() {
    unsafe {
        let regions = [
            "CHROMOSOME_II:2980-2980",
            "CHROMOSOME_IV:1500-1500",
            "CHROMOSOME_II:2980-2980",
            "CHROMOSOME_I:1000-1100",
        ];
        let (actual, format) = render_alignment_regions(
            "htslib/test/range.cram",
            Some("htslib/test/ce.fa"),
            &regions,
        );
        assert_eq!(format, HTS_FORMAT_CRAM);
        assert_eq!(
            actual,
            std::fs::read_to_string(fixture("htslib/test/range.out")).unwrap()
        );
    }
}

#[test]
fn original_view_range_bam_and_cram_multiregions_match_expected_output2() {
    unsafe {
        let regions = [
            "CHROMOSOME_I:1122-1122",
            "CHROMOSOME_II:1136-1136",
            "CHROMOSOME_II:1241-1241",
            "CHROMOSOME_II:1267-1267",
            "CHROMOSOME_II:1326-1326",
            "CHROMOSOME_II:1345-1345",
            "CHROMOSOME_II:1353-1353",
            "CHROMOSOME_II:1366-1366",
            "CHROMOSOME_II:1416-1416",
            "CHROMOSOME_II:1459-1459",
            "CHROMOSOME_II:1536-1536",
        ];
        let expected = std::fs::read_to_string(fixture("htslib/test/range.out2")).unwrap();

        let (actual_bam, bam_format) =
            render_alignment_multiregion("htslib/test/range.bam", None, &regions);
        assert_eq!(bam_format, HTS_FORMAT_BAM);
        assert_compare_sam_records(&expected, &actual_bam, "range.bam -M range.out2");

        let (actual_cram, cram_format) = render_alignment_multiregion(
            "htslib/test/range.cram",
            Some("htslib/test/ce.fa"),
            &regions,
        );
        assert_eq!(cram_format, HTS_FORMAT_CRAM);
        assert_compare_sam_records(&expected, &actual_cram, "range.cram -M range.out2");
    }
}

#[test]
fn original_view_sam_md_tags_survive_parse_and_format_exactly() {
    unsafe {
        for path in [
            "htslib/test/embed_MD.sam",
            "htslib/test/xx#MD.sam",
            "htslib/test/xx#MD2.sam",
        ] {
            let actual = render_alignment_through_sam_writer(path);
            assert_eq!(actual, std::fs::read_to_string(fixture(path)).unwrap());
        }
    }
}

#[test]
fn original_view_index_dos_sam_roundtrip_normalizes_crlf_to_lf() {
    unsafe {
        let actual = render_alignment_through_sam_writer("htslib/test/index_dos.sam");
        let expected = std::fs::read_to_string(fixture("htslib/test/index_dos.sam"))
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(actual, expected);
    }
}

#[test]
fn original_view_cram_embed_ref2_round_trips_like_compare_sam() {
    unsafe {
        for (sam_path, output_name, opts, reference) in [
            (
                "htslib/test/ce#1000.sam",
                "ce-1000-embed-ref2",
                Vec::<&str>::new(),
                None,
            ),
            (
                "htslib/test/c1#bounds.sam",
                "c1-bounds-embed-ref2",
                vec!["no_ref=1"],
                None,
            ),
            (
                "htslib/test/embed_MD.sam",
                "embed-md-embed-ref2",
                vec!["no_ref=1", "seqs_per_slice=3", "bases_per_slice=1000000"],
                None,
            ),
            (
                "htslib/test/xx#pair.sam",
                "xx-pair-embed-ref2-v4",
                vec!["version=4.0", "embed_ref=2"],
                Some("htslib/test/xx.fa"),
            ),
        ] {
            let cram_path = copy_alignment_to_cram(sam_path, output_name, &opts, reference);
            let actual = render_alignment_path_through_sam_writer(&cram_path, None);
            let expected = std::fs::read_to_string(fixture(sam_path)).unwrap();
            assert_compare_sam_records(&expected, &actual, sam_path);
            let _ = std::fs::remove_file(cram_path);
        }
    }
}

#[test]
fn original_view_index3_cram_region_matches_expected_container_skipping_output() {
    unsafe {
        let cram_path = copy_alignment_to_cram_with_index(
            "htslib/test/index3.sam",
            "index3-container-skip",
            &["seqs_per_slice=2", "embed_ref=2"],
        );
        let cram_path_c = CString::new(cram_path.to_string_lossy().as_bytes()).unwrap();
        let fp = hts_open(cram_path_c.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open generated index3 CRAM");
        let hdr = sam_hdr_read(fp);
        assert!(
            !hdr.is_null(),
            "failed to read generated index3 CRAM header"
        );
        let mut actual = sam_header_text(hdr);

        let idx = sam_index_load(fp, cram_path_c.as_ptr());
        assert!(!idx.is_null(), "failed to load generated index3 CRAI");
        let rec = bam_init1();
        assert!(!rec.is_null());
        let mut line: kstring_t = kstring_t::default();
        let region = CString::new("CHROMOSOME_I:5000-5100").unwrap();
        let iter = sam_itr_querys(idx, hdr, region.as_ptr());
        assert!(!iter.is_null(), "failed to query generated index3 CRAM");
        loop {
            let ret = sam_itr_next(fp, iter, rec);
            if ret < 0 {
                assert_eq!(ret, -1, "iterator error for generated index3 CRAM");
                break;
            }
            append_formatted_record(hdr, rec, &mut line, &mut actual);
        }

        hts_itr_destroy(iter);
        bam_destroy1(rec);
        hts_idx_destroy(idx);
        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);

        assert_eq!(
            actual,
            std::fs::read_to_string(fixture("htslib/test/index3_exp.sam")).unwrap()
        );

        let _ = std::fs::remove_file(cram_path.with_extension("cram.crai"));
        let _ = std::fs::remove_file(cram_path);
    }
}

#[test]
fn original_view_index3_cram_multiregion_matches_expected_container_skipping_output() {
    unsafe {
        let cram_path = copy_alignment_to_cram_with_index(
            "htslib/test/index3.sam",
            "index3-container-skip-multi",
            &["seqs_per_slice=2", "embed_ref=2"],
        );
        let cram_path_c = CString::new(cram_path.to_string_lossy().as_bytes()).unwrap();
        let fp = hts_open(cram_path_c.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open generated index3 CRAM");
        let hdr = sam_hdr_read(fp);
        assert!(
            !hdr.is_null(),
            "failed to read generated index3 CRAM header"
        );
        let mut actual = sam_header_text(hdr);

        let idx = sam_index_load(fp, cram_path_c.as_ptr());
        assert!(!idx.is_null(), "failed to load generated index3 CRAI");
        let rec = bam_init1();
        assert!(!rec.is_null());
        let mut line: kstring_t = kstring_t::default();
        let region = CString::new("CHROMOSOME_I:5000-5100").unwrap();
        let mut regions = [region.as_ptr().cast_mut()];
        let iter = sam_c_1768_sam_itr_regarray(idx, hdr, regions.as_mut_ptr(), 1);
        assert!(
            !iter.is_null(),
            "failed to multi-query generated index3 CRAM"
        );
        loop {
            let ret = sam_itr_next(fp, iter, rec);
            if ret < 0 {
                assert_eq!(ret, -1, "multi iterator error for generated index3 CRAM");
                break;
            }
            append_formatted_record(hdr, rec, &mut line, &mut actual);
        }

        hts_itr_destroy(iter);
        bam_destroy1(rec);
        hts_idx_destroy(idx);
        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);

        assert_eq!(
            actual,
            std::fs::read_to_string(fixture("htslib/test/index3_exp.sam")).unwrap()
        );

        let _ = std::fs::remove_file(cram_path.with_extension("cram.crai"));
        let _ = std::fs::remove_file(cram_path);
    }
}

#[test]
fn original_view_index3_cram_eof_and_threaded_decode_match_original_records() {
    unsafe {
        let cram_path = copy_alignment_to_cram_with_index(
            "htslib/test/index3.sam",
            "index3-threaded-eof",
            &["seqs_per_slice=2", "embed_ref=2"],
        );
        let cram_path_c = CString::new(cram_path.to_string_lossy().as_bytes()).unwrap();
        let fp = hts_open(cram_path_c.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open generated index3 CRAM");
        assert_eq!(hts_set_threads(fp, 2), 0);
        assert_eq!(hts_c_2208_hts_check_EOF(fp), 1);

        let hdr = sam_hdr_read(fp);
        assert!(
            !hdr.is_null(),
            "failed to read generated index3 CRAM header"
        );
        let mut actual = sam_header_text(hdr);

        let rec = bam_init1();
        assert!(!rec.is_null());
        let mut line: kstring_t = kstring_t::default();
        loop {
            let ret = sam_read1(fp, hdr, rec);
            if ret < 0 {
                assert_eq!(ret, -1, "read error for generated index3 CRAM");
                break;
            }
            append_formatted_record(hdr, rec, &mut line, &mut actual);
        }

        bam_destroy1(rec);
        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);

        let expected = std::fs::read_to_string(fixture("htslib/test/index3.sam")).unwrap();
        assert_compare_sam_records(&expected, &actual, "threaded index3 CRAM linear decode");

        let _ = std::fs::remove_file(cram_path.with_extension("cram.crai"));
        let _ = std::fs::remove_file(cram_path);
    }
}

#[test]
fn original_view_test_vcf_api_output_renders_exact_text_again() {
    unsafe {
        let (actual, format) = render_vcf("htslib/test/test-vcf-api.out");
        assert_eq!(format, HTS_FORMAT_VCF);
        assert_eq!(
            actual,
            std::fs::read_to_string(fixture("htslib/test/test-vcf-api.out")).unwrap()
        );
    }
}
