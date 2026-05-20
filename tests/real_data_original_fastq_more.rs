use htslib_mini_rs::{
    bam1_t, bam_aux2Z, bam_aux_get, bam_destroy1, bam_get_qname, bam_get_qual, bam_get_seq,
    bam_init1, bam_seqi, hts_close, hts_fmt_option, hts_open, hts_set_opt_int, hts_set_opt_ptr,
    sam_c_4553_sam_write1, sam_hdr_destroy, sam_hdr_read, sam_read1, FASTQ_OPT_AUX,
    FASTQ_OPT_BARCODE, FASTQ_OPT_CASAVA, FASTQ_OPT_NAME2, FASTQ_OPT_RNUM, FASTQ_OPT_UMI,
};
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

#[derive(Debug, PartialEq, Eq)]
struct FastqRecord {
    qname: String,
    flag: u16,
    seq: String,
    qual: String,
    aux_z: Vec<(&'static str, String)>,
}

impl FastqRecord {
    unsafe fn from_record(rec: *const bam1_t, aux_tags: &[&'static CStr]) -> Self {
        Self {
            qname: CStr::from_ptr(bam_get_qname(rec))
                .to_string_lossy()
                .into_owned(),
            flag: (*rec).core.flag,
            seq: seq_string(rec),
            qual: qual_string(rec),
            aux_z: aux_tags
                .iter()
                .filter_map(|tag| {
                    let aux = bam_aux_get(rec, tag.as_ptr());
                    if aux.is_null() {
                        None
                    } else {
                        Some((
                            tag.to_str().unwrap(),
                            CStr::from_ptr(bam_aux2Z(aux))
                                .to_string_lossy()
                                .into_owned(),
                        ))
                    }
                })
                .collect(),
        }
    }
}

unsafe fn seq_string(rec: *const bam1_t) -> String {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

    let seq = bam_get_seq(rec);
    (0..(*rec).core.l_qseq as usize)
        .map(|i| SEQ_NT16_STR[bam_seqi(seq, i) as usize] as char)
        .collect()
}

unsafe fn qual_string(rec: *const bam1_t) -> String {
    if (*rec).core.l_qseq == 0 {
        return String::new();
    }

    let qual = bam_get_qual(rec);
    if *qual == 0xff {
        return "*".to_string();
    }

    (0..(*rec).core.l_qseq as usize)
        .map(|i| (*qual.add(i) + 33) as char)
        .collect()
}

unsafe fn set_fastq_flag(fp: *mut htslib_mini_rs::htsFile, opt: i32) {
    assert_eq!(hts_set_opt_int(fp, opt as hts_fmt_option, 1), 0);
}

unsafe fn set_fastq_string(fp: *mut htslib_mini_rs::htsFile, opt: i32, value: Option<&CStr>) {
    assert_eq!(
        hts_set_opt_ptr(
            fp,
            opt as hts_fmt_option,
            value.map_or(std::ptr::null_mut(), |s| s.as_ptr().cast_mut().cast())
        ),
        0
    );
}

unsafe fn read_fastq_records(
    path: &str,
    configure: impl FnOnce(*mut htslib_mini_rs::htsFile),
    aux_tags: &[&'static CStr],
) -> Vec<FastqRecord> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    configure(fp);

    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut records = Vec::new();
    loop {
        let ret = sam_read1(fp, std::ptr::null_mut(), rec);
        if ret < 0 {
            break;
        }
        records.push(FastqRecord::from_record(rec, aux_tags));
    }

    bam_destroy1(rec);
    assert_eq!(hts_close(fp), 0);
    records
}

unsafe fn read_sam_records(path: &str, aux_tags: &[&'static CStr]) -> Vec<FastqRecord> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());

    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut records = Vec::new();
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        records.push(FastqRecord::from_record(rec, aux_tags));
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    records
}

unsafe fn assert_read_case(
    input: &str,
    expected_sam: &str,
    configure: impl FnOnce(*mut htslib_mini_rs::htsFile),
    aux_tags: &[&'static CStr],
) -> Vec<FastqRecord> {
    let actual = read_fastq_records(input, configure, aux_tags);
    let expected = read_sam_records(expected_sam, aux_tags);
    assert_eq!(actual, expected, "case {input} -> {expected_sam}");
    actual
}

fn temp_output_path(case: &str, extension: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "htslib-mini-rs-fastq-write-{}-{}-{}.{}",
        std::process::id(),
        case,
        std::thread::current().name().unwrap_or("test"),
        extension
    ))
}

unsafe fn write_sam_as_sequence_file(
    input_sam: &str,
    mode: &'static CStr,
    out_path: &std::path::Path,
    configure: impl FnOnce(*mut htslib_mini_rs::htsFile),
) {
    let input = c_fixture(input_sam);
    let in_fp = hts_open(input.as_ptr(), c"r".as_ptr());
    assert!(
        !in_fp.is_null(),
        "failed to open {}",
        input.to_string_lossy()
    );
    let hdr = sam_hdr_read(in_fp);
    assert!(!hdr.is_null());

    let output = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
    let out_fp = hts_open(output.as_ptr(), mode.as_ptr());
    assert!(
        !out_fp.is_null(),
        "failed to open {}",
        output.to_string_lossy()
    );
    configure(out_fp);

    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert!(
            sam_c_4553_sam_write1(out_fp, hdr, rec) > 0,
            "failed to write record from {input_sam}"
        );
    }

    bam_destroy1(rec);
    assert_eq!(hts_close(out_fp), 0);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
}

unsafe fn assert_write_case(
    input_sam: &str,
    expected_output: &str,
    mode: &'static CStr,
    extension: &str,
    configure: impl FnOnce(*mut htslib_mini_rs::htsFile),
) -> String {
    let case = input_sam
        .rsplit('/')
        .next()
        .unwrap()
        .replace(['#', '.'], "_");
    let out_path = temp_output_path(&case, extension);
    let _ = std::fs::remove_file(&out_path);

    write_sam_as_sequence_file(input_sam, mode, &out_path, configure);

    let actual = std::fs::read_to_string(&out_path).unwrap();
    let expected = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(expected_output),
    )
    .unwrap();
    let _ = std::fs::remove_file(&out_path);
    assert_eq!(actual, expected, "case {input_sam} -> {expected_output}");
    actual
}

#[test]
fn original_fastq_minimal_and_multiline_fasta_inputs_match_expected_sam() {
    unsafe {
        let minimal = assert_read_case(
            "htslib/test/fastq/minimal.fa",
            "htslib/test/fastq/minimal-q.sam",
            |_| {},
            &[],
        );
        assert_eq!(minimal.len(), 1);
        assert_eq!(minimal[0].qname, "x");
        assert_eq!(minimal[0].seq, "A");
        assert_eq!(minimal[0].qual, "*");

        let multiline_fq = assert_read_case(
            "htslib/test/fastq/multiline.fq",
            "htslib/test/fastq/multiline.sam",
            |_| {},
            &[],
        );
        assert_eq!(multiline_fq.len(), 2);
        assert_eq!(
            multiline_fq[0].seq,
            "NAAAAAAAAACCCCCCCCCCGGGGGGGGGGTTTTTTTTN"
        );
        assert_eq!(
            multiline_fq[1].seq,
            "RAAAAAAAAACCCCCCCCCCGGGGGGGGGGTTTTTTTTY"
        );

        let multiline_fa = assert_read_case(
            "htslib/test/fastq/multiline.fa",
            "htslib/test/fastq/multiline-q.sam",
            |_| {},
            &[],
        );
        assert_eq!(multiline_fa[0].qual, "*");
        assert_eq!(multiline_fa[1].qual, "*");
    }
}

#[test]
fn original_fastq_single_file_noaux_and_fasta_aux_cases_match_expected_sam() {
    unsafe {
        assert_read_case(
            "htslib/test/fastq/single.fq",
            "htslib/test/fastq/single_noaux.sam",
            |_| {},
            &[],
        );
        assert_read_case(
            "htslib/test/fastq/single.fa",
            "htslib/test/fastq/single_noaux-q.sam",
            |_| {},
            &[],
        );

        let aux_tags = &[c"RG"];
        let single_fa_aux = assert_read_case(
            "htslib/test/fastq/single.fa",
            "htslib/test/fastq/single_aux-q.sam",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
            aux_tags,
        );
        assert_eq!(single_fa_aux.len(), 5);
        assert_eq!(single_fa_aux[0].aux_z, vec![("RG", "1#49".to_string())]);
        assert_eq!(single_fa_aux[4].qual, "*");
    }
}

#[test]
fn original_fastq_interleaved_noaux_and_aux_cases_match_expected_sam() {
    unsafe {
        assert_read_case(
            "htslib/test/fastq/interleaved.fq",
            "htslib/test/fastq/inter_noaux.sam",
            |_| {},
            &[],
        );
        assert_read_case(
            "htslib/test/fastq/interleaved.fa",
            "htslib/test/fastq/inter_noaux-q.sam",
            |_| {},
            &[],
        );

        let aux_tags = &[c"RG", c"BC", c"QT"];
        let inter_fq_aux = assert_read_case(
            "htslib/test/fastq/interleaved.fq",
            "htslib/test/fastq/inter_aux.sam",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
            aux_tags,
        );
        assert_eq!(inter_fq_aux.len(), 10);
        assert_eq!(
            inter_fq_aux[0].aux_z,
            vec![
                ("RG", "1#49".to_string()),
                ("BC", "NGTCTATC".to_string()),
                ("QT", "!1=BDDDF".to_string()),
            ]
        );
        assert_eq!(inter_fq_aux[1].aux_z, vec![("RG", "1#49".to_string())]);

        let inter_fa_aux = assert_read_case(
            "htslib/test/fastq/interleaved.fa",
            "htslib/test/fastq/inter_aux-q.sam",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
            aux_tags,
        );
        assert_eq!(inter_fa_aux[0].qual, "*");
        assert_eq!(inter_fa_aux[0].aux_z[2], ("QT", "!1=BDDDF".to_string()));
    }
}

#[test]
fn original_fastq_interleaved_casava_fasta_cases_match_expected_sam() {
    unsafe {
        let bc_tags = &[c"BC"];
        let casava_fq = assert_read_case(
            "htslib/test/fastq/interleaved_casava.fq",
            "htslib/test/fastq/inter_casava.sam",
            |fp| set_fastq_flag(fp, FASTQ_OPT_CASAVA),
            bc_tags,
        );
        assert_eq!(casava_fq.len(), 10);
        assert_eq!(casava_fq[0].flag, 77);
        assert_eq!(casava_fq[1].flag, 141);
        assert_eq!(casava_fq[0].aux_z, vec![("BC", "NGTCTATC".to_string())]);

        let casava_fa = assert_read_case(
            "htslib/test/fastq/interleaved_casava.fa",
            "htslib/test/fastq/inter_casava-q.sam",
            |fp| set_fastq_flag(fp, FASTQ_OPT_CASAVA),
            bc_tags,
        );
        assert_eq!(casava_fa[0].qual, "*");
        assert_eq!(casava_fa[0].aux_z, vec![("BC", "NGTCTATC".to_string())]);

        let ox = CString::new("OX").unwrap();
        let ox_tags = &[c"OX"];
        let casava_ox_fa = assert_read_case(
            "htslib/test/fastq/interleaved_casava.fa",
            "htslib/test/fastq/inter_casavaOX-q.sam",
            |fp| {
                set_fastq_string(fp, FASTQ_OPT_BARCODE, Some(ox.as_c_str()));
                set_fastq_flag(fp, FASTQ_OPT_CASAVA);
            },
            ox_tags,
        );
        assert_eq!(casava_ox_fa[0].aux_z, vec![("OX", "NGTCTATC".to_string())]);
    }
}

#[test]
fn original_fastq_filter_casava_fasta_case_matches_expected_sam() {
    unsafe {
        let aux_tags = &[c"BC"];
        let records = assert_read_case(
            "htslib/test/fastq/filter_casava.fa",
            "htslib/test/fastq/filter_casava-q.sam",
            |fp| set_fastq_flag(fp, FASTQ_OPT_CASAVA),
            aux_tags,
        );
        assert_eq!(records.len(), 4);
        assert_eq!(
            records.iter().map(|r| r.flag).collect::<Vec<_>>(),
            vec![77, 141, 589, 653]
        );
        assert_eq!(records[2].qual, "*");
    }
}

#[test]
fn original_fastq_paired_r1_r2_inputs_match_expected_sam() {
    unsafe {
        let aux_tags = &[c"RG", c"BC", c"QT"];
        let r1 = assert_read_case(
            "htslib/test/fastq/r1.fq",
            "htslib/test/fastq/r1.sam",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
            aux_tags,
        );
        assert_eq!(r1.len(), 5);
        assert!(r1.iter().all(|r| r.flag == 77));
        assert_eq!(r1[0].aux_z[1], ("BC", "NGTCTATC".to_string()));

        let r2 = assert_read_case(
            "htslib/test/fastq/r2.fq",
            "htslib/test/fastq/r2.sam",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
            aux_tags,
        );
        assert_eq!(r2.len(), 5);
        assert!(r2.iter().all(|r| r.flag == 141));
        assert_eq!(r2[0].aux_z, vec![("RG", "1#49".to_string())]);

        let r1_fa = assert_read_case(
            "htslib/test/fastq/r1.fa",
            "htslib/test/fastq/r1-q.sam",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
            aux_tags,
        );
        assert_eq!(r1_fa[0].qual, "*");
        assert_eq!(r1_fa[0].aux_z[2], ("QT", "!1=BDDDF".to_string()));

        let r2_fa = assert_read_case(
            "htslib/test/fastq/r2.fa",
            "htslib/test/fastq/r2-q.sam",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
            aux_tags,
        );
        assert_eq!(r2_fa[0].qual, "*");
        assert_eq!(r2_fa[0].aux_z, vec![("RG", "1#49".to_string())]);
    }
}

#[test]
fn original_fastq_name2_fasta_case_matches_expected_sam() {
    unsafe {
        let name2 = assert_read_case(
            "htslib/test/fastq/name2.fa",
            "htslib/test/fastq/name2-q.sam",
            |fp| set_fastq_flag(fp, FASTQ_OPT_NAME2),
            &[],
        );
        assert_eq!(
            name2.iter().map(|r| r.qname.as_str()).collect::<Vec<_>>(),
            vec!["name_001", "name_002", "name_003", "name_004"]
        );
        assert!(name2.iter().all(|r| r.qual == "*"));
    }
}

#[test]
fn original_fastq_umi_barcodes_match_expected_sam() {
    unsafe {
        let umi = assert_read_case(
            "htslib/test/fastq/UMI.fq",
            "htslib/test/fastq/UMI.sam",
            |fp| set_fastq_string(fp, FASTQ_OPT_UMI, Some(c"RX")),
            &[c"RX"],
        );
        assert_eq!(umi.len(), 7);
        assert_eq!(umi[0].qname, "HS25:09827:FID:2:1201:1505:59794");
        assert_eq!(umi[0].aux_z, Vec::<(&str, String)>::new());
        assert_eq!(umi[2].aux_z, vec![("RX", "ACG-TTTTTA".to_string())]);
        assert_eq!(umi[3].flag, 77);
        assert_eq!(umi[4].flag, 141);
        assert_eq!(umi[5].qname, "HS25:09827:FID:2:1201:1505:59798#49");
        assert_eq!(umi[5].aux_z, vec![("RX", "TTA".to_string())]);
    }
}

#[test]
fn original_fastq_write_minimal_and_aux_cases_match_expected_fixtures() {
    unsafe {
        let minimal_fq = assert_write_case(
            "htslib/test/fastq/minimal.sam",
            "htslib/test/fastq/minimal.fq",
            c"wf",
            "fq",
            |_| {},
        );
        assert_eq!(minimal_fq, "@x\nA\n+\n+\n");

        let minimal_fa = assert_write_case(
            "htslib/test/fastq/minimal.sam",
            "htslib/test/fastq/minimal.fa",
            c"wF",
            "fa",
            |_| {},
        );
        assert_eq!(minimal_fa, ">x\nA\n");

        let single_fq = assert_write_case(
            "htslib/test/fastq/single_aux.sam",
            "htslib/test/fastq/single.fq",
            c"wf",
            "fq",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
        );
        assert!(single_fq.contains("\tRG:Z:1#49"));

        let single_fa = assert_write_case(
            "htslib/test/fastq/single_aux.sam",
            "htslib/test/fastq/single.fa",
            c"wF",
            "fa",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
        );
        assert!(single_fa.contains("\tRG:Z:1#49"));
        assert!(!single_fa.contains("\n+\n"));
    }
}

#[test]
fn original_fastq_write_rnum_casava_barcode_and_umi_cases_match_expected_fixtures() {
    unsafe {
        let interleaved = assert_write_case(
            "htslib/test/fastq/inter_aux.sam",
            "htslib/test/fastq/interleaved.fq",
            c"wf",
            "fq",
            |fp| {
                set_fastq_string(fp, FASTQ_OPT_AUX, None);
                set_fastq_flag(fp, FASTQ_OPT_RNUM);
            },
        );
        assert!(interleaved.starts_with("@HS25_09827:2:1201:1505:59795#49/1\t"));
        assert!(interleaved.contains("@HS25_09827:2:1201:1505:59795#49/2\t"));

        let interleaved_fa = assert_write_case(
            "htslib/test/fastq/inter_aux.sam",
            "htslib/test/fastq/interleaved.fa",
            c"wF",
            "fa",
            |fp| {
                set_fastq_string(fp, FASTQ_OPT_AUX, None);
                set_fastq_flag(fp, FASTQ_OPT_RNUM);
            },
        );
        assert!(interleaved_fa.starts_with(">HS25_09827:2:1201:1505:59795#49/1\t"));
        assert!(!interleaved_fa.contains("\n+\n"));

        let casava = assert_write_case(
            "htslib/test/fastq/inter_casava.sam",
            "htslib/test/fastq/interleaved_casava.fq",
            c"wf",
            "fq",
            |fp| set_fastq_flag(fp, FASTQ_OPT_CASAVA),
        );
        assert!(casava.starts_with("@HS25_09827:2:1201:1505:59795#49 1:N:0:NGTCTATC\n"));

        let ox = CString::new("OX").unwrap();
        let casava_ox_fa = assert_write_case(
            "htslib/test/fastq/inter_casavaOX.sam",
            "htslib/test/fastq/interleaved_casava.fa",
            c"wF",
            "fa",
            |fp| {
                set_fastq_string(fp, FASTQ_OPT_BARCODE, Some(ox.as_c_str()));
                set_fastq_flag(fp, FASTQ_OPT_CASAVA);
            },
        );
        assert!(casava_ox_fa.starts_with(">HS25_09827:2:1201:1505:59795#49 1:N:0:NGTCTATC\n"));
        assert!(!casava_ox_fa.contains("\n+\n"));

        let filter_casava = assert_write_case(
            "htslib/test/fastq/filter_casava.sam",
            "htslib/test/fastq/filter_casava.fq",
            c"wf",
            "fq",
            |fp| set_fastq_flag(fp, FASTQ_OPT_CASAVA),
        );
        assert!(filter_casava.contains("@HS25_09827:2:1201:1559:70726#49 1:Y:0:NGTCTATC\n"));

        let umi = assert_write_case(
            "htslib/test/fastq/UMI.sam",
            "htslib/test/fastq/UMI.fq",
            c"wf",
            "fq",
            |fp| {
                set_fastq_flag(fp, FASTQ_OPT_RNUM);
                set_fastq_string(fp, FASTQ_OPT_UMI, None);
            },
        );
        assert!(umi.contains("@HS25:09827:FID:2:1201:1505:59796:ACG+TTTTTA\n"));
        assert!(umi.contains("@HS25:09827:FID:2:1201:1505:59797:TACG+TTTTTA/1\n"));
    }
}
