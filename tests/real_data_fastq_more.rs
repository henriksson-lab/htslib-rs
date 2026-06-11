use htslib_rs::{
    bam1_t, bam_aux2Z, bam_aux_get, bam_destroy1, bam_get_qname, bam_get_qual, bam_get_seq,
    bam_init1, bam_seqi, hts_close, hts_fmt_option, hts_open, hts_set_opt_int, hts_set_opt_ptr,
    sam_hdr_destroy, sam_hdr_read, sam_read1, FASTQ_OPT_AUX, FASTQ_OPT_BARCODE, FASTQ_OPT_CASAVA,
    FASTQ_OPT_NAME2,
};
use std::path::{Path, PathBuf};

fn c_fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut bytes = path.to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

fn c_path(path: &Path) -> Vec<u8> {
    let mut bytes = path.to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

fn optional_external_hifi_fastq() -> Option<PathBuf> {
    let path =
        PathBuf::from("/husky/henriksson/for_claude/minimap/testdata/hg002_hifi_20370reads.fq.gz");
    path.is_file().then_some(path)
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
    unsafe fn from_record(rec: *const bam1_t, aux_tags: &[&'static [u8]]) -> Self {
        Self {
            qname: String::from_utf8_lossy(
                std::ffi::CStr::from_ptr(bam_get_qname(rec).cast()).to_bytes(),
            )
            .into_owned(),
            flag: (*rec).core.flag,
            seq: seq_string(rec),
            qual: qual_string(rec),
            aux_z: aux_tags
                .iter()
                .filter_map(|tag| {
                    let aux = bam_aux_get(rec, tag.as_ptr().cast());
                    if aux.is_null() {
                        None
                    } else {
                        Some((
                            std::str::from_utf8(&tag[..tag.len() - 1]).unwrap(),
                            String::from_utf8_lossy(
                                std::ffi::CStr::from_ptr(bam_aux2Z(aux).cast()).to_bytes(),
                            )
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

unsafe fn set_fastq_string(fp: *mut htslib_rs::htsFile, opt: i32, value: Option<&[u8]>) {
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
    aux_tags: &[&'static [u8]],
) -> Vec<FastqRecord> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr().cast(), b"r\0".as_ptr().cast());
    assert!(
        !fp.is_null(),
        "failed to open {}",
        String::from_utf8_lossy(&path[..path.len() - 1])
    );
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

unsafe fn read_sam_records(path: &str, aux_tags: &[&'static [u8]]) -> Vec<FastqRecord> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr().cast(), b"r\0".as_ptr().cast());
    assert!(
        !fp.is_null(),
        "failed to open {}",
        String::from_utf8_lossy(&path[..path.len() - 1])
    );
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

#[derive(Debug, PartialEq, Eq)]
struct FastqStats {
    records: usize,
    bases: usize,
    min_len: i32,
    max_len: i32,
    qname_samples: Vec<String>,
}

unsafe fn scan_fastq_path(path: &Path) -> FastqStats {
    let path = c_path(path);
    let fp = hts_open(path.as_ptr().cast(), b"r\0".as_ptr().cast());
    assert!(
        !fp.is_null(),
        "failed to open {}",
        String::from_utf8_lossy(&path[..path.len() - 1])
    );
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut stats = FastqStats {
        records: 0,
        bases: 0,
        min_len: i32::MAX,
        max_len: 0,
        qname_samples: Vec::new(),
    };

    loop {
        let ret = sam_read1(fp, std::ptr::null_mut(), rec);
        if ret < 0 {
            break;
        }

        let len = (*rec).core.l_qseq;
        assert!(len > 0);
        assert_eq!((*rec).core.flag, 4);
        assert!(!bam_get_qname(rec).is_null());
        if stats.qname_samples.len() < 3 {
            stats.qname_samples.push(
                String::from_utf8_lossy(
                    std::ffi::CStr::from_ptr(bam_get_qname(rec).cast()).to_bytes(),
                )
                .into_owned(),
            );
        }

        let seq = bam_get_seq(rec);
        let qual = bam_get_qual(rec);
        assert_ne!(*qual, 0xff);
        for i in 0..len as usize {
            assert!(matches!(bam_seqi(seq, i), 1 | 2 | 4 | 8 | 15));
            assert!(
                *qual.add(i) <= 93,
                "FASTQ quality should stay in printable Phred+33 range"
            );
        }

        stats.records += 1;
        stats.bases += len as usize;
        stats.min_len = stats.min_len.min(len);
        stats.max_len = stats.max_len.max(len);
    }

    bam_destroy1(rec);
    assert_eq!(hts_close(fp), 0);
    stats
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
        let aux_tags: &[&'static [u8]] = &[b"RG\0"];
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
        let aux_tags: &[&'static [u8]] = &[b"BC\0"];
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
        let ox = b"OX\0";
        let aux_tags: &[&'static [u8]] = &[b"OX\0"];
        let actual = read_fastq_records(
            "htslib/test/fastq/interleaved_casava.fq",
            |fp| {
                set_fastq_string(fp, FASTQ_OPT_BARCODE, Some(ox));
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
fn external_hifi_fastq_gz_scans_real_read_lengths_and_qualities_when_available() {
    let Some(path) = optional_external_hifi_fastq() else {
        return;
    };
    unsafe {
        let stats = scan_fastq_path(&path);
        assert_eq!(
            stats,
            FastqStats {
                records: 20_370,
                bases: 195_341_223,
                min_len: 1_960,
                max_len: 11_550,
                qname_samples: vec![
                    "m54238_180628_014238/4194606/ccs".to_string(),
                    "m54238_180628_014238/4194841/ccs".to_string(),
                    "m54238_180628_014238/4194870/ccs".to_string(),
                ],
            }
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
