use htslib_rs::{
    bam1_t, bam_aux2Z, bam_aux2i, bam_aux_get, bam_cigar2qlen, bam_cigar2rlen, bam_cigar_op,
    bam_cigar_oplen, bam_destroy1, bam_get_cigar, bam_get_qname, bam_get_qual, bam_get_seq,
    bam_init1, bam_seqi, hts_close, hts_open, hts_set_opt_ptr, ks_free, kstring_t, sam_format1,
    sam_hdr_destroy, sam_hdr_length, sam_hdr_read, sam_hdr_str, sam_hdr_t, sam_hdr_tid2name,
    sam_read1, BAM_CHARD_CLIP, BAM_CSOFT_CLIP, HTS_OPT_FILTER,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString};

const BASES: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn expected_count(text: &str) -> usize {
    text.trim().parse().unwrap()
}

unsafe fn header_text(hdr: *mut sam_hdr_t) -> String {
    let text = sam_hdr_str(hdr);
    if text.is_null() {
        return String::new();
    }
    String::from_utf8_lossy(std::slice::from_raw_parts(
        text.cast::<u8>(),
        sam_hdr_length(hdr),
    ))
    .into_owned()
}

unsafe fn record_text(hdr: *const sam_hdr_t, rec: *const bam1_t) -> String {
    let mut out: kstring_t = std::mem::zeroed();
    assert!(sam_format1(hdr, rec, &mut out) >= 0);
    let text =
        String::from_utf8_lossy(std::slice::from_raw_parts(out.s.cast::<u8>(), out.l)).into_owned();
    ks_free(&mut out);
    text
}

unsafe fn with_sam_records<R>(
    path: &str,
    mut f: impl FnMut(*mut sam_hdr_t, *mut bam1_t) -> R,
) -> Vec<R> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut values = Vec::new();
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        values.push(f(hdr, rec));
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    values
}

unsafe fn filtered_sam(
    path: &str,
    include_header: bool,
    mut passes: impl FnMut(*mut sam_hdr_t, *mut bam1_t) -> bool,
) -> String {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut out = if include_header {
        header_text(hdr)
    } else {
        String::new()
    };
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        if passes(hdr, rec) {
            out.push_str(&record_text(hdr, rec));
            out.push('\n');
        }
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe fn filtered_sam_with_expression(path: &str, include_header: bool, expr: &CStr) -> String {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    assert_eq!(
        hts_set_opt_ptr(fp, HTS_OPT_FILTER, expr.as_ptr().cast_mut().cast()),
        0
    );
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut out = if include_header {
        header_text(hdr)
    } else {
        String::new()
    };
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        out.push_str(&record_text(hdr, rec));
        out.push('\n');
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe fn filtered_count_with_expression(path: &str, expr: &CStr) -> usize {
    let output = filtered_sam_with_expression(path, false, expr);
    output.lines().filter(|line| !line.starts_with('@')).count()
}

unsafe fn qname(rec: *const bam1_t) -> String {
    CStr::from_ptr(bam_get_qname(rec))
        .to_string_lossy()
        .into_owned()
}

unsafe fn rname(hdr: *const sam_hdr_t, rec: *const bam1_t) -> String {
    let name = sam_hdr_tid2name(hdr, (*rec).core.tid);
    assert!(!name.is_null());
    CStr::from_ptr(name).to_string_lossy().into_owned()
}

unsafe fn cigar_string(rec: *const bam1_t) -> String {
    let cigar = bam_get_cigar(rec);
    let mut out = String::new();
    for i in 0..(*rec).core.n_cigar {
        let op = *cigar.add(i as usize);
        out.push_str(&bam_cigar_oplen(op).to_string());
        out.push(match bam_cigar_op(op) {
            0 => 'M',
            1 => 'I',
            2 => 'D',
            3 => 'N',
            4 => 'S',
            5 => 'H',
            6 => 'P',
            7 => '=',
            8 => 'X',
            9 => 'B',
            other => panic!("unexpected CIGAR op {other}"),
        });
    }
    out
}

unsafe fn seq_string(rec: *const bam1_t) -> String {
    let seq = bam_get_seq(rec);
    let mut out = Vec::with_capacity((*rec).core.l_qseq as usize);
    for i in 0..(*rec).core.l_qseq as usize {
        out.push(BASES[bam_seqi(seq, i) as usize]);
    }
    String::from_utf8(out).unwrap()
}

unsafe fn rg_library_map(hdr: *mut sam_hdr_t) -> HashMap<String, String> {
    header_text(hdr)
        .lines()
        .filter(|line| line.starts_with("@RG\t"))
        .filter_map(|line| {
            let mut id = None;
            let mut library = None;
            for field in line.split('\t').skip(1) {
                if let Some(value) = field.strip_prefix("ID:") {
                    id = Some(value.to_string());
                } else if let Some(value) = field.strip_prefix("LB:") {
                    library = Some(value.to_string());
                }
            }
            Some((id?, library?))
        })
        .collect()
}

unsafe fn record_library(
    libraries: &HashMap<String, String>,
    rec: *const bam1_t,
) -> Option<String> {
    let rg = bam_aux_get(rec, c"RG".as_ptr());
    if rg.is_null() {
        return None;
    }
    let id = CStr::from_ptr(bam_aux2Z(rg)).to_string_lossy();
    libraries.get(id.as_ref()).cloned()
}

unsafe fn aux_i(rec: *const bam1_t, tag: &'static CStr) -> Option<i64> {
    let value = bam_aux_get(rec, tag.as_ptr());
    (!value.is_null()).then(|| bam_aux2i(value))
}

unsafe fn aux_z(rec: *const bam1_t, tag: &'static CStr) -> Option<String> {
    let value = bam_aux_get(rec, tag.as_ptr());
    (!value.is_null()).then(|| {
        CStr::from_ptr(bam_aux2Z(value))
            .to_string_lossy()
            .into_owned()
    })
}

unsafe fn min_max_avg_qual(rec: *const bam1_t) -> (u8, u8, f64) {
    let qual = bam_get_qual(rec);
    let len = (*rec).core.l_qseq as usize;
    let values = std::slice::from_raw_parts(qual, len);
    let min = *values.iter().min().unwrap();
    let max = *values.iter().max().unwrap();
    let avg = values.iter().map(|&q| q as usize).sum::<usize>() as f64 / len as f64;
    (min, max, avg)
}

unsafe fn clipped_len(rec: *const bam1_t, op: i32) -> i64 {
    let cigar = bam_get_cigar(rec);
    let mut len = 0;
    for i in 0..(*rec).core.n_cigar {
        let item = *cigar.add(i as usize);
        if bam_cigar_op(item) == op {
            len += bam_cigar_oplen(item) as i64;
        }
    }
    len
}

#[test]
fn string_filters_match_original_sam_filter_expected_sam_outputs() {
    unsafe {
        assert_eq!(
            filtered_sam("htslib/test/ce#1000.sam", true, |_, rec| {
                qname(rec).contains(".1") && cigar_string(rec).contains('D')
            }),
            include_str!("../htslib/test/sam_filter/string1.out")
        );
        assert_eq!(
            filtered_sam("htslib/test/ce#5b.sam", true, |hdr, rec| {
                rname(hdr, rec) == "CHROMOSOME_II"
            }),
            include_str!("../htslib/test/sam_filter/string2.out")
        );
        assert_eq!(
            filtered_sam("htslib/test/ce#5b.sam", true, |hdr, rec| {
                rname(hdr, rec).contains("CHROMOSOME_II")
            }),
            include_str!("../htslib/test/sam_filter/string3.out")
        );
        assert_eq!(
            filtered_sam("htslib/test/ce#1000.sam", true, |_, rec| {
                cigar_string(rec).contains('D')
            }),
            include_str!("../htslib/test/sam_filter/string4.out")
        );
        assert_eq!(
            filtered_sam("htslib/test/ce#1000.sam", true, |_, rec| {
                seq_string(rec).contains("ATAT")
            }),
            include_str!("../htslib/test/sam_filter/string5.out")
        );
    }
}

#[test]
fn actual_filter_expression_path_matches_original_outputs() {
    unsafe {
        assert_eq!(
            filtered_sam_with_expression(
                "htslib/test/ce#1000.sam",
                true,
                c"qname =~ \"\\.1\" && cigar =~ \"D\""
            ),
            include_str!("../htslib/test/sam_filter/string1.out")
        );
        assert_eq!(
            filtered_sam_with_expression("htslib/test/xx#rg.sam", true, c"library!=\"x\""),
            include_str!("../htslib/test/sam_filter/string7.out")
        );
        assert_eq!(
            filtered_count_with_expression("htslib/test/ce#1000.sam", c"pos % 23 == 11"),
            expected_count(include_str!("../htslib/test/sam_filter/int1.out"))
        );
        assert_eq!(
            filtered_count_with_expression(
                "htslib/test/ce#1000.sam",
                c"[NM]>=10 || [MD]=~\"A.*A.*A\""
            ),
            expected_count(include_str!("../htslib/test/sam_filter/int3.out"))
        );
        assert_eq!(
            filtered_sam_with_expression("htslib/test/realn02.sam", false, c"sclen>=20"),
            include_str!("../htslib/test/sam_filter/func5.out")
        );
    }
}

#[test]
fn read_group_library_filters_match_original_sam_filter_expected_sam_outputs() {
    unsafe {
        let mut libraries = HashMap::new();
        assert_eq!(
            filtered_sam("htslib/test/xx#rg.sam", true, |hdr, rec| {
                if libraries.is_empty() {
                    libraries = rg_library_map(hdr);
                }
                record_library(&libraries, rec).as_deref() == Some("x")
            }),
            include_str!("../htslib/test/sam_filter/string6.out")
        );

        let mut libraries = HashMap::new();
        assert_eq!(
            filtered_sam("htslib/test/xx#rg.sam", true, |hdr, rec| {
                if libraries.is_empty() {
                    libraries = rg_library_map(hdr);
                }
                record_library(&libraries, rec).as_deref() != Some("x")
            }),
            include_str!("../htslib/test/sam_filter/string7.out")
        );
    }
}

#[test]
fn numeric_and_aux_filters_match_original_sam_filter_expected_counts() {
    unsafe {
        let pos_mod = with_sam_records("htslib/test/ce#1000.sam", |_, rec| {
            ((*rec).core.pos + 1) % 23 == 11
        })
        .into_iter()
        .filter(|passes| *passes)
        .count();
        assert_eq!(
            pos_mod,
            expected_count(include_str!("../htslib/test/sam_filter/int1.out"))
        );

        let qlen_ratio = with_sam_records("htslib/test/ce#1000.sam", |_, rec| {
            let qlen = bam_cigar2qlen((*rec).core.n_cigar as i32, bam_get_cigar(rec));
            let denom = (*rec).core.flag as i64 * (*rec).core.qual as i64 + ((*rec).core.pos + 1);
            qlen as f64 / denom as f64 > 5.0
        })
        .into_iter()
        .filter(|passes| *passes)
        .count();
        assert_eq!(
            qlen_ratio,
            expected_count(include_str!("../htslib/test/sam_filter/int2.out"))
        );

        let aux_match = with_sam_records("htslib/test/ce#1000.sam", |_, rec| {
            aux_i(rec, c"NM").is_some_and(|nm| nm >= 10)
                || aux_z(rec, c"MD").is_some_and(|md| {
                    let mut matches = md.match_indices('A');
                    matches.next().is_some() && matches.next().is_some() && matches.next().is_some()
                })
        })
        .into_iter()
        .filter(|passes| *passes)
        .count();
        assert_eq!(
            aux_match,
            expected_count(include_str!("../htslib/test/sam_filter/int3.out"))
        );
    }
}

#[test]
fn function_filters_match_original_sam_filter_expected_outputs() {
    unsafe {
        let seq_len_ne_qlen = with_sam_records("htslib/test/ce#5b.sam", |_, rec| {
            let qlen = bam_cigar2qlen((*rec).core.n_cigar as i32, bam_get_cigar(rec));
            (*rec).core.l_qseq as i64 != qlen
        })
        .into_iter()
        .filter(|passes| *passes)
        .count();
        assert_eq!(
            seq_len_ne_qlen,
            expected_count(include_str!("../htslib/test/sam_filter/func1.out"))
        );

        let min_qual = with_sam_records("htslib/test/ce#1000.sam", |_, rec| {
            min_max_avg_qual(rec).0 >= 20
        })
        .into_iter()
        .filter(|passes| *passes)
        .count();
        assert_eq!(
            min_qual,
            expected_count(include_str!("../htslib/test/sam_filter/func2.out"))
        );

        let max_qual = with_sam_records("htslib/test/ce#1000.sam", |_, rec| {
            min_max_avg_qual(rec).1 <= 20
        })
        .into_iter()
        .filter(|passes| *passes)
        .count();
        assert_eq!(
            max_qual,
            expected_count(include_str!("../htslib/test/sam_filter/func3.out"))
        );

        let avg_qual = with_sam_records("htslib/test/ce#1000.sam", |_, rec| {
            let avg = min_max_avg_qual(rec).2;
            (20.0..=30.0).contains(&avg)
        })
        .into_iter()
        .filter(|passes| *passes)
        .count();
        assert_eq!(
            avg_qual,
            expected_count(include_str!("../htslib/test/sam_filter/func4.out"))
        );

        assert_eq!(
            filtered_sam("htslib/test/realn02.sam", false, |_, rec| {
                clipped_len(rec, BAM_CSOFT_CLIP) >= 20
            }),
            include_str!("../htslib/test/sam_filter/func5.out")
        );
        assert_eq!(
            filtered_sam("htslib/test/realn02.sam", false, |_, rec| {
                bam_cigar2rlen((*rec).core.n_cigar as i32, bam_get_cigar(rec)) < 50
            }),
            include_str!("../htslib/test/sam_filter/func6.out")
        );
        assert_eq!(
            filtered_sam("htslib/test/realn02.sam", false, |_, rec| {
                bam_cigar2qlen((*rec).core.n_cigar as i32, bam_get_cigar(rec)) > 100
            }),
            include_str!("../htslib/test/sam_filter/func7.out")
        );
        assert_eq!(
            filtered_sam("htslib/test/c1#clip.sam", false, |_, rec| {
                clipped_len(rec, BAM_CHARD_CLIP) >= 4
            }),
            include_str!("../htslib/test/sam_filter/func8.out")
        );
    }
}
