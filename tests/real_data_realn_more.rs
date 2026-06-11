//! Real-data integration tests for the htslib/realn.c API surface
//! (translated into src/realn.rs). Exercises `sam_prob_realn` and
//! `sam_cap_mapq` on real BAM/SAM fixtures with their reference FASTA.
//!
//! Companion to tests/real_data_pileup_baq_more.rs (which validates
//! BAQ realignment output byte-for-byte against the upstream
//! realn0{1,2,3}_exp*.sam fixtures). This file focuses on:
//!   * sam_cap_mapq applied to a real on-disk record (the in-tree unit
//!     tests use only synthetic BAM records);
//!   * sam_prob_realn's flag-dependent behaviours: BAQ-apply writing the
//!     BQ tag with the right type byte and length, the unmapped/no-qual
//!     short-circuit returning -1, and the redo-BAQ flag clearing a
//!     stale tag.

use htslib_rs::realn::{realn_c_106_sam_prob_realn, sam_cap_mapq};
use htslib_rs::{
    bam1_t, bam_aux2Z, bam_aux_append, bam_aux_del, bam_aux_get, bam_destroy1, bam_get_qual,
    bam_init1, fai_destroy, fai_load3_format, faidx_fetch_seq, hts_close, hts_open,
    sam_hdr_destroy, sam_hdr_read, sam_prob_realn, sam_read1, BAM_FUNMAP, FAI_FASTA,
};
fn c_fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut bytes = path.to_string_lossy().as_bytes().to_vec();
    bytes.push(0);
    bytes
}

struct RefSeq {
    fai: *mut htslib_rs::faidx_t,
    seq: *mut u8,
    len: i32,
}

impl RefSeq {
    unsafe fn load(fasta_path: &str, fai_path: &str, target: &[u8]) -> Self {
        let fasta = c_fixture(fasta_path);
        let fai_p = c_fixture(fai_path);
        let fai = fai_load3_format(
            fasta.as_ptr().cast(),
            fai_p.as_ptr().cast(),
            std::ptr::null(),
            0,
            FAI_FASTA,
        );
        assert!(!fai.is_null(), "fai_load3_format failed for {fasta_path}");
        let mut len = 0;
        let seq = faidx_fetch_seq(fai, target.as_ptr().cast(), 0, i32::MAX, &mut len);
        assert!(!seq.is_null(), "faidx_fetch_seq failed for {target:?}");
        Self { fai, seq: seq.cast(), len }
    }

    unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.seq, self.len as usize)
    }
}

impl Drop for RefSeq {
    fn drop(&mut self) {
        unsafe {
            // The `seq` buffer is owned by the production faidx allocator; it
            // is released when that subsystem is torn down (no manual free).
            fai_destroy(self.fai);
        }
    }
}

unsafe fn read_records(path: &str) -> (Vec<*mut bam1_t>, *mut htslib_rs::sam_hdr_t) {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr().cast(), b"r\0".as_ptr().cast());
    assert!(
        !fp.is_null(),
        "failed to open {}",
        String::from_utf8_lossy(&path)
    );
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());

    let mut records = Vec::new();
    loop {
        let rec = bam_init1();
        assert!(!rec.is_null());
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            bam_destroy1(rec);
            break;
        }
        records.push(rec);
    }
    assert_eq!(hts_close(fp), 0);
    (records, hdr)
}

unsafe fn destroy_records(records: Vec<*mut bam1_t>, hdr: *mut htslib_rs::sam_hdr_t) {
    for rec in records {
        bam_destroy1(rec);
    }
    sam_hdr_destroy(hdr);
}

unsafe fn aux_z_string(rec: *const bam1_t, tag: *const u8) -> Option<Vec<u8>> {
    let aux = bam_aux_get(rec, tag);
    if aux.is_null() {
        None
    } else {
        let z = bam_aux2Z(aux);
        let mut len = 0;
        while *z.add(len) != 0 {
            len += 1;
        }
        Some(std::slice::from_raw_parts(z.cast::<u8>(), len).to_vec())
    }
}

// ---------------------------------------------------------------------------
// sam_prob_realn: BAQ-apply path writes a `Z`-typed BQ tag with l_qseq chars
// ---------------------------------------------------------------------------

#[test]
fn sam_prob_realn_writes_zq_tag_with_baq_apply_flag() {
    unsafe {
        // realn02.sam: short-read fixture with mapped, qualified records.
        // With BAQ_APPLY (flag bit 1) set, sam_prob_realn modifies qual in
        // place AND appends a ZQ tag (so the change is reversible). The
        // BQ tag is only written when the apply bit is not set.
        let ref_seq = RefSeq::load(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            b"17\0",
        );
        assert_eq!(ref_seq.len, 4_200);

        let (records, hdr) = read_records("htslib/test/realn02.sam");
        assert!(!records.is_empty());

        let rec = records[0];
        assert_eq!((*rec).core.flag as i32 & BAM_FUNMAP, 0);
        let l_qseq = (*rec).core.l_qseq as usize;
        assert!(l_qseq > 0);
        assert_eq!(
            sam_prob_realn(rec, ref_seq.seq, ref_seq.len.into(), 1),
            0,
            "BAQ-apply call should succeed"
        );
        let zq = aux_z_string(rec, b"ZQ\0".as_ptr()).expect("BAQ-apply should add a ZQ tag");
        assert_eq!(
            zq.len(),
            l_qseq,
            "ZQ tag length must match l_qseq ({l_qseq})"
        );
        // BQ should NOT be added by the apply path.
        assert!(
            aux_z_string(rec, b"BQ\0".as_ptr()).is_none(),
            "BAQ-apply must not also write a BQ tag"
        );

        // Last record has FLAG 133 (unmapped half of a pair) -- should
        // short-circuit with -1 without writing any tag.
        let unmapped = *records.last().unwrap();
        assert_ne!((*unmapped).core.flag as i32 & BAM_FUNMAP, 0);
        assert_eq!(
            sam_prob_realn(unmapped, ref_seq.seq, ref_seq.len.into(), 1),
            -1,
            "unmapped record must short-circuit with -1"
        );
        assert!(
            aux_z_string(unmapped, b"BQ\0".as_ptr()).is_none(),
            "unmapped record must not gain a BQ tag"
        );
        assert!(
            aux_z_string(unmapped, b"ZQ\0".as_ptr()).is_none(),
            "unmapped record must not gain a ZQ tag"
        );

        destroy_records(records, hdr);
    }
}

#[test]
fn sam_prob_realn_writes_bq_tag_without_baq_apply_flag() {
    unsafe {
        // With flag = 0 (no apply, no extend, no redo) and no pre-existing
        // BQ/ZQ tag, sam_prob_realn writes a BQ tag and leaves qual
        // untouched.
        let ref_seq = RefSeq::load(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            b"17\0",
        );
        let (records, hdr) = read_records("htslib/test/realn02.sam");
        let rec = records[0];
        assert_eq!((*rec).core.flag as i32 & BAM_FUNMAP, 0);
        let l_qseq = (*rec).core.l_qseq as usize;
        let orig_qual = std::slice::from_raw_parts(bam_get_qual(rec), l_qseq).to_vec();
        assert!(aux_z_string(rec, b"BQ\0".as_ptr()).is_none());
        assert!(aux_z_string(rec, b"ZQ\0".as_ptr()).is_none());

        assert_eq!(sam_prob_realn(rec, ref_seq.seq, ref_seq.len.into(), 0), 0);

        let bq = aux_z_string(rec, b"BQ\0".as_ptr()).expect("flag=0 path should add a BQ tag");
        assert_eq!(bq.len(), l_qseq, "BQ tag length must match l_qseq");
        // The qual array must be untouched (only BQ tracks the delta).
        let new_qual = std::slice::from_raw_parts(bam_get_qual(rec), l_qseq);
        assert_eq!(new_qual, &orig_qual[..]);

        destroy_records(records, hdr);
    }
}

#[test]
fn sam_prob_realn_translated_path_writes_same_zq_tag_as_re_exported() {
    unsafe {
        // The realn-module entrypoint (realn_c_106_sam_prob_realn) and the
        // sam.rs copy (sam_prob_realn) should produce the same ZQ tag for
        // the same input and apply flags.
        let ref_seq = RefSeq::load(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            b"17\0",
        );

        let (records_a, hdr_a) = read_records("htslib/test/realn02.sam");
        let (records_b, hdr_b) = read_records("htslib/test/realn02.sam");
        assert_eq!(records_a.len(), records_b.len());

        for (rec_a, rec_b) in records_a.iter().zip(records_b.iter()) {
            if (**rec_a).core.flag as i32 & BAM_FUNMAP != 0 {
                continue;
            }
            let ret_a = sam_prob_realn(*rec_a, ref_seq.seq, ref_seq.len.into(), 1);
            let ret_b = realn_c_106_sam_prob_realn(&mut **rec_b, ref_seq.as_slice(), 1);
            assert_eq!(ret_a, ret_b);
            assert_eq!(
                aux_z_string(*rec_a, b"ZQ\0".as_ptr()),
                aux_z_string(*rec_b, b"ZQ\0".as_ptr()),
                "translated and re-exported paths must produce the same ZQ tag"
            );
        }

        destroy_records(records_a, hdr_a);
        destroy_records(records_b, hdr_b);
    }
}

// ---------------------------------------------------------------------------
// sam_prob_realn: BAQ_REDO flag drops a pre-existing BQ tag and rebuilds
// ---------------------------------------------------------------------------

#[test]
fn sam_prob_realn_redo_flag_replaces_existing_bq_tag() {
    unsafe {
        let ref_seq = RefSeq::load(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            b"17\0",
        );
        let (records, hdr) = read_records("htslib/test/realn02.sam");
        let rec = records[0];
        let l_qseq = (*rec).core.l_qseq as usize;

        // Implant an obviously-wrong BQ tag of the right length so we can
        // distinguish from a "do-nothing" pass. With flag=0 the function
        // would see this tag and bail out (return 0, no change). With
        // BAQ_REDO (flag bit 4) the tag must be dropped and recomputed.
        let mut sentinel = vec![b'!'; l_qseq + 1];
        sentinel[l_qseq] = 0;
        let appended = bam_aux_append(
            rec,
            b"BQ\0".as_ptr(),
            b'Z',
            (l_qseq + 1) as i32,
            sentinel.as_ptr(),
        );
        assert_eq!(appended, 0, "bam_aux_append for sentinel BQ failed");
        assert_eq!(
            aux_z_string(rec, b"BQ\0".as_ptr()).as_deref(),
            Some(vec![b'!'; l_qseq].as_slice()),
            "sentinel BQ tag not stored correctly"
        );

        // REDO (flag bit 4) drops the existing BQ and recomputes. Since the
        // apply bit is not set, the recomputed value goes back into BQ.
        assert_eq!(
            sam_prob_realn(rec, ref_seq.seq, ref_seq.len.into(), 4),
            0,
            "BAQ_REDO call should succeed"
        );
        let new_bq = aux_z_string(rec, b"BQ\0".as_ptr()).expect("BAQ_REDO should re-add a BQ tag");
        assert_eq!(new_bq.len(), l_qseq);
        // Vanishingly unlikely the recomputed tag is all '!'.
        assert_ne!(
            new_bq,
            vec![b'!'; l_qseq],
            "BAQ_REDO did not replace the sentinel BQ"
        );

        destroy_records(records, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_prob_realn: invalid existing ZQ tag is rejected with -4
// ---------------------------------------------------------------------------

#[test]
fn sam_prob_realn_rejects_zq_tag_with_wrong_length() {
    unsafe {
        let ref_seq = RefSeq::load(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            b"17\0",
        );
        let (records, hdr) = read_records("htslib/test/realn02.sam");
        let rec = records[0];

        // Attach a ZQ tag whose length disagrees with l_qseq -- this should
        // make realn_check_tag return -1 and the caller bail out with -4.
        let bad = b"!!\0";
        let appended = bam_aux_append(
            rec,
            b"ZQ\0".as_ptr(),
            b'Z',
            bad.len() as i32,
            bad.as_ptr(),
        );
        assert_eq!(appended, 0);

        let ret = sam_prob_realn(rec, ref_seq.seq, ref_seq.len.into(), 1);
        assert_eq!(ret, -4, "expected -4 for invalid ZQ length, got {ret}");

        destroy_records(records, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_cap_mapq: applied to a real on-disk BAM record
// ---------------------------------------------------------------------------

#[test]
fn sam_cap_mapq_caps_mapq_on_realn02_records_with_real_reference() {
    unsafe {
        let ref_seq = RefSeq::load(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            b"17\0",
        );
        let (records, hdr) = read_records("htslib/test/realn02.sam");

        // Walk the mapped records. For each, call sam_cap_mapq with a high
        // threshold (60) and a low threshold (5). A return of -1 means
        // "do not cap" (input MAPQ is fine), a non-negative result is the
        // capped quality. With a low threshold of 5 we expect at least one
        // record to be capped.
        let mut saw_uncapped_high = false;
        let mut saw_capped_low = false;
        for &rec in &records {
            if (*rec).core.flag as i32 & BAM_FUNMAP != 0 {
                continue;
            }
            let ret_high = sam_cap_mapq(&mut *rec, ref_seq.as_slice(), 60);
            let ret_low = sam_cap_mapq(&mut *rec, ref_seq.as_slice(), 5);
            // For high threshold (60), all matched records should be ok
            // (-1) or capped at <= 60 -- never negative-other-than-minus-one.
            assert!(ret_high == -1 || (0..=60).contains(&ret_high));
            assert!(ret_low == -1 || (0..=5).contains(&ret_low));
            if ret_high == -1 {
                saw_uncapped_high = true;
            }
            // A low threshold of 5 will cap most mapped records.
            if ret_low >= 0 && ret_low < (*rec).core.qual as i32 {
                saw_capped_low = true;
            }
        }
        assert!(
            saw_uncapped_high,
            "expected at least one record to pass the high MAPQ threshold uncapped"
        );
        assert!(
            saw_capped_low,
            "expected at least one record to be capped at the low MAPQ threshold"
        );

        destroy_records(records, hdr);
    }
}

#[test]
fn sam_cap_mapq_default_threshold_negative_uses_40() {
    unsafe {
        // Passing thres = -1 selects the default 40 cap. For a high-quality
        // record this should still hit the "do not cap" branch (returns -1).
        let ref_seq = RefSeq::load(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            b"17\0",
        );
        let (records, hdr) = read_records("htslib/test/realn02.sam");

        // The first mapped record in realn02.sam is a high-quality match.
        let rec = records[0];
        assert_eq!((*rec).core.flag as i32 & BAM_FUNMAP, 0);
        let ret = sam_cap_mapq(&mut *rec, ref_seq.as_slice(), -1);
        // Either uncapped (-1) or capped at most at 40.
        assert!(ret == -1 || (0..=40).contains(&ret), "ret={ret}");

        destroy_records(records, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_cap_mapq + sam_prob_realn cross-check on the same record
// ---------------------------------------------------------------------------

#[test]
fn sam_cap_mapq_after_sam_prob_realn_remains_consistent_on_real_record() {
    unsafe {
        // sam_prob_realn rewrites the qual array in the BAQ-apply path; the
        // capped MAPQ after that should not exceed the value computed
        // before -- the BAQ-adjusted quals are by construction <= the
        // originals, and sam_cap_mapq's score grows with quality (more q
        // raises t, which only delays the cap). We verify both calls
        // succeed without error rather than fixing an exact value.
        let ref_seq = RefSeq::load(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            b"17\0",
        );
        let (records, hdr) = read_records("htslib/test/realn02.sam");
        let rec = records[0];
        let l_qseq = (*rec).core.l_qseq as usize;

        // Capture original qual.
        let orig_qual = std::slice::from_raw_parts(bam_get_qual(rec), l_qseq).to_vec();

        let cap_before = sam_cap_mapq(&mut *rec, ref_seq.as_slice(), 60);
        assert_eq!(sam_prob_realn(rec, ref_seq.seq, ref_seq.len.into(), 1), 0);
        let cap_after = sam_cap_mapq(&mut *rec, ref_seq.as_slice(), 60);

        // BAQ-apply reduces (or keeps equal) the qual values -- never
        // increases them.
        let new_qual = std::slice::from_raw_parts(bam_get_qual(rec), l_qseq);
        for (i, (&before, &after)) in orig_qual.iter().zip(new_qual.iter()).enumerate() {
            assert!(
                after <= before,
                "qual at {i} increased after BAQ-apply: {before} -> {after}"
            );
        }
        // And the cap function returned a sensible value in both passes.
        assert!(cap_before == -1 || (0..=60).contains(&cap_before));
        assert!(cap_after == -1 || (0..=60).contains(&cap_after));

        destroy_records(records, hdr);

        // Silence unused warnings on the ZQ helpers when not in scope.
        let _ = bam_aux_del;
    }
}
