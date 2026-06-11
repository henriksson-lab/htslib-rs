use htslib_rs::{
    bam_aux2A, bam_aux2Z, bam_aux2i, bam_auxB2i, bam_auxB_len, bam_aux_get, bam_cigar2qlen,
    bam_cigar2rlen, bam_destroy1, bam_endpos, bam_get_cigar, bam_get_l_aux, bam_get_qname,
    bam_init1, hts_close, hts_open, hts_set_fai_filename, kstring_t, sam_format1, sam_hdr_destroy,
    sam_hdr_read, sam_read1, BAM_CIGAR_TABLE,
};
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn with_sam_records(path: &str, mut f: impl FnMut(usize, *mut htslib_rs::bam1_t)) {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut idx = 0;
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        f(idx, rec);
        idx += 1;
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
}

fn expected_sam_record_lines(path: &str) -> Vec<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
        .lines()
        .filter(|line| !line.starts_with('@'))
        .map(ToOwned::to_owned)
        .collect()
}

unsafe fn formatted_alignment_records(path: &str, reference: Option<&str>) -> Vec<String> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    if let Some(reference) = reference {
        let reference = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference.as_ptr()), 0);
    }
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut line = kstring_t::default();

    let mut out = Vec::new();
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert!(sam_format1(hdr, rec, &mut line) >= 0);
        out.push(String::from_utf8_lossy(&line.data).into_owned());
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

fn canonical_cram_aux_record(line: &str) -> String {
    let mut fields: Vec<String> = line.split('\t').map(ToOwned::to_owned).collect();
    if fields.len() <= 11 {
        return line.to_owned();
    }

    let mut aux: Vec<String> = fields
        .drain(11..)
        .filter(|field| !field.starts_with("MD:Z:") && !field.starts_with("NM:i:"))
        .map(canonical_cram_aux_field)
        .collect();
    aux.sort();
    fields.extend(aux);
    fields.join("\t")
}

fn canonical_cram_aux_field(field: String) -> String {
    let bytes = field.as_bytes();
    if bytes.len() >= 5 && &bytes[2..5] == b":H:" {
        let mut out = format!("{}:B:C", &field[..2]);
        let hex = &field[5..];
        for chunk in hex.as_bytes().chunks(2) {
            if chunk.len() == 2 {
                let text = std::str::from_utf8(chunk).unwrap();
                let value = u8::from_str_radix(text, 16).unwrap();
                out.push(',');
                out.push_str(&value.to_string());
            }
        }
        return out;
    }

    if bytes.len() >= 6 && &bytes[2..5] == b":B:" {
        match bytes[5] {
            b'c' => return canonical_cram_signed_array(&field, 'C', 256),
            b's' => return canonical_cram_signed_array(&field, 'S', 65_536),
            b'i' => return canonical_cram_signed_array(&field, 'I', 4_294_967_296),
            _ => {}
        }
    }

    field
}

fn canonical_cram_signed_array(field: &str, unsigned_type: char, modulus: i64) -> String {
    let mut out = format!("{}:B:{unsigned_type}", &field[..2]);
    if field.len() == 6 {
        return out;
    }
    for value in field[7..].split(',') {
        let signed = value.parse::<i64>().unwrap();
        let unsigned = if signed < 0 { signed + modulus } else { signed };
        out.push(',');
        out.push_str(&unsigned.to_string());
    }
    out
}

fn canonical_cram_aux_records(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .map(|line| canonical_cram_aux_record(&line))
        .collect()
}

unsafe fn assert_sam_formats_to_original_records(path: &str) {
    let actual = formatted_alignment_records(path, None);
    let expected = expected_sam_record_lines(path);
    assert_eq!(actual, expected, "formatted record parity for {path}");
}

unsafe fn aux(rec: *mut htslib_rs::bam1_t, tag: &'static CStr) -> *mut u8 {
    let ptr = bam_aux_get(rec, tag.as_ptr());
    assert!(!ptr.is_null(), "missing aux tag {:?}", tag);
    ptr
}

#[test]
fn real_sam_header_cigar_tab_matches_htslib_character_mapping() {
    unsafe {
        let path = c_fixture("htslib/test/fieldarith.sam");
        let fp = hts_open(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());

        let cigar_tab = (*hdr).cigar_tab;
        assert_eq!(cigar_tab, BAM_CIGAR_TABLE.as_ptr());

        let cigar_ops = b"MIDNSHP=XB";
        for (expected, op) in cigar_ops.iter().enumerate() {
            assert_eq!(*cigar_tab.add(*op as usize), expected as i8);
        }

        let unset = (0..256).filter(|idx| *cigar_tab.add(*idx) < 0).count();
        assert_eq!(unset + cigar_ops.len(), 256);

        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn real_aux_value_java_cram_formats_exactly_like_expected_sam() {
    unsafe {
        let actual = formatted_alignment_records(
            "htslib/test/auxf#values_java.cram",
            Some("htslib/test/auxf.fa"),
        );
        let expected = expected_sam_record_lines("htslib/test/auxf#values.sam");
        assert_eq!(
            canonical_cram_aux_records(actual),
            canonical_cram_aux_records(expected)
        );
    }
}

#[test]
fn real_large_aux_java_cram_formats_exactly_like_expected_sam() {
    unsafe {
        let actual = formatted_alignment_records(
            "htslib/test/xx#large_aux_java.cram",
            Some("htslib/test/xx.fa"),
        );
        let expected = expected_sam_record_lines("htslib/test/xx#large_aux.sam");
        assert_eq!(
            canonical_cram_aux_records(actual),
            canonical_cram_aux_records(expected)
        );
    }
}

#[test]
fn real_cigar_edge_sam_fixtures_format_back_to_original_records() {
    unsafe {
        for path in [
            "htslib/test/c1#pad1.sam",
            "htslib/test/c1#pad2.sam",
            "htslib/test/c1#pad3.sam",
            "htslib/test/c1#clip.sam",
            "htslib/test/c1#bounds.sam",
            "htslib/test/c1#unknown.sam",
            "htslib/test/c2#pad.sam",
        ] {
            assert_sam_formats_to_original_records(path);
        }
    }
}

#[test]
fn real_alignment_tag_and_mate_fixtures_format_back_to_original_records() {
    unsafe {
        for path in [
            "htslib/test/ce#tag_padded.sam",
            "htslib/test/ce#tag_depadded.sam",
            "htslib/test/ce#supp.sam",
            "htslib/test/xx#pair.sam",
            "htslib/test/xx#rg.sam",
            "htslib/test/xx#triplet.sam",
        ] {
            assert_sam_formats_to_original_records(path);
        }
    }
}

#[test]
fn real_unmapped_md_and_repeated_fixtures_format_back_to_original_records() {
    unsafe {
        for path in [
            "htslib/test/ce#unmap.sam",
            "htslib/test/ce#unmap1.sam",
            "htslib/test/ce#unmap2.sam",
            "htslib/test/embed_MD.sam",
            "htslib/test/md#1.sam",
            "htslib/test/xx#MD.sam",
            "htslib/test/xx#MD2.sam",
            "htslib/test/xx#minimal.sam",
            "htslib/test/xx#repeated.sam",
            "htslib/test/xx#repeated2.sam",
        ] {
            assert_sam_formats_to_original_records(path);
        }
    }
}

#[test]
fn real_fieldarith_records_match_cigar_length_aux_expectations() {
    unsafe {
        let mut seen = 0;
        with_sam_records("htslib/test/fieldarith.sam", |_, rec| {
            let qname = CStr::from_ptr(bam_get_qname(rec)).to_bytes();
            let cigar = bam_get_cigar(rec);
            let n_cigar = (*rec).core.n_cigar as i32;

            let qlen = bam_cigar2qlen(n_cigar, cigar);
            let rlen = bam_cigar2rlen(n_cigar, cigar);
            let end = bam_endpos(rec);

            assert_eq!(qlen, bam_aux2i(aux(rec, c"XQ")));
            assert_eq!(rlen, bam_aux2i(aux(rec, c"XR")));
            assert_eq!(end, bam_aux2i(aux(rec, c"XE")));

            match qname {
                b"r1" => assert_eq!((qlen, rlen, end), (8, 8, 57)),
                b"hascigar" => assert_eq!((qlen, rlen, end), (8, 6, 200)),
                b"su3" => assert_eq!((qlen, rlen, end), (2, 2, 500)),
                _ => {}
            }
            seen += 1;
        });
        assert_eq!(seen, 8);
    }
}

#[test]
fn real_aux_value_fixture_preserves_scalar_aux_tags() {
    unsafe {
        let mut seen = 0;
        with_sam_records("htslib/test/auxf#values.sam", |idx, rec| {
            if idx == 0 {
                assert_eq!(CStr::from_ptr(bam_get_qname(rec)), c"Fred");
                assert_eq!(bam_aux2A(aux(rec, c"A!")), b'!' as i8);
                assert_eq!(bam_aux2A(aux(rec, c"Ac")), b'c' as i8);
                assert_eq!(bam_aux2A(aux(rec, c"AC")), b'C' as i8);
                assert_eq!(bam_aux2i(aux(rec, c"I0")), 0);
                assert_eq!(bam_aux2i(aux(rec, c"I7")), 32768);
                assert_eq!(bam_aux2i(aux(rec, c"IA")), 2_147_483_647);
                assert_eq!(bam_aux2i(aux(rec, c"iA")), -2_147_483_647);
                assert_eq!(bam_aux2i(aux(rec, c"iB")), -2_147_483_648);
                assert_eq!(CStr::from_ptr(bam_aux2Z(aux(rec, c"Z0"))), c"space space");
                assert_eq!(CStr::from_ptr(bam_aux2Z(aux(rec, c"Zn"))), c"");
            }
            seen += 1;
        });
        assert_eq!(seen, 2);
    }
}

#[test]
fn real_aux_value_fixture_preserves_array_aux_tags() {
    unsafe {
        with_sam_records("htslib/test/auxf#values.sam", |idx, rec| {
            if idx == 1 {
                assert_eq!(CStr::from_ptr(bam_get_qname(rec)), c"Jim");

                let bc = aux(rec, c"BC");
                assert_eq!(bam_auxB_len(bc), 4);
                assert_eq!(bam_auxB2i(bc, 0), 0);
                assert_eq!(bam_auxB2i(bc, 1), 127);
                assert_eq!(bam_auxB2i(bc, 2), 128);
                assert_eq!(bam_auxB2i(bc, 3), 255);

                let bi = aux(rec, c"Bi");
                assert_eq!(bam_auxB_len(bi), 4);
                assert_eq!(bam_auxB2i(bi, 0), -2_147_483_648);
                assert_eq!(bam_auxB2i(bi, 1), -2_147_483_647);
                assert_eq!(bam_auxB2i(bi, 2), 0);
                assert_eq!(bam_auxB2i(bi, 3), 2_147_483_647);

                assert!(bam_get_l_aux(rec) > 0);
            }
        });
    }
}

#[test]
fn real_base_modification_fixture_exposes_mm_ml_aux_payloads() {
    unsafe {
        let mut seen = 0;
        with_sam_records("htslib/test/base_mods/MM-explicit.sam", |idx, rec| {
            let mm = aux(rec, c"Mm");
            let ml = aux(rec, c"Ml");
            assert!(!bam_aux2Z(mm).is_null());
            assert!(bam_auxB_len(ml) >= 4);

            if idx == 0 {
                assert_eq!(CStr::from_ptr(bam_get_qname(rec)), c"r1");
                assert_eq!(CStr::from_ptr(bam_aux2Z(mm)), c"C+mh,2,0,1;");
                assert_eq!(bam_auxB2i(ml, 0), 200);
                assert_eq!(bam_auxB2i(ml, 1), 10);
            }
            seen += 1;
        });
        assert_eq!(seen, 3);
    }
}
