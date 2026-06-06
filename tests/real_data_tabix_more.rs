use htslib_rs::bgzf::{bgzf_close, bgzf_open, bgzf_read, bgzf_write};
use htslib_rs::{
    bcf_destroy, bcf_hdr_destroy, bcf_hdr_read, bcf_init, bcf_read, bcf_seqname, hts_close,
    hts_get_bgzfp, hts_itr_destroy, hts_itr_next, hts_open, ks_free, kstring_t,
    tbx_c_96_tbx_parse1, tbx_conf_bed, tbx_conf_gff, tbx_conf_vcf, tbx_destroy, tbx_index_build2,
    tbx_index_load, tbx_index_load2, tbx_intv_t, tbx_itr_querys1, tbx_seqnames,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::process::Command;

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

unsafe fn query_tabix_rows(
    src: &str,
    index_suffix: &str,
    min_shift: i32,
    conf: &htslib_rs::tbx_conf_t,
    region: &CStr,
) -> Vec<String> {
    let bgz = temp_path(&format!("{src}.{index_suffix}.gz").replace('/', "_"));
    let idx = temp_path(&format!("{src}.{index_suffix}.gz.{index_suffix}").replace('/', "_"));
    let _ = std::fs::remove_file(&bgz);
    let _ = std::fs::remove_file(&idx);
    bgzip_copy(&fixture(src), &bgz);

    let bgz_c = c_path(&bgz);
    let idx_c = c_path(&idx);
    assert_eq!(
        tbx_index_build2(bgz_c.as_ptr(), idx_c.as_ptr(), min_shift, conf),
        0
    );

    let tbx = tbx_index_load2(bgz_c.as_ptr(), idx_c.as_ptr());
    assert!(!tbx.is_null());
    let fp = hts_open(bgz_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null());
    let itr = tbx_itr_querys1(tbx, region.as_ptr());
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
    let _ = std::fs::remove_file(bgz);
    let _ = std::fs::remove_file(idx);
    rows
}

unsafe fn query_tabix_rows_from_paths(
    bgz: &std::path::Path,
    idx: &std::path::Path,
    region: &CStr,
) -> Vec<String> {
    let bgz_c = c_path(bgz);
    let idx_c = c_path(idx);
    let tbx = tbx_index_load2(bgz_c.as_ptr(), idx_c.as_ptr());
    assert!(!tbx.is_null());
    let fp = hts_open(bgz_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null());
    let itr = tbx_itr_querys1(tbx, region.as_ptr());
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
    rows
}

unsafe fn build_large_generated_index_vcf(
    record_count: usize,
    label: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let plain = temp_path(&format!("{label}.vcf"));
    let bgz = temp_path(&format!("{label}.vcf.gz"));
    let tbi = temp_path(&format!("{label}.vcf.gz.tbi"));
    let _ = std::fs::remove_file(&plain);
    let _ = std::fs::remove_file(&bgz);
    let _ = std::fs::remove_file(&tbi);

    let src = std::fs::read_to_string(fixture("htslib/test/index.vcf")).unwrap();
    let mut header = Vec::new();
    let mut records = Vec::new();
    for line in src.lines() {
        if line.starts_with('#') {
            header.push(line);
        } else if !line.is_empty() {
            records.push(line.split('\t').map(str::to_string).collect::<Vec<_>>());
        }
    }
    assert!(!records.is_empty());

    let mut out = String::with_capacity(src.len() * (record_count / records.len()).max(1));
    for line in header {
        out.push_str(line);
        out.push('\n');
    }
    for i in 0..record_count {
        let mut rec = records[i % records.len()].clone();
        rec[0] = "1".to_string();
        rec[1] = (i + 1).to_string();
        out.push_str(&rec.join("\t"));
        out.push('\n');
    }
    std::fs::write(&plain, out).unwrap();
    bgzip_copy(&plain, &bgz);
    let conf = tbx_conf_vcf();
    let bgz_c = c_path(&bgz);
    let tbi_c = c_path(&tbi);
    assert_eq!(
        tbx_index_build2(bgz_c.as_ptr(), tbi_c.as_ptr(), 0, &conf),
        0
    );
    let _ = std::fs::remove_file(plain);
    (bgz, tbi)
}

unsafe fn build_large_generated_bed(
    records_per_chromosome: usize,
    label: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let plain = temp_path(&format!("{label}.bed"));
    let bgz = temp_path(&format!("{label}.bed.gz"));
    let tbi = temp_path(&format!("{label}.bed.gz.tbi"));
    let _ = std::fs::remove_file(&plain);
    let _ = std::fs::remove_file(&bgz);
    let _ = std::fs::remove_file(&tbi);

    let mut out = String::with_capacity(records_per_chromosome * 4 * 48);
    for chrom in 1..=4 {
        for i in 0..records_per_chromosome {
            let start = (i * 20) as i64;
            let end = start + 100;
            out.push_str(&format!(
                "chr{chrom}\t{start}\t{end}\tfeature_{chrom}_{i}\t{}\t{}\n",
                (i % 1000) + 1,
                if i % 2 == 0 { "+" } else { "-" }
            ));
        }
    }
    std::fs::write(&plain, out).unwrap();
    bgzip_copy(&plain, &bgz);

    let conf = tbx_conf_bed();
    let bgz_c = c_path(&bgz);
    let tbi_c = c_path(&tbi);
    assert_eq!(
        tbx_index_build2(bgz_c.as_ptr(), tbi_c.as_ptr(), 0, &conf),
        0
    );
    let _ = std::fs::remove_file(plain);
    (bgz, tbi)
}

fn run_tabix_command(program: &std::path::Path, bgz: &std::path::Path, region: &str) -> Vec<u8> {
    let output = Command::new(program)
        .arg(bgz)
        .arg(region)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", program.display()));
    assert!(
        output.status.success(),
        "{} exited with {:?}; stderr: {}",
        program.display(),
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn run_tabix_args(program: &std::path::Path, args: &[&std::path::Path]) -> Vec<u8> {
    let output = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", program.display()));
    assert!(
        output.status.success(),
        "{} {:?} exited with {:?}; stderr: {}",
        program.display(),
        args,
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

unsafe fn read_bgzf_to_string(path: &std::path::Path) -> String {
    let path_c = c_path(path);
    let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.display());

    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = bgzf_read(fp, buf.as_mut_ptr().cast::<c_void>(), buf.len());
        assert!(n >= 0, "bgzf_read failed for {}", path.display());
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n as usize]);
    }
    assert_eq!(bgzf_close(fp), 0);
    String::from_utf8(out).unwrap()
}

unsafe fn parse_interval(conf: &htslib_rs::tbx_conf_t, line: &str) -> (String, i64, i64) {
    let mut bytes = CString::new(line).unwrap().into_bytes_with_nul();
    let mut intv: tbx_intv_t = std::mem::zeroed();
    assert_eq!(
        tbx_c_96_tbx_parse1(conf, bytes.len() - 1, bytes.as_mut_ptr().cast(), &mut intv,),
        0
    );
    let ss = intv.ss.expect("parsed interval sequence start").as_ptr();
    let name = if let Some(se) = intv.se {
        let len = se.as_ptr().offset_from(ss) as usize;
        let bytes = std::slice::from_raw_parts(ss.cast::<u8>(), len);
        String::from_utf8_lossy(bytes).into_owned()
    } else {
        CStr::from_ptr(ss).to_string_lossy().into_owned()
    };
    (name, intv.beg, intv.end)
}

#[test]
fn original_tabix_main_list_chroms_prints_index_reference_names() {
    unsafe {
        let bgz = temp_path("main-list-chroms.vcf.gz");
        let tbi = temp_path("main-list-chroms.vcf.gz.tbi");
        let _ = std::fs::remove_file(&bgz);
        let _ = std::fs::remove_file(&tbi);
        bgzip_copy(&fixture("htslib/test/tabix/vcf_file.vcf"), &bgz);

        let rust_tabix = std::path::Path::new(env!("CARGO_BIN_EXE_perf_tabix"));
        run_tabix_args(
            rust_tabix,
            &[
                std::path::Path::new("-p"),
                std::path::Path::new("vcf"),
                &bgz,
            ],
        );
        let stdout = run_tabix_args(rust_tabix, &[std::path::Path::new("-l"), &bgz]);
        assert_eq!(stdout, b"1\n2\n3\n4\n");

        let _ = std::fs::remove_file(bgz);
        let _ = std::fs::remove_file(tbi);
    }
}

#[test]
fn original_tabix_main_reheader_replaces_bgzf_vcf_header_on_stdout() {
    unsafe {
        let bgz = temp_path("main-reheader.vcf.gz");
        let header = temp_path("main-reheader.header");
        let out_bgz = temp_path("main-reheader.out.vcf.gz");
        let _ = std::fs::remove_file(&bgz);
        let _ = std::fs::remove_file(&header);
        let _ = std::fs::remove_file(&out_bgz);
        bgzip_copy(&fixture("htslib/test/tabix/vcf_file.vcf"), &bgz);

        let replacement_header = concat!(
            "##fileformat=VCFv4.1\n",
            "##source=translated-tabix-reheader\n",
            "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tA\tB\n"
        );
        std::fs::write(&header, replacement_header).unwrap();

        let rust_tabix = std::path::Path::new(env!("CARGO_BIN_EXE_perf_tabix"));
        let stdout = run_tabix_args(rust_tabix, &[std::path::Path::new("-r"), &header, &bgz]);
        assert!(!stdout.is_empty());
        std::fs::write(&out_bgz, stdout).unwrap();

        let reheadered = read_bgzf_to_string(&out_bgz);
        assert!(reheadered.starts_with(replacement_header));
        assert!(reheadered
            .contains("\n1\t3000150\t.\tC\tT\t59.2\tPASS\tAN=4;AC=2\tGT:GQ\t0/1:245\t0/1:245\n"));
        assert!(!reheadered.contains("##FILTER=<ID=PASS"));

        let original_body = std::fs::read_to_string(fixture("htslib/test/tabix/vcf_file.vcf"))
            .unwrap()
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        assert_eq!(
            reheadered.strip_prefix(replacement_header).unwrap(),
            original_body
        );

        let _ = std::fs::remove_file(bgz);
        let _ = std::fs::remove_file(header);
        let _ = std::fs::remove_file(out_bgz);
    }
}

#[test]
fn original_tabix_main_builds_vcf_tbi_and_csi_indexes() {
    unsafe {
        let rust_tabix = std::path::Path::new(env!("CARGO_BIN_EXE_perf_tabix"));
        let conf = tbx_conf_vcf();

        let tbi_bgz = temp_path("main-build-tbi.vcf.gz");
        let tbi = temp_path("main-build-tbi.vcf.gz.tbi");
        let _ = std::fs::remove_file(&tbi_bgz);
        let _ = std::fs::remove_file(&tbi);
        bgzip_copy(&fixture("htslib/test/tabix/vcf_file.vcf"), &tbi_bgz);
        run_tabix_args(
            rust_tabix,
            &[
                std::path::Path::new("-p"),
                std::path::Path::new("vcf"),
                &tbi_bgz,
            ],
        );
        assert!(tbi.exists(), "tabix main did not create {}", tbi.display());
        assert_eq!(
            query_tabix_rows_from_paths(&tbi_bgz, &tbi, c"2:3199812-3199812"),
            ["2\t3199812\t.\tG\tGTT,GT\t82.7\tPASS\tAN=4;AC=2,2\tGT:GQ:DP\t1/2:322:26\t1/2:322:26"]
        );

        let csi_bgz = temp_path("main-build-csi.vcf.gz");
        let csi = temp_path("main-build-csi.vcf.gz.csi");
        let _ = std::fs::remove_file(&csi_bgz);
        let _ = std::fs::remove_file(&csi);
        bgzip_copy(&fixture("htslib/test/tabix/large_chr.vcf"), &csi_bgz);
        run_tabix_args(
            rust_tabix,
            &[
                std::path::Path::new("-C"),
                std::path::Path::new("-p"),
                std::path::Path::new("vcf"),
                &csi_bgz,
            ],
        );
        assert!(csi.exists(), "tabix main did not create {}", csi.display());
        let rows = query_tabix_rows_from_paths(&csi_bgz, &csi, c"chr20:2147483647-2147483647");
        assert_eq!(rows, ["chr20\t2147483647\t.\tA\tT\t999\tPASS\t."]);

        assert_eq!(
            query_tabix_rows(
                "htslib/test/tabix/vcf_file.vcf",
                "tbi",
                0,
                &conf,
                c"1:3000151-3000151",
            ),
            ["1\t3000151\t.\tC\tT\t59.2\tPASS\tAN=4;AC=2\tGT:DP:GQ\t0/1:32:245\t0/1:32:245"]
        );

        let _ = std::fs::remove_file(tbi_bgz);
        let _ = std::fs::remove_file(tbi);
        let _ = std::fs::remove_file(csi_bgz);
        let _ = std::fs::remove_file(csi);
    }
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
fn bed_tabix_query_keeps_half_open_overlap_at_right_edge() {
    unsafe {
        let conf = tbx_conf_bed();
        assert_eq!(
            query_tabix_rows(
                "htslib/test/tabix/bed_file.bed",
                "tbi",
                0,
                &conf,
                c"Y:100700-100700",
            ),
            [
                "Y\t100000\t100900\tY1\t600\t+\t100000\t100900\t255,0,0",
                "Y\t100200\t100700\tY2\t600\t+\t100200\t100700\t255,0,0",
                "Y\t100600\t100700\tY4\t600\t+\t100600\t100700\t255,0,0",
            ]
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
fn csi_tabix_query_returns_large_coordinate_fixture_rows_exactly() {
    unsafe {
        let conf = tbx_conf_vcf();
        let rows = query_tabix_rows(
            "htslib/test/tabix/large_chr.vcf",
            "csi",
            14,
            &conf,
            c"chr20:1-2147483647",
        );

        let mut got = rows.join("\n");
        got.push('\n');
        assert_eq!(
            got,
            include_str!("../htslib/test/tabix/large_chr.20.1.2147483647.out")
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

#[test]
fn generated_large_vcf_reads_all_records_through_vcf_parser() {
    unsafe {
        let record_count = 100_000usize;
        let (bgz, tbi) = build_large_generated_index_vcf(record_count, "large-index-read");
        let bgz_c = c_path(&bgz);
        let fp = hts_open(bgz_c.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());

        let mut seen = 0usize;
        let mut sampled_names = Vec::new();
        while bcf_read(fp, hdr, rec) >= 0 {
            if seen == 0 || seen == record_count / 2 || seen + 1 == record_count {
                sampled_names.push(CStr::from_ptr(bcf_seqname(hdr, rec)).to_bytes().to_vec());
            }
            assert_eq!((*rec).pos, seen as i64);
            seen += 1;
        }
        assert_eq!(seen, record_count);
        assert_eq!(
            sampled_names,
            vec![b"1".to_vec(), b"1".to_vec(), b"1".to_vec()]
        );

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
        let _ = std::fs::remove_file(bgz);
        let _ = std::fs::remove_file(tbi);
    }
}

#[test]
fn generated_large_vcf_tabix_queries_cover_dense_regions_exactly() {
    unsafe {
        let (bgz, tbi) = build_large_generated_index_vcf(120_000, "large-index-query");
        let rows = query_tabix_rows_from_paths(&bgz, &tbi, c"1:40000-50000");
        assert_eq!(rows.len(), 10_001);
        assert!(rows.first().unwrap().starts_with("1\t40000\t"));
        assert!(rows.last().unwrap().starts_with("1\t50000\t"));

        let tail_rows = query_tabix_rows_from_paths(&bgz, &tbi, c"1:119990-120010");
        assert_eq!(tail_rows.len(), 11);
        assert!(tail_rows.first().unwrap().starts_with("1\t119990\t"));
        assert!(tail_rows.last().unwrap().starts_with("1\t120000\t"));

        let empty_rows = query_tabix_rows_from_paths(&bgz, &tbi, c"1:130000-131000");
        assert!(empty_rows.is_empty());

        let _ = std::fs::remove_file(bgz);
        let _ = std::fs::remove_file(tbi);
    }
}

#[test]
fn generated_large_bed_tabix_cli_matches_original_htslib_tabix() {
    unsafe {
        let (bgz, tbi) = build_large_generated_bed(50_000, "large-bed-cli");
        let rust_tabix = std::path::Path::new(env!("CARGO_BIN_EXE_perf_tabix"));
        let c_tabix = fixture("htslib/tabix");

        for region in ["chr2:500000-510000", "chr4:800000-810000"] {
            let rust_stdout = run_tabix_command(rust_tabix, &bgz, region);
            let c_stdout = run_tabix_command(&c_tabix, &bgz, region);
            assert_eq!(rust_stdout, c_stdout, "region {region}");

            let line_count = rust_stdout.iter().filter(|&&ch| ch == b'\n').count();
            assert!(
                line_count >= 20,
                "region {region} returned too few rows: {line_count}"
            );
        }

        let empty_rust = run_tabix_command(rust_tabix, &bgz, "chr4:2000000-2000100");
        let empty_c = run_tabix_command(&c_tabix, &bgz, "chr4:2000000-2000100");
        assert_eq!(empty_rust, empty_c);
        assert!(empty_rust.is_empty());

        let _ = std::fs::remove_file(bgz);
        let _ = std::fs::remove_file(tbi);
    }
}
