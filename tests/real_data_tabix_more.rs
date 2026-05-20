use htslib_rs::bgzf::{bgzf_close, bgzf_open, bgzf_write};
use htslib_rs::{
    hts_close, hts_get_bgzfp, hts_itr_destroy, hts_itr_next, hts_open, ks_free, kstring_t,
    tbx_c_96_tbx_parse1, tbx_conf_bed, tbx_conf_gff, tbx_conf_vcf, tbx_destroy, tbx_index_build2,
    tbx_index_load, tbx_index_load2, tbx_intv_t, tbx_itr_querys1, tbx_seqnames,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_path(path: &std::path::Path) -> CString {
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let dir = fixture(".tmp").join("real_data_tabix_more");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(format!("{}-{}", std::process::id(), name))
}

unsafe fn bgzip_copy(src: &std::path::Path, dst: &std::path::Path) {
    let bytes = std::fs::read(src).unwrap();
    let dst_c = c_path(dst);
    let fp = bgzf_open(dst_c.as_ptr(), c"w".as_ptr());
    assert!(!fp.is_null(), "failed to create {}", dst.display());
    assert_eq!(
        bgzf_write(fp, bytes.as_ptr().cast::<c_void>(), bytes.len()),
        bytes.len() as isize
    );
    assert_eq!(bgzf_close(fp), 0);
}

unsafe fn parse_interval(conf: &htslib_rs::tbx_conf_t, line: &str) -> (String, i64, i64) {
    let mut bytes = CString::new(line).unwrap().into_bytes_with_nul();
    let mut intv: tbx_intv_t = std::mem::zeroed();
    assert_eq!(
        tbx_c_96_tbx_parse1(conf, bytes.len() - 1, bytes.as_mut_ptr().cast(), &mut intv,),
        0
    );
    let name = if intv.se.is_null() {
        CStr::from_ptr(intv.ss).to_string_lossy().into_owned()
    } else {
        let len = intv.se.offset_from(intv.ss) as usize;
        let bytes = std::slice::from_raw_parts(intv.ss.cast::<u8>(), len);
        String::from_utf8_lossy(bytes).into_owned()
    };
    (name, intv.beg, intv.end)
}

#[test]
fn parses_tabix_vcf_fixture_intervals() {
    unsafe {
        let conf = tbx_conf_vcf();
        assert_eq!(
            parse_interval(
                &conf,
                "1\t3000150\t.\tC\tT\t59.2\tPASS\tAN=4;AC=2\tGT:GQ\t0/1:245\t0/1:245",
            ),
            ("1".to_string(), 3_000_149, 3_000_150)
        );
        assert_eq!(
            parse_interval(
                &conf,
                "4\t3258448\t.\tTACACACAC\tT\t.\tPASS\tAN=4;AC=2\tGT:GQ:DP\t0/1:325:31\t0/1:325:31",
            ),
            ("4".to_string(), 3_258_447, 3_258_456)
        );
    }
}

#[test]
fn parses_tabix_bed_fixture_intervals() {
    unsafe {
        let conf = tbx_conf_bed();
        assert_eq!(
            parse_interval(&conf, "X\t1000\t1100\tX1\t500\t+\t1000\t1100\t255,0,0",),
            ("X".to_string(), 1000, 1100)
        );
        assert_eq!(
            parse_interval(
                &conf,
                "Y\t100400\t100500\tY3\t600\t+\t100400\t100500\t255,0,0",
            ),
            ("Y".to_string(), 100400, 100500)
        );
    }
}

#[test]
fn parses_tabix_gff_fixture_intervals() {
    unsafe {
        let conf = tbx_conf_gff();
        assert_eq!(
            parse_interval(
                &conf,
                "X\tVega\texon\t2934816\t2935190\t.\t-\t.\tName=OTTHUME00001604789;Parent=OTTHUMT00000055643",
            ),
            ("X".to_string(), 2_934_815, 2_935_190)
        );
        assert_eq!(
            parse_interval(
                &conf,
                "X\tVega\tintron\t2935191\t2936741\t.\t-\t.\tName=intron00049;Parent=OTTHUMT00000055643",
            ),
            ("X".to_string(), 2_935_190, 2_936_741)
        );
    }
}

#[test]
fn parses_large_chromosome_vcf_intervals() {
    unsafe {
        let conf = tbx_conf_vcf();
        assert_eq!(
            parse_interval(&conf, "chr20\t2147483647\t.\tA\tT\t999\tPASS\t."),
            ("chr20".to_string(), 2_147_483_646, 2_147_483_647)
        );
        assert_eq!(
            parse_interval(&conf, "chr20\t2147483648\t.\tG\tA\t999\tPASS\t."),
            ("chr20".to_string(), 2_147_483_647, 2_147_483_648)
        );
    }
}

#[test]
fn parses_htslib_index_vcf_fixture_intervals() {
    unsafe {
        let conf = tbx_conf_vcf();
        assert_eq!(
            parse_interval(
                &conf,
                "1\t9999919\t.\tG\t<*>\t0\t.\tDP=1;I16=1,0,0,0,26,676,0,0,60,3600,0,0,0,0,0,0;QS=1,0;MQ0F=0\tPL\t0,3,26",
            ),
            ("1".to_string(), 9_999_918, 9_999_919)
        );
        assert_eq!(
            parse_interval(
                &conf,
                "10\t3000190\t.\tA\t<*>\t0\t.\tDP=1;I16=0,1,0,0,26,676,0,0,60,3600,0,0,0,0,0,0;QS=1,0;MQ0F=0\tPL\t0,3,26",
            ),
            ("10".to_string(), 3_000_189, 3_000_190)
        );
    }
}

#[test]
fn generated_tbi_for_htslib_index_vcf_loads_exact_reference_names() {
    unsafe {
        let bgz = temp_path("index.vcf.gz");
        let tbi = temp_path("index.vcf.gz.tbi");
        let _ = std::fs::remove_file(&bgz);
        let _ = std::fs::remove_file(&tbi);
        bgzip_copy(&fixture("htslib/test/index.vcf"), &bgz);

        let conf = tbx_conf_vcf();
        let bgz_c = c_path(&bgz);
        let tbi_c = c_path(&tbi);
        assert_eq!(
            tbx_index_build2(bgz_c.as_ptr(), tbi_c.as_ptr(), 0, &conf),
            0
        );

        let tbx = tbx_index_load2(bgz_c.as_ptr(), tbi_c.as_ptr());
        assert!(!tbx.is_null());
        let mut n = 0;
        let names = tbx_seqnames(tbx, &mut n);
        assert!(!names.is_null());
        assert_eq!(n, 3);
        let got = (0..n)
            .map(|i| {
                CStr::from_ptr(*names.add(i as usize))
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        libc::free(names.cast());
        tbx_destroy(tbx);
        assert_eq!(got, ["1", "2", "10"]);
    }
}

#[test]
fn custom_idx_decorated_tabix_lookup_matches_htslib_tabix_out_exactly() {
    unsafe {
        let bgz = temp_path("custom-index.vcf.gz");
        let tbi = temp_path("custom-index.alt.tbi");
        let _ = std::fs::remove_file(&bgz);
        let _ = std::fs::remove_file(&tbi);
        bgzip_copy(&fixture("htslib/test/index.vcf"), &bgz);

        let conf = tbx_conf_vcf();
        let bgz_c = c_path(&bgz);
        let tbi_c = c_path(&tbi);
        assert_eq!(
            tbx_index_build2(bgz_c.as_ptr(), tbi_c.as_ptr(), 0, &conf),
            0
        );

        let decorated = CString::new(format!("{}##idx##{}", bgz.display(), tbi.display())).unwrap();
        let tbx = tbx_index_load(decorated.as_ptr());
        assert!(!tbx.is_null());

        let fp = hts_open(decorated.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        let itr = tbx_itr_querys1(tbx, c"1:10000060-10000060".as_ptr());
        assert!(!itr.is_null());

        let mut line: kstring_t = std::mem::zeroed();
        let mut rows = Vec::new();
        loop {
            let ret = hts_itr_next(
                hts_get_bgzfp(fp),
                itr,
                (&mut line as *mut kstring_t).cast::<c_void>(),
                tbx.cast::<c_void>(),
            );
            if ret < 0 {
                break;
            }
            let bytes = std::slice::from_raw_parts(line.s.cast::<u8>(), line.l);
            rows.push(String::from_utf8_lossy(bytes).into_owned());
        }

        ks_free(&mut line);
        hts_itr_destroy(itr);
        assert_eq!(hts_close(fp), 0);
        tbx_destroy(tbx);

        let mut got = rows.join("\n");
        got.push('\n');
        assert_eq!(got, include_str!("../htslib/test/tabix.out"));
    }
}
