use htslib_rs::{
    bam_aux2f, bam_aux_get, bam_cigar_op, bam_cigar_oplen, bam_destroy1, bam_endpos, bam_get_cigar,
    bam_get_qname, bam_init1, fai_build3, fai_destroy, fai_fetch, fai_fetchqual, fai_load3_format,
    faidx_has_seq, faidx_iseq, faidx_nseq, faidx_seq_len, hclose, hopen, htsFormat,
    hts_detect_format2, regidx_c_246_regidx_init, regidx_c_311_regidx_destroy,
    regidx_c_401_regidx_overlap, regidx_c_584_regitr_init, regidx_c_606_regitr_destroy,
    regidx_c_612_regitr_overlap, regidx_c_91_regidx_seq_nregs, regidx_c_98_regidx_nregs,
    sam_c_4553_sam_write1, sam_hdr_destroy, sam_hdr_nref, sam_hdr_read, sam_hdr_tid2len,
    sam_hdr_tid2name, sam_hdr_write, sam_index_load2, sam_read1, BAM_CMATCH, BAM_FREAD1,
    BAM_FREAD2, FAI_FASTA, FAI_FASTQ, HTS_COMPRESSION_NO_COMPRESSION, HTS_FORMAT_BED,
    HTS_FORMAT_FASTA_FORMAT, HTS_FORMAT_FASTQ_FORMAT, HTS_FORMAT_REGION_LIST, HTS_FORMAT_SAM,
    HTS_FORMAT_SEQUENCE_DATA,
};
use std::ffi::{CStr, CString};
use std::path::PathBuf;

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn c_path(path: PathBuf) -> CString {
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn temp_index_path(label: &str, suffix: &str) -> CString {
    let path = std::env::temp_dir().join(format!(
        "htslib_rs-real-data-samples-more-{}-{}{}",
        std::process::id(),
        label,
        suffix
    ));
    c_path(path)
}

unsafe fn argv_from_cstrings(args: &mut [CString]) -> Vec<*mut std::ffi::c_char> {
    args.iter_mut().map(|arg| arg.as_ptr().cast_mut()).collect()
}

unsafe fn run_sample_stdout(
    args: &mut [CString],
    out_path: &std::path::Path,
    main_fn: unsafe fn(i32, *mut *mut std::ffi::c_char) -> i32,
) -> i32 {
    run_sample_output(args, out_path, false, main_fn)
}

unsafe fn run_sample_output(
    args: &mut [CString],
    out_path: &std::path::Path,
    capture_stderr: bool,
    main_fn: unsafe fn(i32, *mut *mut std::ffi::c_char) -> i32,
) -> i32 {
    let _ = std::fs::remove_file(out_path);
    let pid = libc::fork();
    assert!(pid >= 0, "fork failed");
    if pid == 0 {
        let out_c = c_path(out_path.to_path_buf());
        let out_fd = libc::open(
            out_c.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            0o600,
        );
        if out_fd < 0 {
            libc::_exit(libc::EXIT_FAILURE);
        }
        libc::dup2(out_fd, libc::STDOUT_FILENO);
        if capture_stderr {
            libc::dup2(out_fd, libc::STDERR_FILENO);
        }
        libc::close(out_fd);
        if !capture_stderr {
            let null_fd = libc::open(c"/dev/null".as_ptr(), libc::O_WRONLY);
            if null_fd >= 0 {
                libc::dup2(null_fd, libc::STDERR_FILENO);
                libc::close(null_fd);
            }
        }
        let mut argv = argv_from_cstrings(args);
        let ret = main_fn(argv.len() as i32, argv.as_mut_ptr());
        libc::fflush(std::ptr::null_mut());
        libc::_exit(ret);
    }

    let mut status = 0;
    assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
    assert!(libc::WIFEXITED(status));
    libc::WEXITSTATUS(status)
}

unsafe fn count_sam_records_and_read_flags(path: &std::path::Path) -> (usize, usize, usize) {
    let c_path = c_path(path.to_path_buf());
    let fp = htslib_rs::hts_open(c_path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.display());

    let hdr = sam_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read header from {}",
        path.display()
    );

    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut records = 0;
    let mut read1 = 0;
    let mut read2 = 0;
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        records += 1;
        let flag = (*rec).core.flag as i32;
        if flag & BAM_FREAD1 != 0 {
            read1 += 1;
        }
        if flag & BAM_FREAD2 != 0 {
            read2 += 1;
        }
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(htslib_rs::hts_close(fp), 0);
    (records, read1, read2)
}

unsafe fn count_cram_records_with_reference(
    path: &std::path::Path,
    reference_path: &std::path::Path,
) -> usize {
    let cram_c = c_path(path.to_path_buf());
    let c_reference = c_path(reference_path.to_path_buf());
    let fp = htslib_rs::hts_open(cram_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.display());
    assert_eq!(htslib_rs::hts_set_fai_filename(fp, c_reference.as_ptr()), 0);

    let hdr = sam_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read header from {}",
        path.display()
    );

    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut records = 0;
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        records += 1;
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(htslib_rs::hts_close(fp), 0);
    records
}

unsafe fn write_bam_copy(input_path: &std::path::Path, output_path: &std::path::Path) {
    let input_c = c_path(input_path.to_path_buf());
    let output_c = c_path(output_path.to_path_buf());
    let in_fp = htslib_rs::hts_open(input_c.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {}", input_path.display());
    let out_fp = htslib_rs::hts_open(output_c.as_ptr(), c"wb".as_ptr());
    assert!(
        !out_fp.is_null(),
        "failed to create {}",
        output_path.display()
    );

    let hdr = sam_hdr_read(in_fp);
    assert!(!hdr.is_null());
    assert!(sam_hdr_write(out_fp, hdr) >= 0);

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
    assert_eq!(htslib_rs::hts_close(in_fp), 0);
    assert_eq!(htslib_rs::hts_close(out_fp), 0);
}

unsafe fn collect_qtask_ordered_xr(path: &std::path::Path) -> Vec<(String, f64)> {
    let c_path = c_path(path.to_path_buf());
    let fp = htslib_rs::hts_open(c_path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.display());

    let hdr = sam_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read header from {}",
        path.display()
    );

    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut values = Vec::new();
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        let aux = bam_aux_get(rec, c"xr".as_ptr());
        assert!(
            !aux.is_null(),
            "missing xr aux tag on {}",
            CStr::from_ptr(bam_get_qname(rec)).to_string_lossy()
        );
        let ratio = bam_aux2f(aux);
        assert!(ratio.is_finite());
        assert!((0.0..=1.0).contains(&ratio));
        values.push((
            CStr::from_ptr(bam_get_qname(rec))
                .to_string_lossy()
                .into_owned(),
            ratio,
        ));
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(htslib_rs::hts_close(fp), 0);
    values
}

fn detect_fixture_format(path: &str) -> htsFormat {
    unsafe {
        let path = c_fixture(path);
        let fp = hopen(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

        let mut fmt: htsFormat = std::mem::zeroed();
        assert_eq!(hts_detect_format2(fp, path.as_ptr(), &mut fmt), 0);
        assert_eq!(hclose(fp), 0);
        fmt
    }
}

unsafe fn fetch_text(fai: *const htslib_rs::faidx_t, region: &CStr) -> String {
    let mut len = 0;
    let seq = fai_fetch(fai, region.as_ptr(), &mut len);
    assert!(!seq.is_null());
    let text = CStr::from_ptr(seq).to_string_lossy().into_owned();
    assert_eq!(text.len(), len as usize);
    libc::free(seq.cast());
    text
}

unsafe fn fetch_qual_text(fai: *const htslib_rs::faidx_t, region: &CStr) -> String {
    let mut len = 0;
    let qual = fai_fetchqual(fai, region.as_ptr(), &mut len);
    assert!(!qual.is_null());
    let text = CStr::from_ptr(qual).to_string_lossy().into_owned();
    assert_eq!(text.len(), len as usize);
    libc::free(qual.cast());
    text
}

#[test]
fn detects_demo_sample_fixture_formats() {
    let sam = detect_fixture_format("htslib/samples/sample.sam");
    assert_eq!(sam.category, HTS_FORMAT_SEQUENCE_DATA);
    assert_eq!(sam.format, HTS_FORMAT_SAM);
    assert_eq!(sam.compression, HTS_COMPRESSION_NO_COMPRESSION);

    let fasta = detect_fixture_format("htslib/samples/sample.ref.fa");
    assert_eq!(fasta.category, HTS_FORMAT_SEQUENCE_DATA);
    assert_eq!(fasta.format, HTS_FORMAT_FASTA_FORMAT);
    assert_eq!(fasta.compression, HTS_COMPRESSION_NO_COMPRESSION);

    let fastq = detect_fixture_format("htslib/samples/sample.ref.fq");
    assert_eq!(fastq.category, HTS_FORMAT_SEQUENCE_DATA);
    assert_eq!(fastq.format, HTS_FORMAT_FASTQ_FORMAT);
    assert_eq!(fastq.compression, HTS_COMPRESSION_NO_COMPRESSION);

    let bed = detect_fixture_format("htslib/samples/sample.bed");
    assert_eq!(bed.category, HTS_FORMAT_REGION_LIST);
    assert_eq!(bed.format, HTS_FORMAT_BED);
    assert_eq!(bed.compression, HTS_COMPRESSION_NO_COMPRESSION);
}

#[test]
fn reads_demo_sample_sam_header_and_exact_records() {
    unsafe {
        let path = c_fixture("htslib/samples/sample.sam");
        let fp = htslib_rs::hts_open(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(sam_hdr_nref(&*hdr), 2);
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 0)), c"T1");
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 1)), c"T2");
        assert_eq!(sam_hdr_tid2len(&*hdr, 0), 40);
        assert_eq!(sam_hdr_tid2len(&*hdr, 1), 40);

        let rec = bam_init1();
        assert!(!rec.is_null());

        assert!(sam_read1(fp, hdr, rec) >= 0);
        assert_eq!(CStr::from_ptr(bam_get_qname(rec)), c"ITR1");
        assert_eq!((*rec).core.flag, 99);
        assert_eq!((*rec).core.tid, 0);
        assert_eq!((*rec).core.pos, 4);
        assert_eq!((*rec).core.qual, 40);
        assert_eq!((*rec).core.l_qseq, 4);
        assert_eq!(bam_endpos(rec), 8);
        assert_eq!((*rec).core.n_cigar, 1);
        assert_eq!(bam_cigar_op(*bam_get_cigar(rec)), BAM_CMATCH);
        assert_eq!(bam_cigar_oplen(*bam_get_cigar(rec)), 4);

        let mut names = vec!["ITR1".to_string()];
        loop {
            let ret = sam_read1(fp, hdr, rec);
            if ret < 0 {
                break;
            }
            names.push(
                CStr::from_ptr(bam_get_qname(rec))
                    .to_string_lossy()
                    .into_owned(),
            );
        }

        assert_eq!(names.len(), 16);
        assert_eq!(names[1], "ITR2");
        assert_eq!(names[2], "ITR2M");
        assert_eq!(names[3], "ITR1M");
        assert_eq!(names[15], "B5");

        bam_destroy1(rec);
        sam_hdr_destroy(hdr);
        assert_eq!(htslib_rs::hts_close(fp), 0);
    }
}

#[test]
fn indexes_and_fetches_demo_sample_fasta_sequences() {
    unsafe {
        let fasta = c_fixture("htslib/samples/sample.ref.fa");
        let fai_path = temp_index_path("sample-ref-fa", ".fai");
        assert_eq!(
            fai_build3(fasta.as_ptr(), fai_path.as_ptr(), std::ptr::null()),
            0
        );

        let fai = fai_load3_format(
            fasta.as_ptr(),
            fai_path.as_ptr(),
            std::ptr::null(),
            0,
            FAI_FASTA,
        );
        assert!(!fai.is_null());

        assert_eq!(faidx_nseq(fai), 2);
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)), c"T1");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)), c"T2");
        assert_eq!(faidx_has_seq(fai, c"T1".as_ptr()), 1);
        assert_eq!(faidx_has_seq(fai, c"T3".as_ptr()), 0);
        assert_eq!(faidx_seq_len(fai, c"T1".as_ptr()), 40);
        assert_eq!(faidx_seq_len(fai, c"T2".as_ptr()), 40);

        assert_eq!(fetch_text(fai, c"T1:1-12"), "AAAAACTGAAAA");
        assert_eq!(fetch_text(fai, c"T1:33-40"), "CAGTTTTT");
        assert_eq!(fetch_text(fai, c"T2:1-12"), "TTTTCCCCACTG");
        assert_eq!(fetch_text(fai, c"T2:29-40"), "ACTGTTAACAGT");

        fai_destroy(fai);
    }
}

#[test]
fn indexes_and_fetches_demo_sample_fastq_sequences_and_qualities() {
    unsafe {
        let fastq = c_fixture("htslib/samples/sample.ref.fq");
        let fqi_path = temp_index_path("sample-ref-fq", ".fqi");
        assert_eq!(
            fai_build3(fastq.as_ptr(), fqi_path.as_ptr(), std::ptr::null()),
            0
        );

        let fai = fai_load3_format(
            fastq.as_ptr(),
            fqi_path.as_ptr(),
            std::ptr::null(),
            0,
            FAI_FASTQ,
        );
        assert!(!fai.is_null());

        assert_eq!(faidx_nseq(fai), 4);
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)), c"T1");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)), c"T2");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 2)), c"T3");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 3)), c"T4");
        assert_eq!(faidx_seq_len(fai, c"T1".as_ptr()), 40);
        assert_eq!(faidx_seq_len(fai, c"T3".as_ptr()), 20);
        assert_eq!(faidx_seq_len(fai, c"T4".as_ptr()), 100);

        assert_eq!(fetch_text(fai, c"T1:1-12"), "AAAAACTGAAAA");
        assert_eq!(fetch_qual_text(fai, c"T1:1-12"), "AAAAACTGAAAA");
        assert_eq!(fetch_text(fai, c"T3:1-20"), "TTTTGGGGACTGTTAACAGT");
        assert_eq!(fetch_qual_text(fai, c"T3:1-20"), "TTTTGGGGACTGTTAACAGT");
        assert_eq!(fetch_text(fai, c"T4:81-100"), "TTTTGGGGACTGTTAACAGT");

        fai_destroy(fai);
    }
}

#[test]
fn read_fast_sample_command_prints_demo_fasta_and_fastq_payloads() {
    unsafe {
        let base =
            std::env::temp_dir().join(format!("htslib_rs-read-fast-sample-{}", std::process::id()));
        let fasta_out = base.with_extension("fa.out");
        let fastq_out = base.with_extension("fq.out");

        let mut fasta_args = [
            CString::new("read_fast").unwrap(),
            c_fixture("htslib/samples/sample.ref.fa"),
        ];
        assert_eq!(
            run_sample_stdout(
                &mut fasta_args,
                &fasta_out,
                htslib_rs::samples::read_fast::samples_read_fast_c_49_main,
            ),
            libc::EXIT_SUCCESS
        );
        assert_eq!(
            std::fs::read_to_string(&fasta_out).unwrap(),
            "\nname: T1\nsequence: AAAAACTGAAAACCCCTTTTGGGGACTGTTAACAGTTTTT\
\nname: T2\nsequence: TTTTCCCCACTGAAAACCCCTTTTGGGGACTGTTAACAGT\n"
        );

        let mut fastq_args = [
            CString::new("read_fast").unwrap(),
            c_fixture("htslib/samples/sample.ref.fq"),
        ];
        assert_eq!(
            run_sample_stdout(
                &mut fastq_args,
                &fastq_out,
                htslib_rs::samples::read_fast::samples_read_fast_c_49_main,
            ),
            libc::EXIT_SUCCESS
        );
        let fastq_text = std::fs::read_to_string(&fastq_out).unwrap();
        assert!(fastq_text.starts_with(
            "\nname: T1\nsequence: AAAAACTGAAAACCCCTTTTGGGGACTGTTAACAGTTTTT\
\nquality: AAAAACTGAAAACCCCTTTTGGGGACTGTTAACAGTTTTT"
        ));
        assert!(fastq_text.contains(
            "\nname: T4\nsequence: TTTTCCCCACTGAAAACCCCTTTTGGGGACTGTTAACAGTTTTTCCCCACTGAAAACCCCTTTTGGGGACTGTTAACAGTTTTTGGGGACTGTTAACAGT\
\nquality: TTTTCCCCACTGAAAACCCCTTTTGGGGACTGTTAACAGTTTTTCCCCACTGAAAACCCCTTTTGGGGACTGTTAACAGTTTTTGGGGACTGTTAACAGT\n"
        ));

        let _ = std::fs::remove_file(fasta_out);
        let _ = std::fs::remove_file(fastq_out);
    }
}

#[test]
fn flags_htsopt_field_sample_counts_demo_read_pair_flags() {
    unsafe {
        let out_path = std::env::temp_dir().join(format!(
            "htslib_rs-flags-htsopt-field-sample-{}.out",
            std::process::id()
        ));
        let sample = c_fixture("htslib/samples/sample.sam");
        let sample_text = sample.to_string_lossy().into_owned();
        let mut args = [
            CString::new("flags_field").unwrap(),
            CString::new(sample.as_bytes()).unwrap(),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_path,
                htslib_rs::samples::flags_htsopt_field::samples_flags_htsopt_field_c_50_main,
            ),
            libc::EXIT_SUCCESS
        );
        assert_eq!(
            std::fs::read_to_string(&out_path).unwrap(),
            format!("File {sample_text} has 10 read1 and 6 read2 alignments\n")
        );

        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn add_header_sample_appends_expected_demo_header_lines() {
    unsafe {
        let out_path = std::env::temp_dir().join(format!(
            "htslib_rs-add-header-sample-{}.sam",
            std::process::id()
        ));
        let sample = c_fixture("htslib/samples/sample.sam");
        let sample_text = sample.to_string_lossy().into_owned();
        let mut args = [
            CString::new("add_header").unwrap(),
            CString::new(sample.as_bytes()).unwrap(),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_path,
                htslib_rs::samples::add_header::samples_add_header_c_49_main,
            ),
            libc::EXIT_SUCCESS
        );

        let output = std::fs::read_to_string(&out_path).unwrap();
        assert!(output.starts_with("@HD\tVN:1.17\tSO:unknown\n"));
        assert!(output.contains("@SQ\tSN:T1\tLN:40\n@SQ\tSN:T2\tLN:40\n"));
        assert!(output.contains("@SQ\tSN:TR1\tLN:100\n@SQ\tSN:TR2\tLN:50\n"));
        assert!(output.contains("@RG\tID:RG1\tLB:Test\tSM:S1\n"));
        assert!(output.contains("@CO\tTest data\n"));
        assert!(output.ends_with(&format!(
            "@PG\tID:add_header\tPN:add_header\tVN:Test\tCL:add_header {sample_text} \n"
        )));

        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn update_header_sample_rewrites_demo_sq_length() {
    unsafe {
        let out_path = std::env::temp_dir().join(format!(
            "htslib_rs-update-header-sample-{}.sam",
            std::process::id()
        ));
        let mut args = [
            CString::new("update_header").unwrap(),
            c_fixture("htslib/samples/sample.sam"),
            CString::new("SQ").unwrap(),
            CString::new("T1").unwrap(),
            CString::new("LN").unwrap(),
            CString::new("38").unwrap(),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_path,
                htslib_rs::samples::update_header::samples_update_header_c_49_main,
            ),
            libc::EXIT_SUCCESS
        );

        let output = std::fs::read_to_string(&out_path).unwrap();
        assert!(output.starts_with("@HD\tVN:1.17\tSO:unknown\n"));
        assert!(output.contains("@SQ\tSN:T1\tLN:38\n"));
        assert!(output.contains("@SQ\tSN:T2\tLN:40\n"));
        assert!(!output.contains("@SQ\tSN:T1\tLN:40\n"));
        assert!(!output.lines().any(|line| !line.starts_with('@')));

        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn qtask_unordered_sample_counts_demo_bases_with_small_chunks() {
    unsafe {
        let out_path = std::env::temp_dir().join(format!(
            "htslib_rs-qtask-unordered-sample-{}.out",
            std::process::id()
        ));
        let mut args = [
            CString::new("qtask_unordered").unwrap(),
            c_fixture("htslib/samples/sample.sam"),
            CString::new("2").unwrap(),
            CString::new("3").unwrap(),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_path,
                htslib_rs::samples::qtask_unordered::samples_qtask_unordered_c_181_main,
            ),
            libc::EXIT_SUCCESS
        );
        assert_eq!(
            std::fs::read_to_string(&out_path).unwrap(),
            "GCratio: 0.413793\nBase counts:\n\
=: 0\n\
A: 15\n\
C: 6\n\
M: 0\n\
G: 18\n\
R: 0\n\
S: 0\n\
V: 0\n\
T: 19\n\
W: 0\n\
Y: 0\n\
H: 0\n\
K: 0\n\
D: 0\n\
B: 0\n\
N: 0\n"
        );

        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn qtask_ordered_sample_writes_ordered_bam_with_gc_ratio_aux_tags() {
    unsafe {
        let base = std::env::temp_dir().join(format!(
            "htslib_rs-qtask-ordered-sample-{}",
            std::process::id()
        ));
        let out_capture = base.with_extension("out");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let in_bam = base.with_extension("bam");
        write_bam_copy(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/samples/sample.sam"),
            &in_bam,
        );

        let mut args = [
            CString::new("qtask_ordered").unwrap(),
            c_path(in_bam.clone()),
            CString::new("2").unwrap(),
            c_path(base.clone()),
            CString::new("3").unwrap(),
        ];

        assert_eq!(
            run_sample_output(
                &mut args,
                &out_capture,
                true,
                htslib_rs::samples::qtask_ordered::samples_qtask_ordered_c_223_main,
            ),
            libc::EXIT_SUCCESS
        );

        let out_bam = base.join("out.bam");
        assert!(out_bam.exists());
        assert_eq!(count_sam_records_and_read_flags(&out_bam), (16, 10, 6));

        let xr_values = collect_qtask_ordered_xr(&out_bam);
        assert_eq!(xr_values.len(), 16);
        assert_eq!(
            xr_values.first().map(|(name, _)| name.as_str()),
            Some("ITR1")
        );
        assert_eq!(xr_values.last().map(|(name, _)| name.as_str()), Some("B5"));
        assert!(xr_values.iter().any(|(_, ratio)| *ratio > 0.0));

        let _ = std::fs::remove_file(out_capture);
        let _ = std::fs::remove_file(in_bam);
        let _ = std::fs::remove_dir_all(base);
    }
}

#[test]
fn split2_sample_writes_read1_and_read2_outputs() {
    unsafe {
        let base =
            std::env::temp_dir().join(format!("htslib_rs-split2-sample-{}", std::process::id()));
        let out_capture = base.with_extension("out");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        let mut args = [
            CString::new("split2").unwrap(),
            c_fixture("htslib/samples/sample.sam"),
            c_path(base.clone()),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_capture,
                htslib_rs::samples::split2::samples_split2_c_50_main,
            ),
            libc::EXIT_SUCCESS
        );

        let read1_path = base.join("1.sam.gz");
        let read2_path = base.join("2.sam");
        assert!(read1_path.exists());
        assert!(read2_path.exists());
        assert_eq!(count_sam_records_and_read_flags(&read1_path), (10, 10, 0));
        assert_eq!(count_sam_records_and_read_flags(&read2_path), (6, 0, 6));

        let _ = std::fs::remove_file(out_capture);
        let _ = std::fs::remove_dir_all(base);
    }
}

#[test]
fn split_sample_writes_read1_sam_and_read2_bam_outputs() {
    unsafe {
        let base =
            std::env::temp_dir().join(format!("htslib_rs-split-sample-{}", std::process::id()));
        let out_capture = base.with_extension("out");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        let mut args = [
            CString::new("split").unwrap(),
            c_fixture("htslib/samples/sample.sam"),
            c_path(base.clone()),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_capture,
                htslib_rs::samples::split::samples_split_c_50_main,
            ),
            libc::EXIT_SUCCESS
        );

        let read1_path = base.join("1.sam");
        let read2_path = base.join("2.bam");
        assert!(read1_path.exists());
        assert!(read2_path.exists());
        assert_eq!(count_sam_records_and_read_flags(&read1_path), (10, 10, 0));
        assert_eq!(count_sam_records_and_read_flags(&read2_path), (6, 0, 6));

        let _ = std::fs::remove_file(out_capture);
        let _ = std::fs::remove_dir_all(base);
    }
}

#[test]
fn split_thread1_sample_writes_threaded_read1_sam_and_read2_bam_outputs() {
    unsafe {
        let base = std::env::temp_dir().join(format!(
            "htslib_rs-split-thread1-sample-{}",
            std::process::id()
        ));
        let out_capture = base.with_extension("out");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        let mut args = [
            CString::new("split_t1").unwrap(),
            c_fixture("htslib/samples/sample.sam"),
            c_path(base.clone()),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_capture,
                htslib_rs::samples::split_thread1::samples_split_thread1_c_50_main,
            ),
            libc::EXIT_SUCCESS
        );

        let read1_path = base.join("1.sam");
        let read2_path = base.join("2.bam");
        assert!(read1_path.exists());
        assert!(read2_path.exists());
        assert_eq!(count_sam_records_and_read_flags(&read1_path), (10, 10, 0));
        assert_eq!(count_sam_records_and_read_flags(&read2_path), (6, 0, 6));

        let _ = std::fs::remove_file(out_capture);
        let _ = std::fs::remove_dir_all(base);
    }
}

#[test]
fn split_thread2_sample_writes_thread_pool_read1_sam_and_read2_bam_outputs() {
    unsafe {
        let base = std::env::temp_dir().join(format!(
            "htslib_rs-split-thread2-sample-{}",
            std::process::id()
        ));
        let out_capture = base.with_extension("out");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        let mut args = [
            CString::new("split_t2").unwrap(),
            c_fixture("htslib/samples/sample.sam"),
            c_path(base.clone()),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_capture,
                htslib_rs::samples::split_thread2::samples_split_thread2_c_51_main,
            ),
            libc::EXIT_SUCCESS
        );

        let read1_path = base.join("1.sam");
        let read2_path = base.join("2.bam");
        assert!(read1_path.exists());
        assert!(read2_path.exists());
        assert_eq!(count_sam_records_and_read_flags(&read1_path), (10, 10, 0));
        assert_eq!(count_sam_records_and_read_flags(&read2_path), (6, 0, 6));

        let _ = std::fs::remove_file(out_capture);
        let _ = std::fs::remove_dir_all(base);
    }
}

#[test]
fn cram_sample_writes_all_reference_modes_as_readable_cram() {
    unsafe {
        let base =
            std::env::temp_dir().join(format!("htslib_rs-cram-sample-{}", std::process::id()));
        let out_capture = base.with_extension("out");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        let reference = base.join("sample.ref.fa");
        std::fs::copy(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/samples/sample.ref.fa"),
            &reference,
        )
        .unwrap();
        let reference_c = c_path(reference.clone());
        assert_eq!(
            fai_build3(reference_c.as_ptr(), std::ptr::null(), std::ptr::null()),
            0
        );

        let mut args = [
            CString::new("cram").unwrap(),
            c_fixture("htslib/samples/sample.sam"),
            reference_c,
            c_path(base.clone()),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_capture,
                htslib_rs::samples::cram::samples_cram_c_53_main,
            ),
            libc::EXIT_SUCCESS
        );

        for name in ["1.cram", "2.cram", "3.cram", "4.cram"] {
            let cram_path = base.join(name);
            assert!(cram_path.exists(), "missing {}", cram_path.display());
            assert_eq!(
                count_cram_records_with_reference(&cram_path, &reference),
                16
            );
        }

        let _ = std::fs::remove_file(out_capture);
        let _ = std::fs::remove_dir_all(base);
    }
}

#[test]
fn index_write_sample_creates_compressed_sam_and_loadable_indexes() {
    unsafe {
        for (shift, suffix) in [("0", "bai"), ("14", "csi")] {
            let base = std::env::temp_dir().join(format!(
                "htslib_rs-index-write-sample-{}-{}",
                std::process::id(),
                suffix
            ));
            let out_capture = base.with_extension("out");
            let _ = std::fs::remove_dir_all(&base);
            std::fs::create_dir_all(&base).unwrap();

            let mut args = [
                CString::new("idx_on_write").unwrap(),
                c_fixture("htslib/test/index.sam"),
                CString::new(shift).unwrap(),
                c_path(base.clone()),
            ];

            assert_eq!(
                run_sample_stdout(
                    &mut args,
                    &out_capture,
                    htslib_rs::samples::index_write::samples_index_write_c_50_main,
                ),
                libc::EXIT_SUCCESS
            );

            let out_path = base.join("index.sam.gz");
            let index_path = base.join(format!("index.sam.gz.{suffix}"));
            assert!(out_path.exists(), "missing {}", out_path.display());
            assert!(index_path.exists(), "missing {}", index_path.display());
            assert_eq!(count_sam_records_and_read_flags(&out_path), (181, 0, 0));

            let out_c = c_path(out_path);
            let index_c = c_path(index_path);
            let fp = htslib_rs::hts_open(out_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let idx = sam_index_load2(fp, out_c.as_ptr(), index_c.as_ptr());
            assert!(!idx.is_null(), "failed to load {suffix} index");
            htslib_rs::hts_idx_destroy(idx);
            assert_eq!(htslib_rs::hts_close(fp), 0);

            let _ = std::fs::remove_file(out_capture);
            let _ = std::fs::remove_dir_all(base);
        }
    }
}

#[test]
fn read_fast_index_sample_command_fetches_single_and_multi_regions() {
    unsafe {
        let base = std::env::temp_dir().join(format!(
            "htslib_rs-read-fast-index-sample-{}",
            std::process::id()
        ));
        let fq_path = base.with_extension("fq");
        let out_path = base.with_extension("out");
        std::fs::copy(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/samples/sample.ref.fq"),
            &fq_path,
        )
        .unwrap();

        let mut single_args = [
            CString::new("read_fast_i").unwrap(),
            c_path(fq_path.clone()),
            CString::new("Q").unwrap(),
            CString::new("0").unwrap(),
            CString::new("T1:6-13").unwrap(),
        ];
        assert_eq!(
            run_sample_stdout(
                &mut single_args,
                &out_path,
                htslib_rs::samples::read_fast_index::samples_read_fast_index_c_53_main,
            ),
            libc::EXIT_SUCCESS
        );
        assert_eq!(
            std::fs::read_to_string(&out_path).unwrap(),
            "Data: 8 CTGAAAAC\nQual: 8 CTGAAAAC\n"
        );

        let mut multi_args = [
            CString::new("read_fast_i").unwrap(),
            c_path(fq_path.clone()),
            CString::new("Q").unwrap(),
            CString::new("1").unwrap(),
            CString::new("T2:1-4,T3:17-20,T4:97-100").unwrap(),
        ];
        assert_eq!(
            run_sample_stdout(
                &mut multi_args,
                &out_path,
                htslib_rs::samples::read_fast_index::samples_read_fast_index_c_53_main,
            ),
            libc::EXIT_SUCCESS
        );
        assert_eq!(
            std::fs::read_to_string(&out_path).unwrap(),
            "Data: 5 TTTTC\nQual: 5 TTTTC\nData: 4 CAGT\nQual: 4 CAGT\nData: 4 CAGT\nQual: 4 CAGT\n"
        );

        let _ = std::fs::remove_file(&fq_path);
        let _ = std::fs::remove_file(fq_path.with_extension("fq.fai"));
        let _ = std::fs::remove_file(fq_path.with_extension("fq.fqi"));
        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn write_fast_sample_command_appends_fasta_and_fastq_records() {
    unsafe {
        let base = std::env::temp_dir().join(format!(
            "htslib_rs-write-fast-sample-{}",
            std::process::id()
        ));
        let fa_path = base.with_extension("fa");
        let fq_path = base.with_extension("fq");
        let out_path = base.with_extension("out");
        let _ = std::fs::remove_file(&fa_path);
        let _ = std::fs::remove_file(&fq_path);

        let mut fa_args = [
            CString::new("write_fast").unwrap(),
            c_path(fa_path.clone()),
            CString::new("ACTGN").unwrap(),
        ];
        assert_eq!(
            run_sample_stdout(
                &mut fa_args,
                &out_path,
                htslib_rs::samples::write_fast::samples_write_fast_c_51_main,
            ),
            libc::EXIT_SUCCESS
        );
        let fa = std::fs::read_to_string(&fa_path).unwrap();
        assert!(fa.starts_with(">Test_"));
        assert!(fa.ends_with("\nACTGN\n"));
        assert!(!fa.contains("\n+\n"));

        let mut fq_args = [
            CString::new("write_fast").unwrap(),
            c_path(fq_path.clone()),
            CString::new("ACTGN").unwrap(),
            CString::new("ABCDE").unwrap(),
        ];
        assert_eq!(
            run_sample_stdout(
                &mut fq_args,
                &out_path,
                htslib_rs::samples::write_fast::samples_write_fast_c_51_main,
            ),
            libc::EXIT_SUCCESS
        );
        let fq = std::fs::read_to_string(&fq_path).unwrap();
        assert!(fq.starts_with("@Test_"));
        assert!(fq.ends_with("\nACTGN\n+\nbcdef\n"));

        let _ = std::fs::remove_file(fa_path);
        let _ = std::fs::remove_file(fq_path);
        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn parses_demo_sample_bed_with_regidx_and_queries_overlaps() {
    unsafe {
        let path = c_fixture("htslib/samples/sample.bed");
        let mut idx = regidx_c_246_regidx_init(Some(path.as_bytes()), None, None, 0, None)
            .expect("regidx_init failed");

        assert_eq!(regidx_c_98_regidx_nregs(&idx), 4);
        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"T1"), 2);
        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"T2"), 2);
        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"T3"), 0);

        let mut itr = regidx_c_584_regitr_init(&mut idx);

        assert_eq!(
            regidx_c_401_regidx_overlap(&mut idx, b"T1", 1, 1, Some(&mut itr)),
            1
        );
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 1);
        assert_eq!(itr.beg, 1);
        assert_eq!(itr.end, 1);
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 0);

        assert_eq!(
            regidx_c_401_regidx_overlap(&mut idx, b"T1", 30, 34, Some(&mut itr)),
            1
        );
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 1);
        assert_eq!(itr.beg, 30);
        assert_eq!(itr.end, 34);

        assert_eq!(
            regidx_c_401_regidx_overlap(&mut idx, b"T2", 39, 39, Some(&mut itr)),
            1
        );
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 1);
        assert_eq!(itr.beg, 30);
        assert_eq!(itr.end, 39);

        assert_eq!(
            regidx_c_401_regidx_overlap(&mut idx, b"T2", 41, 41, Some(&mut itr)),
            0
        );
        assert_eq!(
            regidx_c_401_regidx_overlap(&mut idx, b"T3", 0, 10, Some(&mut itr)),
            0
        );

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }
}

#[test]
fn index_multireg_sample_emits_expected_records_and_preserves_status_bug() {
    unsafe {
        let out_path = std::env::temp_dir().join(format!(
            "htslib_rs-index-multireg-read-sample-{}.sam",
            std::process::id()
        ));
        let mut args = [
            CString::new("read_multireg").unwrap(),
            c_fixture("htslib/test/range.bam"),
            CString::new("11").unwrap(),
            CString::new(
                "CHROMOSOME_I:1122-1122,\
CHROMOSOME_II:1136-1136,\
CHROMOSOME_II:1241-1241,\
CHROMOSOME_II:1267-1267,\
CHROMOSOME_II:1326-1326,\
CHROMOSOME_II:1345-1345,\
CHROMOSOME_II:1353-1353,\
CHROMOSOME_II:1366-1366,\
CHROMOSOME_II:1416-1416,\
CHROMOSOME_II:1459-1459,\
CHROMOSOME_II:1536-1536",
            )
            .unwrap(),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_path,
                htslib_rs::samples::index_multireg_read::samples_index_multireg_read_c_50_main,
            ),
            libc::EXIT_FAILURE
        );

        let output = std::fs::read_to_string(&out_path).unwrap();
        assert!(output.ends_with("Error during read\n"));
        let records = output
            .strip_suffix("Error during read\n")
            .expect("missing original status-bug diagnostic");
        let expected = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/test/range.out2"),
        )
        .unwrap();
        let expected_records = expected
            .lines()
            .filter(|line| !line.starts_with('@'))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        assert_eq!(records, expected_records);

        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn index_reg_read_sample_emits_expected_range_records() {
    unsafe {
        let out_path = std::env::temp_dir().join(format!(
            "htslib_rs-index-reg-read-sample-{}.sam",
            std::process::id()
        ));
        let mut args = [
            CString::new("read_reg").unwrap(),
            c_fixture("htslib/test/range.bam"),
            c_fixture("htslib/test/range.bam.bai"),
            CString::new("CHROMOSOME_II:2976-3070").unwrap(),
        ];

        assert_eq!(
            run_sample_stdout(
                &mut args,
                &out_path,
                htslib_rs::samples::index_reg_read::samples_index_reg_read_c_55_main,
            ),
            libc::EXIT_SUCCESS
        );

        let output = std::fs::read_to_string(&out_path).unwrap();
        let lines = output.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert!(lines[0]
            .starts_with("HS18_09653:4:2112:13048:11874\t99\tCHROMOSOME_II\t2976\t60\t100M"));
        assert!(
            lines[1].starts_with("HS18_09653:4:2212:4301:59362\t99\tCHROMOSOME_II\t2983\t60\t100M")
        );

        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn mod_bam_cigar_update_preserves_single_aux_payload_copy() {
    unsafe {
        let base =
            std::env::temp_dir().join(format!("htslib_rs-mod-bam-cigar-{}", std::process::id()));
        let in_path = base.with_extension("sam");
        let out_path = base.with_extension("out.sam");
        std::fs::write(
            &in_path,
            b"@HD\tVN:1.6\n@SQ\tSN:T1\tLN:40\nr1\t0\tT1\t1\t60\t4M\t*\t0\t0\tACTG\tIIII\tNM:i:7\n",
        )
        .unwrap();

        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");
        if pid == 0 {
            let out_c = c_path(out_path.clone());
            let out_fd = libc::open(
                out_c.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o600,
            );
            if out_fd < 0 {
                libc::_exit(libc::EXIT_FAILURE);
            }
            libc::dup2(out_fd, libc::STDOUT_FILENO);
            libc::close(out_fd);
            let null_fd = libc::open(c"/dev/null".as_ptr(), libc::O_WRONLY);
            if null_fd >= 0 {
                libc::dup2(null_fd, libc::STDERR_FILENO);
                libc::close(null_fd);
            }
            let mut args = [
                CString::new("mod_bam").unwrap(),
                c_path(in_path.clone()),
                CString::new("r1").unwrap(),
                CString::new("6").unwrap(),
                CString::new("2M2M").unwrap(),
            ];
            let mut argv = argv_from_cstrings(&mut args);
            let ret = htslib_rs::samples::mod_bam::samples_mod_bam_c_50_main(
                argv.len() as i32,
                argv.as_mut_ptr(),
            );
            libc::_exit(ret);
        }

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), libc::EXIT_SUCCESS);

        let out = std::fs::read(&out_path).unwrap();
        assert!(out.windows(b"2M2M".len()).any(|w| w == b"2M2M"));
        assert!(out
            .windows(b"r1\0\0\t0\tT1\t1\t60\t2M2M\t*\t0\t0\tACTG\tIIII\n".len())
            .any(|w| w == b"r1\0\0\t0\tT1\t1\t60\t2M2M\t*\t0\t0\tACTG\tIIII\n"));
        assert!(!out.windows(b"NM:i:7".len()).any(|w| w == b"NM:i:7"));

        let _ = std::fs::remove_file(in_path);
        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn mod_bam_no_matching_qname_streams_original_sample_records() {
    unsafe {
        let out_path = std::env::temp_dir().join(format!(
            "htslib_rs-mod-bam-pass-through-{}.sam",
            std::process::id()
        ));

        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");
        if pid == 0 {
            let out_c = c_path(out_path.clone());
            let out_fd = libc::open(
                out_c.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o600,
            );
            if out_fd < 0 {
                libc::_exit(libc::EXIT_FAILURE);
            }
            libc::dup2(out_fd, libc::STDOUT_FILENO);
            libc::close(out_fd);
            let null_fd = libc::open(c"/dev/null".as_ptr(), libc::O_WRONLY);
            if null_fd >= 0 {
                libc::dup2(null_fd, libc::STDERR_FILENO);
                libc::close(null_fd);
            }
            let mut args = [
                CString::new("mod_bam").unwrap(),
                c_fixture("htslib/samples/sample.sam"),
                CString::new("missing-qname").unwrap(),
                CString::new("5").unwrap(),
                CString::new("17").unwrap(),
            ];
            let mut argv = argv_from_cstrings(&mut args);
            let ret = htslib_rs::samples::mod_bam::samples_mod_bam_c_50_main(
                argv.len() as i32,
                argv.as_mut_ptr(),
            );
            libc::_exit(ret);
        }

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), libc::EXIT_SUCCESS);

        let expected = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/samples/sample.sam"),
        )
        .unwrap();
        let actual = std::fs::read_to_string(&out_path).unwrap();
        assert_eq!(
            actual
                .lines()
                .filter(|line| !line.starts_with('@'))
                .collect::<Vec<_>>(),
            expected
                .lines()
                .filter(|line| !line.starts_with('@'))
                .collect::<Vec<_>>()
        );

        let _ = std::fs::remove_file(out_path);
    }
}

#[test]
fn mod_bam_sample_updates_sequence_and_quality_fields_case_insensitively() {
    unsafe {
        let out_path = std::env::temp_dir().join(format!(
            "htslib_rs-mod-bam-real-data-update-{}.sam",
            std::process::id()
        ));

        let mut seq_args = [
            CString::new("mod_bam").unwrap(),
            c_fixture("htslib/samples/sample.sam"),
            CString::new("a4").unwrap(),
            CString::new("10").unwrap(),
            CString::new("CCT").unwrap(),
        ];
        assert_eq!(
            run_sample_stdout(
                &mut seq_args,
                &out_path,
                htslib_rs::samples::mod_bam::samples_mod_bam_c_50_main,
            ),
            libc::EXIT_SUCCESS
        );
        let seq_output = std::fs::read_to_string(&out_path).unwrap();
        let seq_line = seq_output
            .lines()
            .find(|line| line.starts_with("A4\t"))
            .expect("missing updated A4 record");
        assert_eq!(seq_line, "A4\t99\tT2\t12\t50\t3M\t=\t23\t5\tCCT\t()(");

        let mut qual_args = [
            CString::new("mod_bam").unwrap(),
            c_fixture("htslib/samples/sample.sam"),
            CString::new("a4").unwrap(),
            CString::new("11").unwrap(),
            CString::new("ABC").unwrap(),
        ];
        assert_eq!(
            run_sample_stdout(
                &mut qual_args,
                &out_path,
                htslib_rs::samples::mod_bam::samples_mod_bam_c_50_main,
            ),
            libc::EXIT_SUCCESS
        );
        let qual_output = std::fs::read_to_string(&out_path).unwrap();
        let qual_line = qual_output
            .lines()
            .find(|line| line.starts_with("A4\t"))
            .expect("missing quality-updated A4 record");
        assert_eq!(qual_line, "A4\t99\tT2\t12\t50\t3M\t=\t23\t5\tGAA\tABC");

        let _ = std::fs::remove_file(out_path);
    }
}
