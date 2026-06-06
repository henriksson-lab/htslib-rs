// Functions translated from htslib/synced_bcf_reader.c.
// Extracted from src/vcf.rs.

use std::ffi::{c_char, c_int, c_void};
use std::ptr::NonNull;

use crate::htslib_rs::hts::{
    htsFile, hts_close, hts_open, hts_pos_t, kstring_t, HTS_FORMAT_VCF, KS_SEP_LINE,
};
use crate::htslib_rs::vcf::*;

// (extracted functions in src/vcf.rs order)

pub unsafe fn synced_bcf_reader_c_1070_regions_merge(reg: *mut c_void) {
    unsafe { regions_merge(reg.cast::<BcfSrRegion>()) }
}

pub unsafe fn synced_bcf_reader_c_1085__regions_sort_and_merge(reg: *mut bcf_sr_regions_t) {
    unsafe { regions_sort_and_merge(reg) }
}

pub unsafe fn bcf_sr_add_hreader(
    readers: *mut bcf_srs_t,
    file_ptr: *mut htsFile,
    autoclose: c_int,
    idxname: *const c_char,
) -> c_int {
    unsafe {
        // Defensive guards (Rust-safety nicety not present in the C library,
        // which would dereference a NULL readers pointer): bail out cleanly.
        if readers.is_null() {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            return 0;
        }
        if file_ptr.is_null() {
            (*readers).errnum = bcf_sr_error_api_usage_error;
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            return 0;
        }
        bcf_sr_add_hreader_impl(readers, file_ptr, autoclose, idxname)
    }
}

pub unsafe fn bcf_sr_init() -> *mut bcf_srs_t {
    unsafe {
        let mut files = Box::new(std::mem::zeroed::<bcf_srs_t>());
        let aux = Box::new(std::mem::zeroed::<BcfSrAux>());
        files.aux = Box::into_raw(aux).cast::<c_void>();
        let files = Box::into_raw(files);
        bcf_sr_sort_c_675_bcf_sr_sort_init(&mut (*bcf_sr_aux_mut(files)).sort);
        bcf_sr_set_opt(files, BCF_SR_REGIONS_OVERLAP, 1);
        bcf_sr_set_opt(files, BCF_SR_TARGETS_OVERLAP, 0);
        files
    }
}

pub unsafe fn synced_bcf_reader_c_461_bcf_sr_destroy1(reader: *mut bcf_sr_t, closefile: c_int) {
    unsafe { bcf_sr_destroy1(reader, closefile) }
}

pub unsafe fn bcf_sr_destroy(files: *mut bcf_srs_t) {
    unsafe {
        let Some(files_ptr) = NonNull::new(files) else {
            return;
        };
        let files = files_ptr.as_ptr();
        let aux = bcf_sr_aux_mut(files);
        let autoclose = if aux.is_null() {
            std::ptr::null_mut()
        } else {
            (*aux).closefile
        };
        let nreaders = (*files).nreaders as usize;
        for i in 0..nreaders {
            let cf = if autoclose.is_null() {
                0
            } else {
                *autoclose.add(i)
            };
            bcf_sr_destroy1((*files).readers.add(i), cf);
        }
        libc::free((*files).has_line.cast());
        libc::free((*files).readers.cast());
        let samples = if (*files).samples.is_null() {
            None
        } else {
            Some(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                (*files).samples,
                (*files).n_smpl as usize,
            )))
        };
        if let Some(samples) = samples {
            for sample in samples.iter().copied() {
                libc::free(sample.cast());
            }
        }
        if !(*files).targets.is_null() {
            bcf_sr_regions_destroy((*files).targets);
        }
        if !(*files).regions.is_null() {
            bcf_sr_regions_destroy((*files).regions);
        }
        if (*files).tmps.m != 0 {
            libc::free((*files).tmps.s.cast());
        }
        if (*files).n_threads != 0 {
            bcf_sr_destroy_threads(files);
        }
        if !aux.is_null() {
            bcf_sr_sort_c_685_bcf_sr_sort_destroy(&mut (*aux).sort);
        }
        libc::free(autoclose.cast());
        if !(*files).aux.is_null() {
            drop(Box::from_raw((*files).aux.cast::<BcfSrAux>()));
        }
        drop(Box::from_raw(files));
    }
}

pub unsafe fn bcf_sr_strerror(errnum: c_int) -> *mut c_char {
    match errnum {
        x if x == bcf_sr_error_open_failed as c_int => unsafe {
            libc::strerror(*crate::htslib_rs::c_compat::__errno_location())
        },
        x if x == bcf_sr_error_not_bgzf as c_int => {
            c"not compressed with bgzip".as_ptr().cast_mut()
        }
        x if x == bcf_sr_error_idx_load_failed as c_int => {
            c"could not load index".as_ptr().cast_mut()
        }
        x if x == bcf_sr_error_file_type_error as c_int => c"unknown file type".as_ptr().cast_mut(),
        x if x == bcf_sr_error_api_usage_error as c_int => c"API usage error".as_ptr().cast_mut(),
        x if x == bcf_sr_error_header_error as c_int => {
            c"could not parse header".as_ptr().cast_mut()
        }
        x if x == bcf_sr_error_no_eof as c_int => c"no BGZF EOF marker; file may be truncated"
            .as_ptr()
            .cast_mut(),
        x if x == bcf_sr_error_no_memory as c_int => c"Out of memory".as_ptr().cast_mut(),
        x if x == bcf_sr_error_vcf_parse_error as c_int => c"VCF parse error".as_ptr().cast_mut(),
        x if x == bcf_sr_error_bcf_read_error as c_int => c"BCF read error".as_ptr().cast_mut(),
        BCF_SR_ERROR_NOIDX_ERROR => c"merge of unindexed files failed".as_ptr().cast_mut(),
        _ => c"".as_ptr().cast_mut(),
    }
}

// original: bcf_sr_set_threads (htslib/synced_bcf_reader.c:228)
pub unsafe fn bcf_sr_set_threads(files: *mut bcf_srs_t, n_threads: c_int) -> c_int {
    unsafe {
        (*files).n_threads = n_threads;
        if n_threads == 0 {
            return 0;
        }
        let mut thread_pool = Box::new(std::mem::zeroed::<crate::hts::htsThreadPool>());
        let p = thread_pool.as_mut() as *mut crate::hts::htsThreadPool;
        (*p).pool = crate::thread_pool::hts_tpool_init(n_threads);
        if (*p).pool.is_null() {
            (*files).errnum = bcf_sr_error_no_memory;
            return -1;
        }
        (*files).p = Box::into_raw(thread_pool);
        0
    }
}

pub unsafe fn bcf_sr_set_opt_require_idx(readers: *mut bcf_srs_t) -> c_int {
    unsafe {
        (*readers).require_index = REQUIRE_IDX_;
    }
    0
}

pub unsafe fn bcf_sr_set_opt_allow_no_idx(readers: *mut bcf_srs_t) -> c_int {
    unsafe {
        (*readers).require_index = ALLOW_NO_IDX_;
    }
    0
}

pub unsafe fn bcf_sr_set_opt_pair_logic(readers: *mut bcf_srs_t, pair_logic: c_int) -> c_int {
    unsafe {
        (*bcf_sr_aux_mut(readers)).sort.pair = pair_logic;
    }
    0
}

pub unsafe fn bcf_sr_set_opt_regions_overlap(readers: *mut bcf_srs_t, overlap: c_int) -> c_int {
    unsafe {
        (*bcf_sr_aux_mut(readers)).regions_overlap = overlap;
        if !(*readers).regions.is_null() {
            bcf_sr_regions_set_overlap((*readers).regions, overlap);
        }
    }
    0
}

pub unsafe fn bcf_sr_set_opt_targets_overlap(readers: *mut bcf_srs_t, overlap: c_int) -> c_int {
    unsafe {
        (*bcf_sr_aux_mut(readers)).targets_overlap = overlap;
        if !(*readers).targets.is_null() {
            bcf_sr_regions_set_overlap((*readers).targets, overlap);
        }
    }
    0
}

pub unsafe fn bcf_sr_set_opt(readers: *mut bcf_srs_t, opt: bcf_sr_opt_t, value: c_int) -> c_int {
    match opt {
        BCF_SR_REQUIRE_IDX => unsafe { bcf_sr_set_opt_require_idx(readers) },
        BCF_SR_ALLOW_NO_IDX => unsafe { bcf_sr_set_opt_allow_no_idx(readers) },
        BCF_SR_PAIR_LOGIC => unsafe { bcf_sr_set_opt_pair_logic(readers, value) },
        BCF_SR_REGIONS_OVERLAP => unsafe { bcf_sr_set_opt_regions_overlap(readers, value) },
        BCF_SR_TARGETS_OVERLAP => unsafe { bcf_sr_set_opt_targets_overlap(readers, value) },
        _ => 1,
    }
}

// original: bcf_sr_destroy_threads (htslib/synced_bcf_reader.c:244)
pub unsafe fn bcf_sr_destroy_threads(files: *mut bcf_srs_t) {
    unsafe {
        let Some(p) = NonNull::new((*files).p.cast::<crate::hts::htsThreadPool>()) else {
            return;
        };
        let p = p.as_ptr();
        if !(*p).pool.is_null() {
            crate::thread_pool::hts_tpool_destroy((*p).pool);
        }
        drop(Box::from_raw(p));
        (*files).p = std::ptr::null_mut();
    }
}

pub unsafe fn bcf_sr_add_reader(files: *mut bcf_srs_t, fname: *const c_char) -> c_int {
    unsafe {
        let mut fmode = [0 as c_char; 5];
        fmode[0] = b'r' as c_char;
        vcf_open_mode(fmode.as_mut_ptr().add(1), fname, std::ptr::null());
        let file_ptr = hts_open(fname, fmode.as_ptr());
        if file_ptr.is_null() {
            (*files).errnum = bcf_sr_error_open_failed;
            return 0;
        }
        // get idx name and pass to add_hreader
        let mut needle = [0u8; 8];
        needle[..HTS_IDX_DELIM.len()].copy_from_slice(HTS_IDX_DELIM);
        let mut idxname = libc::strstr(fname, needle.as_ptr().cast());
        if !idxname.is_null() {
            idxname = idxname.add(HTS_IDX_DELIM.len());
        }
        let ret = bcf_sr_add_hreader_impl(files, file_ptr, 1, idxname);
        if ret == 0 {
            let _ = hts_close(file_ptr);
        }
        ret
    }
}

// original: bcf_sr_remove_reader (htslib/synced_bcf_reader.c:504)
pub unsafe fn bcf_sr_remove_reader(files: *mut bcf_srs_t, i: c_int) {
    unsafe {
        // assert( !files->samples );  // not ready for this yet
        let autoclose = (*bcf_sr_aux_mut(files)).closefile;

        bcf_sr_sort_c_662_bcf_sr_sort_remove_reader(files, &mut (*bcf_sr_aux_mut(files)).sort, i);
        let cf = if autoclose.is_null() {
            0
        } else {
            *autoclose.add(i as usize)
        };
        bcf_sr_destroy1((*files).readers.add(i as usize), cf);
        if i + 1 < (*files).nreaders {
            let n = ((*files).nreaders - i - 1) as usize;
            std::ptr::copy(
                (*files).readers.add((i + 1) as usize),
                (*files).readers.add(i as usize),
                n,
            );
            std::ptr::copy(
                (*files).has_line.add((i + 1) as usize),
                (*files).has_line.add(i as usize),
                n,
            );
            if !autoclose.is_null() {
                std::ptr::copy(
                    autoclose.add((i + 1) as usize),
                    autoclose.add(i as usize),
                    n,
                );
            }
        }
        (*files).nreaders -= 1;
    }
}

// original: bcf_sr_next_line (htslib/synced_bcf_reader.c:869)
pub unsafe fn bcf_sr_next_line(files: *mut bcf_srs_t) -> c_int {
    unsafe {
        if (*files).targets_als == 0 {
            return sr_next_line(files);
        }

        loop {
            let ret = sr_next_line(files);
            if ret == 0 {
                return ret;
            }

            let mut i = 0;
            while i < (*files).nreaders {
                if *(*files).has_line.add(i as usize) != 0 {
                    break;
                }
                i += 1;
            }

            if sr_regions_match_alleles(
                (*files).targets,
                (*files).targets_als - 1,
                *(*(*files).readers.add(i as usize)).buffer,
            ) != 0
            {
                return ret;
            }

            // Check if there are more duplicate lines in the buffers. If not,
            // return this line even if there is a type mismatch.
            i = 0;
            while i < (*files).nreaders {
                if *(*files).has_line.add(i as usize) == 0 {
                    i += 1;
                    continue;
                }
                let r = (*files).readers.add(i as usize);
                if (*r).nbuffer == 0 || (*(*(*r).buffer.add(1))).pos != (*(*(*r).buffer)).pos {
                    i += 1;
                    continue;
                }
                break;
            }
            if i == (*files).nreaders {
                return ret;
            }
        }
    }
}

pub unsafe fn bcf_sr_has_line(readers: *mut bcf_srs_t, i: c_int) -> c_int {
    if readers.is_null() || i < 0 || i >= (*readers).nreaders || (*readers).has_line.is_null() {
        return 0;
    }
    *(*readers).has_line.add(i as usize)
}

pub unsafe fn bcf_sr_get_line(readers: *mut bcf_srs_t, i: c_int) -> *mut bcf1_t {
    if bcf_sr_has_line(readers, i) == 0 || (*readers).readers.is_null() {
        return std::ptr::null_mut();
    }
    let reader = (*readers).readers.add(i as usize);
    if (*reader).buffer.is_null() {
        return std::ptr::null_mut();
    }
    *(*reader).buffer
}

pub unsafe fn bcf_sr_get_header(readers: *mut bcf_srs_t, i: c_int) -> *mut bcf_hdr_t {
    if readers.is_null() || i < 0 || i >= (*readers).nreaders || (*readers).readers.is_null() {
        return std::ptr::null_mut();
    }
    (*(*readers).readers.add(i as usize)).header
}

// original: bcf_sr_seek (htslib/synced_bcf_reader.c:911)
pub unsafe fn bcf_sr_seek(readers: *mut bcf_srs_t, seq: *const c_char, pos: hts_pos_t) -> c_int {
    unsafe {
        if (*readers).regions.is_null() {
            return 0;
        }
        bcf_sr_sort_c_681_bcf_sr_sort_reset(&mut (*bcf_sr_aux_mut(readers)).sort);
        if seq.is_null() && pos == 0 {
            bcf_sr_seek_start(readers);
            return 0;
        }

        bcf_sr_seek_start(readers);
        let mut i = -1;
        if crate::sam::khash_str2int_get((*(*readers).regions).seq_hash, seq, &mut i) >= 0 {
            (*(*readers).regions).iseq = i;
        }
        bcf_sr_regions_overlap_inner((*readers).regions, seq, pos, pos, 0);

        let mut nret = 0;
        for j in 0..(*readers).nreaders as usize {
            nret += sr_reader_seek((*readers).readers.add(j), seq, pos, MAX_CSI_COOR - 1);
        }
        nret
    }
}

// original: bcf_sr_set_samples (htslib/synced_bcf_reader.c:940)
pub unsafe fn bcf_sr_set_samples(
    files: *mut bcf_srs_t,
    fname: *const c_char,
    is_file: c_int,
) -> c_int {
    unsafe {
        let mut nsmpl = 0;
        let mut free_smpl = 0;
        let mut smpl: *mut *mut c_char = std::ptr::null_mut();

        let exclude = if *fname == b'^' as c_char {
            crate::sam::khash_str2int_init()
        } else {
            std::ptr::null_mut()
        };
        if !exclude.is_null() || libc::strcmp(c"-".as_ptr(), fname) != 0 {
            smpl = crate::hts::hts_readlist(fname, is_file, &mut nsmpl);
            if smpl.is_null() {
                return 0;
            }
            if !exclude.is_null() {
                for i in 0..nsmpl as usize {
                    crate::sam::khash_str2int_inc(exclude, *smpl.add(i));
                }
            }
            free_smpl = 1;
        }
        if smpl.is_null() {
            smpl = (*(*(*files).readers).header).samples;
            nsmpl = bcf_hdr_nsamples_native((*(*files).readers).header);
        }

        let mut samples = Vec::new();
        for i in 0..nsmpl as usize {
            if !exclude.is_null() && crate::sam::khash_str2int_has_key(exclude, *smpl.add(i)) != 0 {
                continue;
            }
            let mut n_isec = 0;
            for j in 0..(*files).nreaders as usize {
                if bcf_hdr_id2int(
                    (*(*files).readers.add(j)).header,
                    BCF_DT_SAMPLE as c_int,
                    *smpl.add(i),
                ) < 0
                {
                    break;
                }
                n_isec += 1;
            }
            if n_isec != (*files).nreaders {
                continue;
            }
            samples.push(libc::strdup(*smpl.add(i)));
        }

        if !exclude.is_null() {
            crate::sam::khash_str2int_destroy(exclude);
        }
        if free_smpl != 0 {
            for i in 0..nsmpl as usize {
                libc::free((*smpl.add(i)).cast());
            }
            libc::free(smpl.cast());
        }

        if samples.is_empty() {
            return 0;
        }
        (*files).n_smpl = samples.len() as c_int;
        let mut samples = samples.into_boxed_slice();
        (*files).samples = samples.as_mut_ptr();
        std::mem::forget(samples);

        for i in 0..(*files).nreaders as usize {
            let reader = (*files).readers.add(i);
            (*reader).samples =
                libc::malloc((*files).n_smpl as usize * size_of::<c_int>()).cast::<c_int>();
            (*reader).n_smpl = (*files).n_smpl;
            for j in 0..(*files).n_smpl as usize {
                *(*reader).samples.add(j) = bcf_hdr_id2int(
                    (*reader).header,
                    BCF_DT_SAMPLE as c_int,
                    *(*files).samples.add(j),
                );
            }
        }
        1
    }
}

// original: bcf_sr_set_targets (htslib/synced_bcf_reader.c:209)
pub unsafe fn bcf_sr_set_targets(
    readers: *mut bcf_srs_t,
    mut targets: *const c_char,
    is_file: c_int,
    alleles: c_int,
) -> c_int {
    unsafe {
        if (*readers).nreaders != 0 || !(*readers).targets.is_null() {
            return -1;
        }
        if *targets == b'^' as c_char {
            (*readers).targets_exclude = 1;
            targets = targets.add(1);
        }
        (*readers).targets = bcf_sr_regions_init(targets, is_file, 0, 1, -2);
        if (*readers).targets.is_null() {
            return -1;
        }
        (*readers).targets_als = alleles;
        bcf_sr_regions_set_overlap(
            (*readers).targets,
            (*bcf_sr_aux_mut(readers)).targets_overlap,
        );
        0
    }
}

// original: bcf_sr_set_regions (htslib/synced_bcf_reader.c:191)
pub unsafe fn bcf_sr_set_regions(
    readers: *mut bcf_srs_t,
    regions: *const c_char,
    is_file: c_int,
) -> c_int {
    unsafe {
        if (*readers).nreaders != 0 || !(*readers).regions.is_null() {
            if !(*readers).regions.is_null() {
                bcf_sr_regions_destroy((*readers).regions);
            }
            (*readers).regions = bcf_sr_regions_init(regions, is_file, 0, 1, -2);
            bcf_sr_seek_start(readers);
            return 0;
        }

        (*readers).regions = bcf_sr_regions_init(regions, is_file, 0, 1, -2);
        if (*readers).regions.is_null() {
            return -1;
        }
        (*readers).explicit_regs = 1;
        (*readers).require_index = REQUIRE_IDX_;
        bcf_sr_regions_set_overlap(
            (*readers).regions,
            (*bcf_sr_aux_mut(readers)).regions_overlap,
        );
        0
    }
}

// original: bcf_sr_regions_init (htslib/synced_bcf_reader.c:1248)
pub unsafe fn bcf_sr_regions_init(
    regions: *const c_char,
    is_file: c_int,
    ichr: c_int,
    ifrom: c_int,
    mut ito: c_int,
) -> *mut bcf_sr_regions_t {
    unsafe {
        if is_file == 0 {
            let reg = regions_init_string(regions);
            regions_sort_and_merge(reg);
            return reg;
        }

        let Some(reg) = NonNull::new(bcf_sr_regions_alloc()) else {
            return std::ptr::null_mut();
        };
        let reg = reg.as_ptr();

        (*reg).file = hts_open(regions, c"rb".as_ptr()).cast();
        if (*reg).file.is_null() {
            bcf_sr_regions_destroy(reg);
            return std::ptr::null_mut();
        }

        (*reg).tbx = crate::tbx::tbx_index_load3(
            regions,
            std::ptr::null(),
            crate::hts::HTS_IDX_SAVE_REMOTE | crate::hts::HTS_IDX_SILENT_FAIL,
        )
        .cast();
        if (*reg).tbx.is_null() {
            let len = libc::strlen(regions) as isize;
            let mut is_bed = if libc::strcasecmp(regions.offset(len - 4), c".bed".as_ptr()) != 0 {
                0
            } else {
                1
            };
            if is_bed == 0 && libc::strcasecmp(regions.offset(len - 7), c".bed.gz".as_ptr()) == 0 {
                is_bed = 1;
            }

            let rfile: *mut htsFile = (*reg).file.cast();
            let line_ptr: *mut kstring_t = (&raw mut (*reg).line).cast();
            if (*rfile).format.format == HTS_FORMAT_VCF {
                ito = 1;
            }

            loop {
                if crate::hts::hts_getline(rfile, KS_SEP_LINE as c_int, line_ptr) <= 0 {
                    break;
                }
                let mut chr: *mut c_char = std::ptr::null_mut();
                let mut chr_end: *mut c_char = std::ptr::null_mut();
                let mut from: hts_pos_t = 0;
                let mut to: hts_pos_t = 0;
                let mut ret = regions_parse_line(
                    (*reg).line.s,
                    ichr,
                    ifrom,
                    ito.abs(),
                    &mut chr,
                    &mut chr_end,
                    &mut from,
                    &mut to,
                );
                if ret < 0 {
                    if ito < 0 {
                        ret = regions_parse_line(
                            (*reg).line.s,
                            ichr,
                            ifrom,
                            ifrom,
                            &mut chr,
                            &mut chr_end,
                            &mut from,
                            &mut to,
                        );
                    }
                    if ret < 0 {
                        bcf_sr_regions_destroy(reg);
                        return std::ptr::null_mut();
                    }
                    ito = ifrom;
                } else if ito < 0 {
                    ito = ito.abs();
                }
                if ret == 0 {
                    continue;
                }
                if is_bed != 0 {
                    from += 1;
                }
                *chr_end = 0;
                bcf_sr_regions_add(reg, chr, from, to);
                *chr_end = b'\t' as c_char;
            }
            let _ = hts_close((*reg).file.cast());
            (*reg).file = std::ptr::null_mut();
            if (*reg).nseqs == 0 {
                bcf_sr_regions_destroy(reg);
                return std::ptr::null_mut();
            }
            regions_sort_and_merge(reg);
            return reg;
        }

        (*reg).seq_names =
            crate::tbx::tbx_seqnames((*reg).tbx.cast(), &mut (*reg).nseqs).cast::<*mut c_char>();
        if (*reg).seq_hash.is_null() {
            (*reg).seq_hash = crate::sam::khash_str2int_init();
        }
        for i in 0..(*reg).nseqs as usize {
            crate::sam::khash_str2int_set((*reg).seq_hash, *(*reg).seq_names.add(i), i as c_int);
        }
        (*reg).fname = libc::strdup(regions);
        (*reg).is_bin = 1;
        reg
    }
}

pub unsafe fn bcf_sr_regions_destroy(regions: *mut bcf_sr_regions_t) {
    unsafe { bcf_sr_regions_destroy_translated(regions) }
}

// original: bcf_sr_regions_seek (htslib/synced_bcf_reader.c:1352)
pub unsafe fn bcf_sr_regions_seek(reg: *mut bcf_sr_regions_t, seq: *const c_char) -> c_int {
    unsafe {
        if reg.is_null() || seq.is_null() {
            return -1;
        }
        (*reg).iseq = -1;
        (*reg).start = -1;
        (*reg).end = -1;
        if crate::sam::khash_str2int_get((*reg).seq_hash, seq, &mut (*reg).iseq) < 0 {
            return -1; // sequence seq not in regions
        }

        // using in-memory regions
        if !(*reg).regs.is_null() {
            (*(*reg).regs.cast::<BcfSrRegion>().add((*reg).iseq as usize)).creg = -1;
            return 0;
        }

        // reading regions from tabix
        if !(*reg).itr.is_null() {
            crate::hts::hts_itr_destroy((*reg).itr.cast());
        }
        (*reg).itr = crate::tbx::tbx_itr_querys1((*reg).tbx.cast(), seq).cast();
        if !(*reg).itr.is_null() {
            return 0;
        }
        -1
    }
}

pub unsafe fn bcf_sr_regions_next(reg: *mut bcf_sr_regions_t) -> c_int {
    unsafe {
        if reg.is_null() || (*reg).iseq < 0 {
            return -1;
        }

        if !(*reg).regs.is_null() {
            (*reg).start = -1;
            (*reg).end = -1;
            (*reg).nals = 0;

            let regs = (*reg).regs.cast::<BcfSrRegion>();
            while (*reg).iseq < (*reg).nseqs {
                if advance_creg(regs.add((*reg).iseq as usize)) == 0 {
                    break;
                }
                (*reg).iseq += 1;
            }
            if (*reg).iseq >= (*reg).nseqs {
                (*reg).iseq = -1;
                return -1;
            }

            let seq_reg = regs.add((*reg).iseq as usize);
            let creg = (*seq_reg).regs.add((*seq_reg).creg as usize);
            (*reg).start = (*creg).start;
            (*reg).end = (*creg).end;
            return 0;
        }

        // reading from tabix
        const TBX_UCSC: c_int = 0x10000;
        let mut chr: *mut c_char = std::ptr::null_mut();
        let mut chr_end: *mut c_char = std::ptr::null_mut();
        let mut ichr = 0;
        let mut ifrom = 1;
        let mut ito = 2;
        let mut is_bed = 0;
        let mut from: hts_pos_t = 0;
        let mut to: hts_pos_t = 0;
        if !(*reg).tbx.is_null() {
            ichr = (*(*reg).tbx).conf.sc - 1;
            ifrom = (*(*reg).tbx).conf.bc - 1;
            ito = (*(*reg).tbx).conf.ec - 1;
            if ito < 0 {
                ito = ifrom;
            }
            is_bed = if (*(*reg).tbx).conf.preset == TBX_UCSC {
                1
            } else {
                0
            };
        }

        let line_ptr: *mut kstring_t = (&raw mut (*reg).line).cast();
        let mut ret = 0;
        while ret == 0 {
            if !(*reg).itr.is_null() {
                ret = sr_tbx_itr_next(
                    (*reg).file.cast(),
                    (*reg).tbx.cast(),
                    (*reg).itr.cast(),
                    line_ptr,
                );
                if ret < 0 {
                    (*reg).iseq = -1;
                    return -1;
                }
            } else {
                if (*reg).is_bin != 0 {
                    // Waited for seek which never came. Reopen in text mode.
                    let _ = hts_close((*reg).file.cast());
                    (*reg).file = hts_open((*reg).fname, c"r".as_ptr()).cast();
                    if (*reg).file.is_null() {
                        bcf_sr_regions_destroy_translated(reg);
                        return -1;
                    }
                    (*reg).is_bin = 0;
                }
                ret = if !(*reg).file.is_null() {
                    crate::hts::hts_getline((*reg).file.cast(), KS_SEP_LINE as c_int, line_ptr)
                } else {
                    -1
                };
                if ret < 0 {
                    (*reg).iseq = -1;
                    return -1;
                }
            }
            ret = regions_parse_line(
                (*reg).line.s,
                ichr,
                ifrom,
                ito,
                &mut chr,
                &mut chr_end,
                &mut from,
                &mut to,
            );
            if ret < 0 {
                return -1;
            }
        }
        if is_bed != 0 {
            from += 1;
        }

        *chr_end = 0;
        if crate::sam::khash_str2int_get((*reg).seq_hash, chr, &mut (*reg).iseq) < 0 {
            libc::abort();
        }
        *chr_end = b'\t' as c_char;

        (*reg).start = from - 1;
        (*reg).end = to - 1;
        0
    }
}

// original: bcf_sr_regions_overlap (htslib/synced_bcf_reader.c:1525)
pub unsafe fn bcf_sr_regions_overlap(
    reg: *mut bcf_sr_regions_t,
    seq: *const c_char,
    start: hts_pos_t,
    end: hts_pos_t,
) -> c_int {
    unsafe {
        if reg.is_null() || seq.is_null() {
            return -1;
        }
        bcf_sr_regions_overlap_inner(reg, seq, start, end, 1)
    }
}

// original: bcf_sr_regions_flush (htslib/synced_bcf_reader.c:1559)
pub unsafe fn bcf_sr_regions_flush(reg: *mut bcf_sr_regions_t) -> c_int {
    unsafe {
        if reg.is_null() {
            return -1;
        }
        let Some(handler) = (*reg).missed_reg_handler else {
            return 0;
        };
        if (*reg).prev_seq == -1 {
            return 0;
        }
        while bcf_sr_regions_next(reg) == 0 {
            handler(reg, (*reg).missed_reg_data);
        }
        0
    }
}
