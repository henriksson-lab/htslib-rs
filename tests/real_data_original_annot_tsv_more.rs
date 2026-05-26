mod htslib_rs {
    pub use ::htslib_rs::*;
}

#[path = "../src/annot_tsv.rs"]
mod annot_tsv;

use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static ANNOT_TSV_LOCK: Mutex<()> = Mutex::new(());

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn run_annot_tsv(src: &str, dst: &str, args: &[&str]) -> String {
    let out = std::env::temp_dir().join(format!(
        "htslib_rs-annot-tsv-{}-{}.txt",
        std::process::id(),
        args.join("_")
            .bytes()
            .map(|b| if b.is_ascii_alphanumeric() {
                b as char
            } else {
                '_'
            })
            .collect::<String>()
    ));

    let src = fixture(&format!("htslib/test/annot-tsv/{src}"));
    let dst = fixture(&format!("htslib/test/annot-tsv/{dst}"));

    let mut argv = vec![
        CString::new("annot-tsv").unwrap(),
        CString::new("-s").unwrap(),
        CString::new(src.to_string_lossy().as_bytes()).unwrap(),
        CString::new("-t").unwrap(),
        CString::new(dst.to_string_lossy().as_bytes()).unwrap(),
        CString::new("-o").unwrap(),
        CString::new(out.to_string_lossy().as_bytes()).unwrap(),
    ];
    argv.extend(args.iter().map(|arg| CString::new(*arg).unwrap()));
    let mut ptrs = argv
        .iter()
        .map(|arg| arg.as_ptr().cast_mut())
        .collect::<Vec<_>>();

    let _guard = ANNOT_TSV_LOCK.lock().unwrap();
    let ret =
        unsafe { annot_tsv::annot_tsv_c_956_main(ptrs.len() as libc::c_int, ptrs.as_mut_ptr()) };
    assert_eq!(ret, 0);
    std::fs::read_to_string(out).unwrap()
}

fn assert_fixture(src: &str, dst: &str, out: &str, args: &[&str]) {
    let actual = run_annot_tsv(src, dst, args);
    let expected =
        std::fs::read_to_string(fixture(&format!("htslib/test/annot-tsv/{out}"))).unwrap();
    assert_eq!(actual, expected, "{out}");
}

#[test]
fn annot_tsv_priority_fixtures_match_exact_expected_output() {
    assert_fixture(
        "src.1.txt",
        "dst.1.txt",
        "out.1.1.txt",
        &["-f", "smpl:overlap", "--allow-dups"],
    );
    assert_fixture(
        "src.1.txt",
        "dst.1.txt",
        "out.1.6.txt",
        &["-f", "smpl:overlap", "--allow-dups", "--max-annots", "2"],
    );
    assert_fixture(
        "src.1.txt",
        "dst.1.txt",
        "out.1.5.txt",
        &["-f", "smpl:overlap", "-rO", "0.5"],
    );
    assert_fixture(
        "src.9.txt",
        "dst.9.txt",
        "out.9.1.txt",
        &["-c", "1,2,3:chr,beg,end", "-a", "nbp,frac,cnt"],
    );

    assert_fixture("src.10.txt", "dst.10.txt", "out.10.1.txt", &["-f", "smpl"]);
    assert_fixture("src.10.txt", "dst.10.txt", "out.10.2.txt", &[]);
    assert_fixture("src.10.txt", "dst.10.txt", "out.10.3.txt", &["-x"]);
    assert_fixture(
        "src.10.txt",
        "dst.10.txt",
        "out.10.4.txt",
        &["-m", "smpl", "-f", "smpl"],
    );
    assert_fixture("src.10.txt", "dst.10.txt", "out.10.5.txt", &["-m", "smpl"]);
    assert_fixture(
        "src.10.txt",
        "dst.10.txt",
        "out.10.6.txt",
        &["-m", "smpl", "-x"],
    );

    assert_fixture(
        "src.14.txt",
        "dst.14.txt",
        "out.14.1.txt",
        &["-c", "1,2,3", "-C", "11:11"],
    );
    assert_fixture(
        "src.14.txt",
        "dst.14.txt",
        "out.14.2.txt",
        &["-c", "1,2,3", "-C", "01:01"],
    );
}

#[test]
fn annot_tsv_missing_htslib_fixtures_match_exact_expected_output() {
    let cases: &[(&str, &str, &str, &[&str])] = &[
        (
            "src.2.txt",
            "dst.2.txt",
            "out.2.1.txt",
            &["-c", "1,2,3:1,2,3", "-f", "4:5", "--allow-dups"],
        ),
        (
            "src.2.txt",
            "dst.2.txt",
            "out.2.2.txt",
            &["-c", "1,2,3:1,2,3", "-f", "4:5"],
        ),
        (
            "src.2.txt",
            "dst.2.txt",
            "out.2.3.txt",
            &["-c", "1,2,3:1,2,3", "-f", "4,value:5,value"],
        ),
        (
            "src.2.txt",
            "dst.2.txt",
            "out.2.4.txt",
            &["-c", "1,2,3:1,2,3", "-f", "value,4:value,5"],
        ),
        (
            "src.2.txt",
            "dst.2.txt",
            "out.2.5.txt",
            &[
                "-c",
                "1,2,3:1,2,3",
                "-f",
                "value,4:value,5",
                "-a",
                "nbp,frac",
            ],
        ),
        (
            "src.2.txt",
            "dst.2.txt",
            "out.2.6.txt",
            &[
                "-c",
                "1,2,3:1,2,3",
                "-f",
                "4:5",
                "--allow-dups",
                "--max-annots",
                "2",
            ],
        ),
        (
            "src.3.txt",
            "dst.3.txt",
            "out.3.1.txt",
            &["-f", "smpl:overlap", "-a", "nbp,frac"],
        ),
        (
            "src.4.txt",
            "dst.4.txt",
            "out.4.1.txt",
            &[
                "-c",
                "2,3,4:2,3,4",
                "-m",
                "1:1",
                "-f",
                "1:1",
                "-a",
                "nbp,frac",
            ],
        ),
        (
            "src.5.txt",
            "dst.5.txt",
            "out.5.1.txt",
            &["-c", "2,3,4:2,3,4", "-a", "nbp,frac"],
        ),
        (
            "src.6.txt",
            "dst.6.txt",
            "out.6.1.txt",
            &["-c", "1,2,2:1,2,2", "-a", "nbp"],
        ),
        (
            "src.7.txt",
            "dst.7.txt",
            "out.7.1.txt",
            &["-c", "1,2,2:1,2,2", "-f", "overlap", "-H"],
        ),
        (
            "src.8.txt",
            "dst.8.txt",
            "out.8.1.txt",
            &[
                "-c",
                "chr,beg,end:chr,start,end",
                "-m",
                "sample",
                "-f",
                "is_tp",
            ],
        ),
        (
            "src.11.txt",
            "dst.11.txt",
            "out.11.1.txt",
            &["-c", "1,2,3:1,2,3", "-f", "4:5", "-h", "0:0"],
        ),
        (
            "src.11.txt",
            "dst.11.txt",
            "out.11.1.txt",
            &[
                "-c",
                "chr1,beg1,end1:chr,beg,end",
                "-f",
                "smpl1:src_smpl",
                "-h",
                "2:2",
                "-II",
            ],
        ),
        (
            "src.11.txt",
            "dst.11.txt",
            "out.11.1.txt",
            &[
                "-c",
                "chr1,beg1,end1:chr,beg,end",
                "-f",
                "smpl1:src_smpl",
                "-h",
                "2:-1",
                "-I",
                "-I",
            ],
        ),
        (
            "src.11.txt",
            "dst.11.txt",
            "out.11.2.txt",
            &[
                "-c",
                "chr1,beg1,end1:chr,beg,end",
                "-f",
                "smpl1:src_smpl",
                "-h",
                "2:2",
            ],
        ),
        (
            "src.11.txt",
            "dst.11.txt",
            "out.11.2.txt",
            &[
                "-c",
                "chr2,beg2,end2:chr,beg,end",
                "-f",
                "smpl2:src_smpl",
                "-h",
                "3:2",
            ],
        ),
        (
            "src.11.txt",
            "dst.11.txt",
            "out.11.3.txt",
            &[
                "-c",
                "chr1,beg1,end1:chr,beg,end",
                "-f",
                "smpl1:src_smpl",
                "-h",
                "2:2",
                "-I",
            ],
        ),
        (
            "src.11.txt",
            "dst.11.txt",
            "out.11.3.txt",
            &[
                "-c",
                "chr2,beg2,end2:chr,beg,end",
                "-f",
                "smpl2:src_smpl",
                "-h",
                "3:2",
                "-I",
            ],
        ),
        (
            "src.12.txt",
            "dst.12.txt",
            "out.12.1.txt",
            &["-c", "1,2,3:1,2,3", "-f", "4:5", "-h", "0:0", "-d", ","],
        ),
        (
            "src.13.txt",
            "src.13.txt",
            "out.13.1.txt",
            &["-c", "1,2,3", "-f", "4:5"],
        ),
        (
            "src.13.txt",
            "src.13.txt",
            "out.13.1.txt",
            &["-c", "1,2,3", "-f", "4:5", "-O", "0.5"],
        ),
        (
            "src.13.txt",
            "src.13.txt",
            "out.13.2.txt",
            &["-c", "1,2,3", "-f", "4:5", "-O", "0.5", "-r"],
        ),
        (
            "src.13.txt",
            "src.13.txt",
            "out.13.2.txt",
            &["-c", "1,2,3", "-f", "4:5", "-O", "0.5,0.5"],
        ),
        (
            "src.13.txt",
            "src.13.txt",
            "out.13.3.txt",
            &["-c", "1,2,3", "-f", "4:5", "-O", "0,1"],
        ),
        (
            "src.13.txt",
            "src.13.txt",
            "out.13.4.txt",
            &["-c", "1,2,3", "-f", "4:5", "-O", "1,0"],
        ),
    ];

    for (src, dst, out, args) in cases {
        assert_fixture(src, dst, out, args);
    }
}
