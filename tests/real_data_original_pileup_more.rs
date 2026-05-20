use htslib_rs::{
    bam1_t, bam_get_qual, bam_get_seq, bam_mplp_auto, bam_mplp_destroy, bam_mplp_init,
    bam_mplp_init_overlaps, bam_pileup1_is_del, bam_pileup1_is_head, bam_pileup1_is_refskip,
    bam_pileup1_is_tail, bam_pileup1_t, bam_plp_auto, bam_plp_destroy, bam_plp_init,
    bam_plp_insertion, bam_seqi, htsFile, hts_close, hts_open, ks_free, kstring_t, sam_hdr_destroy,
    sam_hdr_read, sam_hdr_t, sam_hdr_tid2name, sam_read1, BAM_FDUP, BAM_FQCFAIL, BAM_FREVERSE,
    BAM_FSECONDARY, BAM_FUNMAP,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_void};

const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_fixture(path: &str) -> CString {
    CString::new(fixture(path).to_string_lossy().as_bytes()).unwrap()
}

fn expected_lines(path: &str) -> Vec<String> {
    std::fs::read_to_string(fixture(path))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

struct SamReader {
    fp: *mut htsFile,
    hdr: *mut sam_hdr_t,
}

impl SamReader {
    unsafe fn open(path: &str) -> Self {
        let path = c_fixture(path);
        let fp = hts_open(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

        let hdr = sam_hdr_read(fp);
        assert!(
            !hdr.is_null(),
            "failed to read header from {}",
            path.to_string_lossy()
        );

        Self { fp, hdr }
    }

    unsafe fn target_name(&self, tid: c_int) -> String {
        CStr::from_ptr(sam_hdr_tid2name(self.hdr, tid))
            .to_string_lossy()
            .into_owned()
    }
}

impl Drop for SamReader {
    fn drop(&mut self) {
        unsafe {
            sam_hdr_destroy(self.hdr);
            assert_eq!(hts_close(self.fp), 0);
        }
    }
}

unsafe extern "C" fn read_original_pileup_record(data: *mut c_void, rec: *mut bam1_t) -> c_int {
    let reader = &mut *(data.cast::<SamReader>());

    loop {
        let ret = sam_read1(reader.fp, reader.hdr, rec);
        if ret < 0 {
            return ret;
        }
        if ((*rec).core.flag as c_int & (BAM_FUNMAP | BAM_FSECONDARY | BAM_FQCFAIL | BAM_FDUP)) == 0
        {
            return ret;
        }
    }
}

unsafe fn push_original_pileup_seq(out: &mut String, mut p: *const bam_pileup1_t, n: c_int) {
    let mut ks = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };

    for _ in 0..n {
        let is_rev = ((*(*p).b).core.flag as c_int & BAM_FREVERSE) != 0;

        if bam_pileup1_is_head(p) != 0 {
            out.push('^');
            out.push((b'!' + (*(*p).b).core.qual.min(93)) as char);
        }

        if bam_pileup1_is_del(p) != 0 {
            out.push(if bam_pileup1_is_refskip(p) != 0 {
                if is_rev {
                    '<'
                } else {
                    '>'
                }
            } else {
                '*'
            });
        } else {
            let base = SEQ_NT16_STR[bam_seqi(bam_get_seq((*p).b), (*p).qpos as usize) as usize];
            out.push(if is_rev {
                base.to_ascii_lowercase() as char
            } else {
                base.to_ascii_uppercase() as char
            });
        }

        let mut del_len = -(*p).indel;
        if (*p).indel > 0 {
            let len = bam_plp_insertion(p, &mut ks, &mut del_len);
            assert!(len >= 0, "bam_plp_insertion failed");
            out.push_str(&format!("{:+}(", len));
            for j in 0..len {
                let base = *ks.s.add(j as usize) as u8;
                out.push(if is_rev {
                    base.to_ascii_lowercase() as char
                } else {
                    base.to_ascii_uppercase() as char
                });
            }
            out.push(')');
        }
        if del_len > 0 {
            out.push_str(&format!("-{}()", del_len));
        }
        if bam_pileup1_is_tail(p) != 0 {
            out.push('$');
        }

        p = p.add(1);
    }

    ks_free(&mut ks);
}

unsafe fn push_original_pileup_qual(out: &mut String, mut p: *const bam_pileup1_t, n: c_int) {
    for _ in 0..n {
        let qual = bam_get_qual((*p).b);
        let q = if (*p).qpos < (*(*p).b).core.l_qseq {
            let q = *qual.add((*p).qpos as usize) as u16 + 33;
            if q < b'~' as u16 {
                q as u8
            } else {
                b'~'
            }
        } else {
            b'~'
        };
        out.push(q as char);
        p = p.add(1);
    }
}

unsafe fn format_original_column(
    reader: &SamReader,
    tid: c_int,
    pos: c_int,
    n: c_int,
    pileup: *const bam_pileup1_t,
) -> String {
    let mut out = format!("{}\t{}\t{}\t", reader.target_name(tid), pos + 1, n);
    push_original_pileup_seq(&mut out, pileup, n);
    out.push('\t');
    push_original_pileup_qual(&mut out, pileup, n);
    out
}

unsafe fn collect_plp_lines(path: &str) -> Vec<String> {
    let mut reader = SamReader::open(path);
    let iter = bam_plp_init(
        Some(read_original_pileup_record),
        (&mut reader as *mut SamReader).cast(),
    );
    assert!(!iter.is_null());

    let mut lines = Vec::new();
    loop {
        let mut tid = -1;
        let mut pos = -1;
        let mut n = 0;
        let pileup = bam_plp_auto(iter, &mut tid, &mut pos, &mut n);
        if pileup.is_null() {
            assert_eq!(n, 0);
            break;
        }
        assert!(tid >= 0);
        lines.push(format_original_column(&reader, tid, pos, n, pileup));
    }

    bam_plp_destroy(iter);
    lines
}

unsafe fn collect_mplp_lines(path: &str) -> Vec<String> {
    let mut reader = SamReader::open(path);
    let mut data = (&mut reader as *mut SamReader).cast::<c_void>();
    let iter = bam_mplp_init(1, Some(read_original_pileup_record), &mut data);
    assert!(!iter.is_null());
    assert_eq!(bam_mplp_init_overlaps(iter), 0);

    let mut lines = Vec::new();
    loop {
        let mut tid = -1;
        let mut pos = -1;
        let mut n_plp = [0];
        let mut pileups = [std::ptr::null::<bam_pileup1_t>()];
        let ret = bam_mplp_auto(
            iter,
            &mut tid,
            &mut pos,
            n_plp.as_mut_ptr(),
            pileups.as_mut_ptr(),
        );
        if ret <= 0 {
            assert_eq!(ret, 0);
            break;
        }
        assert!(tid >= 0);
        lines.push(format_original_column(
            &reader, tid, pos, n_plp[0], pileups[0],
        ));
    }

    bam_mplp_destroy(iter);
    lines
}

unsafe fn assert_plp_matches_original_fixture(sam: &str, out: &str) {
    assert_eq!(collect_plp_lines(sam), expected_lines(out));
}

unsafe fn assert_mplp_matches_original_fixture(sam: &str, out: &str) {
    assert_eq!(collect_mplp_lines(sam), expected_lines(out));
}

#[test]
fn original_pileup_insertions_and_refskips_match_htslib_expected_outputs() {
    unsafe {
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/mp_D.sam",
            "htslib/test/mpileup/mp_D.out",
        );
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/mp_DI.sam",
            "htslib/test/mpileup/mp_DI.out",
        );
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/mp_I.sam",
            "htslib/test/mpileup/mp_I.out",
        );
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/mp_P.sam",
            "htslib/test/mpileup/mp_P.out",
        );
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/mp_N.sam",
            "htslib/test/mpileup/mp_N.out",
        );
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/mp_N2.sam",
            "htslib/test/mpileup/mp_N2.out",
        );
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/mp_ID.sam",
            "htslib/test/mpileup/mp_ID.out",
        );
    }
}

#[test]
fn original_pileup_pad_columns_match_htslib_expected_outputs() {
    unsafe {
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/c1#pad1.sam",
            "htslib/test/mpileup/c1#pad1.out",
        );
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/c1#pad2.sam",
            "htslib/test/mpileup/c1#pad2.out",
        );
        assert_plp_matches_original_fixture(
            "htslib/test/mpileup/c1#pad3.sam",
            "htslib/test/mpileup/c1#pad3.out",
        );
    }
}

#[test]
fn original_mpileup_overlap_and_small_bam_match_htslib_expected_outputs() {
    unsafe {
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/mp_overlap1.sam",
            "htslib/test/mpileup/mp_overlap1.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/mp_overlap2.sam",
            "htslib/test/mpileup/mp_overlap2.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/small.bam",
            "htslib/test/mpileup/small.out",
        );
    }
}

#[test]
fn original_mpileup_indels_refskips_and_pads_match_htslib_expected_outputs() {
    unsafe {
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/mp_D.sam",
            "htslib/test/mpileup/mp_D.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/mp_DI.sam",
            "htslib/test/mpileup/mp_DI.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/mp_I.sam",
            "htslib/test/mpileup/mp_I.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/mp_P.sam",
            "htslib/test/mpileup/mp_P.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/mp_ID.sam",
            "htslib/test/mpileup/mp_ID.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/mp_N.sam",
            "htslib/test/mpileup/mp_N.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/mp_N2.sam",
            "htslib/test/mpileup/mp_N2.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/c1#pad1.sam",
            "htslib/test/mpileup/c1#pad1.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/c1#pad2.sam",
            "htslib/test/mpileup/c1#pad2.out",
        );
        assert_mplp_matches_original_fixture(
            "htslib/test/mpileup/c1#pad3.sam",
            "htslib/test/mpileup/c1#pad3.out",
        );
    }
}
