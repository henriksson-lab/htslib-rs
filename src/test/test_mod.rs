use crate::htslib_rs::{hts, sam};

// original: code (htslib/test/test_mod.c:76)
pub unsafe fn test_test_mod_c_76_code(id: i32) -> Vec<u8> {
    if id > 0 {
        vec![id as u8]
    } else {
        format!("({})", -id).into_bytes()
    }
}

// original: main (htslib/test/test_mod.c:88)
pub unsafe fn test_test_mod_c_88_main(mut argc: i32, mut argv: *mut *mut u8) -> i32 {
    use std::io::Write;
    let mut __out = std::io::stdout();
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

    let mut extended = 0;
    let mut flags = 0_u32;

    if argc > 1 && libc::strcmp((*argv.add(1)).cast(), c"-x".as_ptr()) == 0 {
        extended = 1;
        argv = argv.add(1);
        argc -= 1;
    }

    if argc > 2 && libc::strcmp((*argv.add(1)).cast(), c"-f".as_ptr()) == 0 {
        flags = libc::atoi((*argv.add(2)).cast()) as u32;
        argv = argv.add(2);
        argc -= 2;
    }

    if argc < 2 {
        return 1;
    }

    let in_ = hts::hts_open((*argv.add(1)).cast(), c"r".as_ptr());
    if in_.is_null() {
        return 1;
    }

    let b = sam::bam_init1();
    let h = sam::sam_hdr_read(in_);
    let mut m = sam::hts_base_mod_state_alloc();
    if h.is_null() || b.is_null() {
        sam::bam_destroy1(b);
        sam::sam_hdr_destroy(h);
        sam::hts_base_mod_state_free(Some(m));
        return if hts::hts_close(in_) != 0 { 1 } else { 2 };
    }

    let mut r;
    loop {
        r = sam::sam_read1(in_, h, b);
        if r < 0 {
            break;
        }

        if sam::bam_parse_basemod2(&*b, &mut m, flags) < 0 {
            eprintln!("Failed to parse MM/ML aux tags");
            sam::bam_destroy1(b);
            sam::sam_hdr_destroy(h);
            sam::hts_base_mod_state_free(Some(m));
            return if hts::hts_close(in_) != 0 { 1 } else { 2 };
        }

        let mut mods = [sam::hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 5];
        let mut i = 0;
        while i < (*b).core.l_qseq {
            let mut sp = b'\t';
            let n = sam::bam_mods_at_next_pos(&*b, &mut m, &mut mods);
            write!(
                __out,
                "{}\t{}",
                i,
                SEQ_NT16_STR[sam::bam_seqi(sam::bam_get_seq(b), i as usize) as usize] as char,
            ).unwrap();
            let mut j = 0;
            while j < n && j < 5 {
                let qstr: Vec<u8> = if mods[j as usize].qual == sam::HTS_MOD_UNCHECKED {
                    vec![b'#']
                } else if mods[j as usize].qual == sam::HTS_MOD_UNKNOWN {
                    vec![b'.']
                } else {
                    format!("{}", mods[j as usize].qual).into_bytes()
                };

                if extended != 0 {
                    let mut m_strand = 0;
                    let mut m_implicit = 0;
                    let mut m_canonical: u8 = 0;
                    let ret = sam::bam_mods_query_type(
                        &m,
                        mods[j as usize].modified_base,
                        Some(&mut m_strand),
                        Some(&mut m_implicit),
                        Some(&mut m_canonical),
                    );
                    if ret < 0
                        || m_canonical as i32 != mods[j as usize].canonical_base
                        || m_strand != mods[j as usize].strand
                    {
                        sam::bam_destroy1(b);
                        sam::sam_hdr_destroy(h);
                        sam::hts_base_mod_state_free(Some(m));
                        return if hts::hts_close(in_) != 0 { 1 } else { 2 };
                    }
                    write!(
                        __out,
                        "{}{}{}{}{}{}",
                        sp as char,
                        mods[j as usize].canonical_base as u8 as char,
                        b"+-"[mods[j as usize].strand as usize] as char,
                        String::from_utf8_lossy(&test_test_mod_c_76_code(mods[j as usize].modified_base)),
                        b"?."[m_implicit as usize] as char,
                        String::from_utf8_lossy(&qstr),
                    ).unwrap();
                } else {
                    write!(
                        __out,
                        "{}{}{}{}{}",
                        sp as char,
                        mods[j as usize].canonical_base as u8 as char,
                        b"+-"[mods[j as usize].strand as usize] as char,
                        String::from_utf8_lossy(&test_test_mod_c_76_code(mods[j as usize].modified_base)),
                        String::from_utf8_lossy(&qstr),
                    ).unwrap();
                }
                sp = b' ';
                j += 1;
            }
            writeln!(__out).unwrap();
            i += 1;
        }

        writeln!(__out, "---").unwrap();

        sam::bam_parse_basemod2(&*b, &mut m, flags);

        let mut all_mods_n = 0;
        let all_mods = sam::bam_mods_recorded(&mut m, &mut all_mods_n).to_vec();
        write!(__out, "Present:").unwrap();
        i = 0;
        while i < all_mods_n {
            let mut m_strand = 0;
            let mut m_implicit = 0;
            let mut m_canonical: u8 = 0;
            sam::bam_mods_queryi(
                &m,
                i,
                Some(&mut m_strand),
                Some(&mut m_implicit),
                Some(&mut m_canonical),
            );
            if all_mods[i as usize] > 0 {
                write!(__out, " {}", all_mods[i as usize] as u8 as char).unwrap();
            } else {
                write!(__out, " #{}", all_mods[i as usize]).unwrap();
            }
            write!(__out, "{}", b"?."[m_implicit as usize] as char).unwrap();
            i += 1;
        }
        writeln!(__out).unwrap();

        let mut pos = 0;
        loop {
            let n = sam::bam_next_basemod(&*b, &mut m, &mut mods, &mut pos);
            if n <= 0 {
                if n < 0 {
                    sam::bam_destroy1(b);
                    sam::sam_hdr_destroy(h);
                    sam::hts_base_mod_state_free(Some(m));
                    return if hts::hts_close(in_) != 0 { 1 } else { 2 };
                }
                break;
            }
            let mut sp = b'\t';
            write!(
                __out,
                "{}\t{}",
                pos,
                SEQ_NT16_STR[sam::bam_seqi(sam::bam_get_seq(b), pos as usize) as usize] as char,
            ).unwrap();
            let mut j = 0;
            while j < n && j < 5 {
                let qstr: Vec<u8> = if mods[j as usize].qual == sam::HTS_MOD_UNCHECKED {
                    vec![b'#']
                } else if mods[j as usize].qual == sam::HTS_MOD_UNKNOWN {
                    vec![b'.']
                } else {
                    format!("{}", mods[j as usize].qual).into_bytes()
                };

                write!(
                    __out,
                    "{}{}{}{}{}",
                    sp as char,
                    mods[j as usize].canonical_base as u8 as char,
                    b"+-"[mods[j as usize].strand as usize] as char,
                    String::from_utf8_lossy(&test_test_mod_c_76_code(mods[j as usize].modified_base)),
                    String::from_utf8_lossy(&qstr),
                ).unwrap();
                sp = b' ';
                j += 1;
            }
            writeln!(__out).unwrap();
        }

        writeln!(__out, "\n===\n").unwrap();
    }

    let _ = __out.flush();
    let mut ret = 0;
    if hts::hts_close(in_) != 0 || r < -1 {
        ret = 1;
    }

    sam::bam_destroy1(b);
    sam::sam_hdr_destroy(h);
    sam::hts_base_mod_state_free(Some(m));
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn fixture(path: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
    }

    fn c_path(path: &Path) -> Vec<u8> {
        let mut v = path.to_string_lossy().as_bytes().to_vec();
        v.push(0);
        v
    }

    fn temp_output(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "htslib-rs-test-mod-main-{label}-{}-{}.out",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ))
    }

    unsafe fn run_main_capture_stdout(args: &mut [Vec<u8>], out_path: &Path) -> i32 {
        let _ = std::fs::remove_file(out_path);
        libc::fflush(std::ptr::null_mut());
        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");

        if pid == 0 {
            let out_c = c_path(out_path);
            let out_fd = libc::open(
                out_c.as_ptr().cast(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o600,
            );
            if out_fd < 0 {
                libc::_exit(libc::EXIT_FAILURE);
            }
            if libc::dup2(out_fd, libc::STDOUT_FILENO) < 0 {
                libc::close(out_fd);
                libc::_exit(libc::EXIT_FAILURE);
            }
            libc::close(out_fd);

            let mut argv = args
                .iter_mut()
                .map(|arg| arg.as_mut_ptr())
                .collect::<Vec<_>>();
            let ret = test_test_mod_c_88_main(argv.len() as i32, argv.as_mut_ptr());
            libc::fflush(std::ptr::null_mut());
            libc::_exit(ret);
        }

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status), "child did not exit normally");
        libc::WEXITSTATUS(status)
    }

    fn assert_main_output(label: &str, mut args: Vec<Vec<u8>>, expected: &str) {
        let out = temp_output(label);
        unsafe {
            assert_eq!(run_main_capture_stdout(&mut args, &out), 0);
        }
        let actual = std::fs::read_to_string(&out).unwrap();
        let expected = std::fs::read_to_string(fixture(expected)).unwrap();
        assert_eq!(actual, expected);
        let _ = std::fs::remove_file(out);
    }

    #[test]
    fn original_test_mod_main_matches_base_mod_fixture_outputs() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert_main_output(
            "chebi-default",
            vec![
                b"test_mod\0".to_vec(),
                c_path(&fixture("htslib/test/base_mods/MM-chebi.sam")),
            ],
            "htslib/test/base_mods/MM-chebi.out",
        );
        assert_main_output(
            "explicit-extended",
            vec![
                b"test_mod\0".to_vec(),
                b"-x\0".to_vec(),
                c_path(&fixture("htslib/test/base_mods/MM-explicit.sam")),
            ],
            "htslib/test/base_mods/MM-explicit-x.out",
        );
        assert_main_output(
            "explicit-flags",
            vec![
                b"test_mod\0".to_vec(),
                b"-f\0".to_vec(),
                b"1\0".to_vec(),
                c_path(&fixture("htslib/test/base_mods/MM-explicit.sam")),
            ],
            "htslib/test/base_mods/MM-explicit-f.out",
        );
    }
}
