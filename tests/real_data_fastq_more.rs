use htslib_rs::{
    bam1_t, bam_aux2Z, bam_aux_get, bam_destroy1, bam_get_qname, bam_get_qual, bam_get_seq,
    bam_init1, bam_seqi, hts_close, hts_fmt_option, hts_open, hts_set_opt_int, hts_set_opt_ptr,
    sam_hdr_destroy, sam_hdr_read, sam_read1, FASTQ_OPT_AUX, FASTQ_OPT_BARCODE, FASTQ_OPT_CASAVA,
    FASTQ_OPT_NAME2,
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
    let seq = bam_get_seq(rec);
    (0..(*rec).core.l_qseq as usize)
        .map(|i| match bam_seqi(seq, i) {
            1 => 'A',
            2 => 'C',
            4 => 'G',
            8 => 'T',
            15 => 'N',
            base => panic!("unexpected encoded base {base}"),
        })
        .collect()
}

unsafe fn qual_string(rec: *const bam1_t) -> String {
    let qual = bam_get_qual(rec);
    (0..(*rec).core.l_qseq as usize)
        .map(|i| (*qual.add(i) + 33) as char)
        .collect()
}

unsafe fn set_fastq_flag(fp: *mut htslib_rs::htsFile, opt: i32) {
    assert_eq!(hts_set_opt_int(fp, opt as hts_fmt_option, 1), 0);
}

unsafe fn set_fastq_string(fp: *mut htslib_rs::htsFile, opt: i32, value: Option<&CStr>) {
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
    configure: impl FnOnce(*mut htslib_rs::htsFile),
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

#[test]
fn reads_minimal_fastq_record_with_exact_sam_payload() {
    unsafe {
        let actual = read_fastq_records("htslib/test/fastq/minimal.fq", |_| {}, &[]);
        let expected = read_sam_records("htslib/test/fastq/minimal.sam", &[]);

        assert_eq!(actual, expected);
        assert_eq!(
            actual[0],
            FastqRecord {
                qname: "x".to_string(),
                flag: 4,
                seq: "A".to_string(),
                qual: "+".to_string(),
                aux_z: Vec::new(),
            }
        );
    }
}

#[test]
fn reads_single_fastq_aux_tags_and_exact_first_records() {
    unsafe {
        let aux_tags = &[c"RG"];
        let actual = read_fastq_records(
            "htslib/test/fastq/single.fq",
            |fp| set_fastq_string(fp, FASTQ_OPT_AUX, None),
            aux_tags,
        );
        let expected = read_sam_records("htslib/test/fastq/single_aux.sam", aux_tags);

        assert_eq!(&actual[..2], &expected[..2]);
        assert_eq!(actual[0].qname, "HS25_09827:2:1201:1505:59795#49");
        assert_eq!(actual[0].flag, 4);
        assert_eq!(
            actual[0].seq,
            "CCGTTAGAGCATTTGTTGAAAATGCTTTCCTTGCTCCATGTGATGACTCTGGTGCCCTTGTCAAAAGCCAGCTGGGCCTATTCGTGTGGGTCTGTTTCTG"
        );
        assert_eq!(
            actual[0].qual,
            "CABCFGDEEFFEFHGHGGFFGDIGIJFIFHHGHEIFGHBCGHDIFBE9GIAICGGICFIBFGGHGDGGGHE?GIGDFGGHEGIEJG>;FG<GGHACEFGH"
        );
        assert_eq!(actual[0].aux_z, vec![("RG", "1#49".to_string())]);
        assert_eq!(actual[1].qname, "HS25_09827:2:1201:1559:70726#49");
        assert_eq!(actual[1].aux_z, vec![("RG", "1#49".to_string())]);
    }
}

#[test]
fn reads_interleaved_casava_filter_flags_and_barcode_tags() {
    unsafe {
        let aux_tags = &[c"BC"];
        let actual = read_fastq_records(
            "htslib/test/fastq/filter_casava.fq",
            |fp| set_fastq_flag(fp, FASTQ_OPT_CASAVA),
            aux_tags,
        );
        let expected = read_sam_records("htslib/test/fastq/filter_casava.sam", aux_tags);

        assert_eq!(actual, expected);
        assert_eq!(actual[0].qname, "HS25_09827:2:1201:1505:59795#49");
        assert_eq!(actual[0].flag, 77);
        assert_eq!(actual[0].aux_z, vec![("BC", "NGTCTATC".to_string())]);
        assert_eq!(actual[1].flag, 141);
        assert_eq!(actual[2].qname, "HS25_09827:2:1201:1559:70726#49");
        assert_eq!(actual[2].flag, 589);
        assert_eq!(actual[3].flag, 653);
    }
}

#[test]
fn reads_casava_fastq_with_custom_barcode_aux_tag() {
    unsafe {
        let ox = CString::new("OX").unwrap();
        let aux_tags = &[c"OX"];
        let actual = read_fastq_records(
            "htslib/test/fastq/interleaved_casava.fq",
            |fp| {
                set_fastq_string(fp, FASTQ_OPT_BARCODE, Some(ox.as_c_str()));
                set_fastq_flag(fp, FASTQ_OPT_CASAVA);
            },
            aux_tags,
        );
        let expected = read_sam_records("htslib/test/fastq/inter_casavaOX.sam", aux_tags);

        assert_eq!(&actual[..3], &expected[..3]);
        assert_eq!(actual[0].flag, 77);
        assert_eq!(actual[0].aux_z, vec![("OX", "NGTCTATC".to_string())]);
        assert_eq!(actual[1].flag, 141);
        assert_eq!(actual[1].aux_z, vec![("OX", "NGTCTATC".to_string())]);
        assert_eq!(actual[2].qname, "HS25_09827:2:1201:1559:70726#49");
    }
}

#[test]
fn reads_name2_fastq_qnames_exactly_from_expected_sam() {
    unsafe {
        let actual = read_fastq_records(
            "htslib/test/fastq/name2.fq",
            |fp| set_fastq_flag(fp, FASTQ_OPT_NAME2),
            &[],
        );
        let expected = read_sam_records("htslib/test/fastq/name2.sam", &[]);

        assert_eq!(actual, expected);
        assert_eq!(
            actual.iter().map(|r| r.qname.as_str()).collect::<Vec<_>>(),
            vec!["name_001", "name_002", "name_003", "name_004"]
        );
    }
}

#[test]
fn reads_longline_fastq_payload_exactly_from_expected_sam() {
    unsafe {
        let actual = read_fastq_records("htslib/test/fastq/longline.fq", |_| {}, &[]);
        let expected = read_sam_records("htslib/test/fastq/longline.sam", &[]);

        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].qname, "readname");
        assert_eq!(actual[0].flag, 4);
        assert_eq!(actual[0].seq, "ATGC");
        assert_eq!(actual[0].qual, "qqqq");
    }
}
