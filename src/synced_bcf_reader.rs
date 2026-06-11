// Functions translated from htslib/synced_bcf_reader.c.
// Extracted from src/vcf.rs.

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::mem::size_of;
use std::ptr::NonNull;

use crate::htslib_rs::hts::{
    htsFile, hts_close, hts_open, hts_pos_t, kstring_t, HTS_FORMAT_VCF, KS_SEP_LINE,
};
use crate::htslib_rs::vcf::*;

// (extracted functions in src/vcf.rs order)

pub unsafe fn synced_bcf_reader_c_1070_regions_merge(reg: *mut c_void) {
    unsafe {
        if let Some(reg) = reg.cast::<BcfSrRegion>().as_mut() {
            regions_merge_ref(reg);
        }
    }
}

pub unsafe fn synced_bcf_reader_c_1085__regions_sort_and_merge(reg: *mut bcf_sr_regions_t) {
    unsafe {
        if let Some(reg) = reg.as_mut() {
            regions_sort_and_merge_ref(reg);
        }
    }
}

pub(crate) unsafe fn regions_sort_and_merge_ref(reg: &mut bcf_sr_regions_t) {
    unsafe {
        let regs = if reg.regs.is_null() || reg.nseqs <= 0 {
            &mut [][..]
        } else {
            std::slice::from_raw_parts_mut(reg.regs.cast::<BcfSrRegion>(), reg.nseqs as usize)
        };
        for seq_reg in regs {
            if !seq_reg.regs.is_null() && seq_reg.nregs > 1 {
                let intervals =
                    std::slice::from_raw_parts_mut(seq_reg.regs, seq_reg.nregs as usize);
                intervals.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));
            }
            regions_merge_ref(seq_reg);
        }
    }
}

unsafe fn regions_merge_ref(reg: &mut BcfSrRegion) {
    unsafe {
        let intervals = if reg.regs.is_null() || reg.nregs <= 0 {
            &mut [][..]
        } else {
            std::slice::from_raw_parts_mut(reg.regs, reg.nregs as usize)
        };
        let mut i = 0;
        while i < intervals.len() {
            let mut j = i + 1;
            while j < intervals.len() && intervals[i].end >= intervals[j].start {
                if intervals[i].end < intervals[j].end {
                    intervals[i].end = intervals[j].end;
                }
                intervals[j].start = 1;
                intervals[j].end = 0;
                j += 1;
            }
            i = j;
        }
    }
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
        let Some(readers) = readers.as_mut() else {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            return 0;
        };
        if file_ptr.is_null() {
            readers.errnum = bcf_sr_error_api_usage_error;
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            return 0;
        }
        let idxname = idxname.as_ref().map(|_| CStr::from_ptr(idxname).to_bytes());
        bcf_sr_add_hreader_ref(readers, &mut *file_ptr, autoclose, idxname)
    }
}

pub(crate) unsafe fn bcf_sr_add_hreader_ref(
    readers: &mut bcf_srs_t,
    file: &mut htsFile,
    autoclose: c_int,
    idxname: Option<&[u8]>,
) -> c_int {
    unsafe {
        // Re-NUL-terminate the optional index name at the libhts boundary.
        let idxname_c = idxname.map(|s| CString::new(s).unwrap());
        let idxname_ptr = idxname_c.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
        bcf_sr_add_hreader_impl(readers as *mut bcf_srs_t, file, autoclose, idxname_ptr)
    }
}

unsafe fn bcf_sr_aux_ref(readers: &bcf_srs_t) -> Option<&BcfSrAux> {
    unsafe { readers.aux.cast::<BcfSrAux>().as_ref() }
}

unsafe fn bcf_sr_aux_ref_mut(readers: &mut bcf_srs_t) -> Option<&mut BcfSrAux> {
    unsafe { readers.aux.cast::<BcfSrAux>().as_mut() }
}

pub unsafe fn bcf_sr_init() -> *mut bcf_srs_t {
    unsafe {
        let mut files = Box::new(bcf_srs_t::default());
        let aux = Box::new(BcfSrAux::default());
        files.aux = Box::into_raw(aux);
        let files = Box::into_raw(files);
        bcf_sr_sort_c_675_bcf_sr_sort_init(&mut (*bcf_sr_aux_mut(files)).sort);
        bcf_sr_set_opt_ref(&mut *files, BCF_SR_REGIONS_OVERLAP, 1);
        bcf_sr_set_opt_ref(&mut *files, BCF_SR_TARGETS_OVERLAP, 0);
        files
    }
}

pub unsafe fn synced_bcf_reader_c_461_bcf_sr_destroy1(reader: *mut bcf_sr_t, closefile: c_int) {
    unsafe {
        if let Some(reader) = reader.as_mut() {
            bcf_sr_destroy1_ref(reader, closefile);
        }
    }
}

pub(crate) unsafe fn bcf_sr_destroy1_ref(reader: &mut bcf_sr_t, closefile: c_int) {
    unsafe {
        if !reader.file.is_null() && closefile != 0 {
            let _ = hts_close(reader.file.cast());
        }
        if !reader.fname.is_null() {
            drop(CString::from_raw(reader.fname));
            reader.fname = std::ptr::null_mut();
        }
        if !reader.tbx_idx.is_null() {
            crate::tbx::tbx_destroy(reader.tbx_idx.cast());
        }
        if !reader.bcf_idx.is_null() {
            crate::hts::hts_idx_destroy(reader.bcf_idx.cast());
        }
        bcf_hdr_destroy(reader.header);
        if !reader.itr.is_null() {
            crate::hts::hts_itr_destroy(reader.itr.cast());
        }
        if !reader.buffer.is_null() && reader.mbuffer > 0 {
            for record in std::slice::from_raw_parts(reader.buffer, reader.mbuffer as usize) {
                bcf_destroy(*record);
            }
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                reader.buffer,
                reader.mbuffer as usize,
            )));
        }
        if !reader.samples.is_null() && reader.n_smpl > 0 {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                reader.samples,
                reader.n_smpl as usize,
            )));
        }
        if !reader.filter_ids.is_null() && reader.nfilter_ids > 0 {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                reader.filter_ids,
                reader.nfilter_ids as usize,
            )));
        }
    }
}

pub(crate) unsafe fn bcf_sr_regions_set_overlap_ref(
    regions: &mut bcf_sr_regions_t,
    overlap: c_int,
) {
    unsafe {
        let overlap_ptr = (regions as *mut bcf_sr_regions_t)
            .cast::<u8>()
            .add(size_of::<bcf_sr_regions_t>())
            .cast::<c_int>();
        *overlap_ptr = overlap;
    }
}

pub(crate) unsafe fn bcf_sr_regions_destroy_ref(regions: &mut bcf_sr_regions_t) {
    unsafe { bcf_sr_regions_destroy_translated(regions as *mut bcf_sr_regions_t) }
}

pub unsafe fn bcf_sr_destroy(files: *mut bcf_srs_t) {
    unsafe {
        let Some(files_ptr) = NonNull::new(files) else {
            return;
        };
        let mut files = Box::from_raw(files_ptr.as_ptr());
        let autoclose = bcf_sr_aux_ref(&files).map_or(std::ptr::null_mut(), |aux| aux.closefile);
        let readers = if files.readers.is_null() || files.nreaders <= 0 {
            &mut [][..]
        } else {
            std::slice::from_raw_parts_mut(files.readers, files.nreaders as usize)
        };
        for (i, reader) in readers.iter_mut().enumerate() {
            let cf = if autoclose.is_null() {
                0
            } else {
                *autoclose.add(i)
            };
            bcf_sr_destroy1_ref(reader, cf);
        }
        if !files.has_line.is_null() && files.nreaders > 0 {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                files.has_line,
                files.nreaders as usize,
            )));
        }
        if !files.readers.is_null() && files.nreaders > 0 {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                files.readers,
                files.nreaders as usize,
            )));
        }
        let samples = if files.samples.is_null() {
            None
        } else {
            Some(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                files.samples,
                files.n_smpl as usize,
            )))
        };
        if let Some(samples) = samples {
            for sample in samples.iter().copied() {
                drop(CString::from_raw(sample));
            }
        }
        if let Some(targets) = files.targets.as_mut() {
            bcf_sr_regions_destroy_ref(targets);
        }
        if let Some(regions) = files.regions.as_mut() {
            bcf_sr_regions_destroy_ref(regions);
        }
        if files.tmps.data.capacity() != 0 {
            files.tmps.data = Vec::new();
        }
        if files.n_threads != 0 {
            bcf_sr_destroy_threads_ref(&mut files);
        }
        if let Some(aux) = bcf_sr_aux_ref_mut(&mut files) {
            bcf_sr_sort_c_685_bcf_sr_sort_destroy(Some(&mut aux.sort));
        }
        if !autoclose.is_null() && files.nreaders >= 0 {
            // `closefile` is the autoclose array sized to the reader count.
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                autoclose,
                files.nreaders.max(0) as usize,
            )));
        }
        if !files.aux.is_null() {
            drop(Box::from_raw(files.aux));
        }
    }
}

pub unsafe fn bcf_sr_strerror(errnum: c_int) -> Vec<u8> {
    match errnum {
        x if x == bcf_sr_error_open_failed as c_int => {
            let errno = unsafe { *crate::htslib_rs::c_compat::__errno_location() };
            std::io::Error::from_raw_os_error(errno)
                .to_string()
                .into_bytes()
        }
        x if x == bcf_sr_error_not_bgzf as c_int => b"not compressed with bgzip".to_vec(),
        x if x == bcf_sr_error_idx_load_failed as c_int => b"could not load index".to_vec(),
        x if x == bcf_sr_error_file_type_error as c_int => b"unknown file type".to_vec(),
        x if x == bcf_sr_error_api_usage_error as c_int => b"API usage error".to_vec(),
        x if x == bcf_sr_error_header_error as c_int => b"could not parse header".to_vec(),
        x if x == bcf_sr_error_no_eof as c_int => {
            b"no BGZF EOF marker; file may be truncated".to_vec()
        }
        x if x == bcf_sr_error_no_memory as c_int => b"Out of memory".to_vec(),
        x if x == bcf_sr_error_vcf_parse_error as c_int => b"VCF parse error".to_vec(),
        x if x == bcf_sr_error_bcf_read_error as c_int => b"BCF read error".to_vec(),
        BCF_SR_ERROR_NOIDX_ERROR => b"merge of unindexed files failed".to_vec(),
        _ => Vec::new(),
    }
}

// original: bcf_sr_set_threads (htslib/synced_bcf_reader.c:228)
pub unsafe fn bcf_sr_set_threads(files: *mut bcf_srs_t, n_threads: c_int) -> c_int {
    unsafe {
        let Some(files) = files.as_mut() else {
            return -1;
        };
        bcf_sr_set_threads_ref(files, n_threads)
    }
}

pub(crate) unsafe fn bcf_sr_set_threads_ref(files: &mut bcf_srs_t, n_threads: c_int) -> c_int {
    files.n_threads = n_threads;
    if n_threads == 0 {
        return 0;
    }

    let mut thread_pool = Box::new(unsafe { std::mem::zeroed::<crate::hts::htsThreadPool>() });
    thread_pool.pool = crate::thread_pool::hts_tpool_init(n_threads);
    if thread_pool.pool.is_null() {
        files.errnum = bcf_sr_error_no_memory;
        return -1;
    }
    files.p = Box::into_raw(thread_pool);
    0
}

pub unsafe fn bcf_sr_set_opt_require_idx(readers: *mut bcf_srs_t) -> c_int {
    unsafe { readers.as_mut().map_or(-1, bcf_sr_set_opt_require_idx_ref) }
}

pub(crate) fn bcf_sr_set_opt_require_idx_ref(readers: &mut bcf_srs_t) -> c_int {
    readers.require_index = REQUIRE_IDX_;
    0
}

pub unsafe fn bcf_sr_set_opt_allow_no_idx(readers: *mut bcf_srs_t) -> c_int {
    unsafe { readers.as_mut().map_or(-1, bcf_sr_set_opt_allow_no_idx_ref) }
}

pub(crate) fn bcf_sr_set_opt_allow_no_idx_ref(readers: &mut bcf_srs_t) -> c_int {
    readers.require_index = ALLOW_NO_IDX_;
    0
}

pub unsafe fn bcf_sr_set_opt_pair_logic(readers: *mut bcf_srs_t, pair_logic: c_int) -> c_int {
    unsafe {
        let Some(readers) = readers.as_mut() else {
            return -1;
        };
        bcf_sr_set_opt_pair_logic_ref(readers, pair_logic)
    }
}

pub(crate) unsafe fn bcf_sr_set_opt_pair_logic_ref(
    readers: &mut bcf_srs_t,
    pair_logic: c_int,
) -> c_int {
    unsafe {
        let Some(aux) = bcf_sr_aux_ref_mut(readers) else {
            return -1;
        };
        aux.sort.pair = pair_logic;
    }
    0
}

pub unsafe fn bcf_sr_set_opt_regions_overlap(readers: *mut bcf_srs_t, overlap: c_int) -> c_int {
    unsafe {
        let Some(readers) = readers.as_mut() else {
            return -1;
        };
        bcf_sr_set_opt_regions_overlap_ref(readers, overlap)
    }
}

pub(crate) unsafe fn bcf_sr_set_opt_regions_overlap_ref(
    readers: &mut bcf_srs_t,
    overlap: c_int,
) -> c_int {
    unsafe {
        let Some(aux) = bcf_sr_aux_ref_mut(readers) else {
            return -1;
        };
        aux.regions_overlap = overlap;
        if let Some(regions) = readers.regions.as_mut() {
            bcf_sr_regions_set_overlap_ref(regions, overlap);
        }
    }
    0
}

pub unsafe fn bcf_sr_set_opt_targets_overlap(readers: *mut bcf_srs_t, overlap: c_int) -> c_int {
    unsafe {
        let Some(readers) = readers.as_mut() else {
            return -1;
        };
        bcf_sr_set_opt_targets_overlap_ref(readers, overlap)
    }
}

pub(crate) unsafe fn bcf_sr_set_opt_targets_overlap_ref(
    readers: &mut bcf_srs_t,
    overlap: c_int,
) -> c_int {
    unsafe {
        let Some(aux) = bcf_sr_aux_ref_mut(readers) else {
            return -1;
        };
        aux.targets_overlap = overlap;
        if let Some(targets) = readers.targets.as_mut() {
            bcf_sr_regions_set_overlap_ref(targets, overlap);
        }
    }
    0
}

pub unsafe fn bcf_sr_set_opt(readers: *mut bcf_srs_t, opt: bcf_sr_opt_t, value: c_int) -> c_int {
    unsafe {
        let Some(readers) = readers.as_mut() else {
            return -1;
        };
        bcf_sr_set_opt_ref(readers, opt, value)
    }
}

pub(crate) unsafe fn bcf_sr_set_opt_ref(
    readers: &mut bcf_srs_t,
    opt: bcf_sr_opt_t,
    value: c_int,
) -> c_int {
    match opt {
        BCF_SR_REQUIRE_IDX => bcf_sr_set_opt_require_idx_ref(readers),
        BCF_SR_ALLOW_NO_IDX => bcf_sr_set_opt_allow_no_idx_ref(readers),
        BCF_SR_PAIR_LOGIC => unsafe { bcf_sr_set_opt_pair_logic_ref(readers, value) },
        BCF_SR_REGIONS_OVERLAP => unsafe { bcf_sr_set_opt_regions_overlap_ref(readers, value) },
        BCF_SR_TARGETS_OVERLAP => unsafe { bcf_sr_set_opt_targets_overlap_ref(readers, value) },
        _ => 1,
    }
}

// original: bcf_sr_destroy_threads (htslib/synced_bcf_reader.c:244)
pub unsafe fn bcf_sr_destroy_threads(files: *mut bcf_srs_t) {
    unsafe {
        if let Some(files) = files.as_mut() {
            bcf_sr_destroy_threads_ref(files);
        }
    }
}

pub(crate) unsafe fn bcf_sr_destroy_threads_ref(files: &mut bcf_srs_t) {
    unsafe {
        let Some(p) = NonNull::new(files.p.cast::<crate::hts::htsThreadPool>()) else {
            return;
        };
        let p = Box::from_raw(p.as_ptr());
        if !p.pool.is_null() {
            crate::thread_pool::hts_tpool_destroy(p.pool);
        }
        files.p = std::ptr::null_mut();
    }
}

pub unsafe fn bcf_sr_add_reader(files: *mut bcf_srs_t, fname: *const c_char) -> c_int {
    unsafe {
        let Some(files) = files.as_mut() else {
            return 0;
        };
        let Some(fname) = fname.as_ref().map(|_| CStr::from_ptr(fname).to_bytes()) else {
            return 0;
        };
        bcf_sr_add_reader_ref(files, fname)
    }
}

pub(crate) unsafe fn bcf_sr_add_reader_ref(files: &mut bcf_srs_t, fname: &[u8]) -> c_int {
    unsafe {
        // Re-NUL-terminate at the libhts boundary.
        let fname_c = CString::new(fname).unwrap();
        let mut fmode = [0 as c_char; 5];
        fmode[0] = b'r' as c_char;
        vcf_open_mode(fmode.as_mut_ptr().add(1), fname_c.as_ptr(), std::ptr::null());
        let file_ptr = hts_open(fname_c.as_ptr(), fmode.as_ptr());
        if file_ptr.is_null() {
            files.errnum = bcf_sr_error_open_failed;
            return 0;
        }
        // get idx name and pass to add_hreader
        let idxname = fname
            .windows(HTS_IDX_DELIM.len())
            .position(|w| w == HTS_IDX_DELIM)
            .map(|pos| &fname[pos + HTS_IDX_DELIM.len()..]);
        let ret = bcf_sr_add_hreader_ref(files, &mut *file_ptr, 1, idxname);
        if ret == 0 {
            let _ = hts_close(file_ptr);
        }
        ret
    }
}

// original: bcf_sr_remove_reader (htslib/synced_bcf_reader.c:504)
pub unsafe fn bcf_sr_remove_reader(files: *mut bcf_srs_t, i: c_int) {
    unsafe {
        if let Some(files) = files.as_mut() {
            bcf_sr_remove_reader_ref(files, i);
        }
    }
}

pub(crate) unsafe fn bcf_sr_remove_reader_ref(files: &mut bcf_srs_t, i: c_int) {
    unsafe {
        // assert( !files->samples );  // not ready for this yet
        let files_ptr = files as *mut bcf_srs_t;
        let autoclose = (*bcf_sr_aux_mut(files_ptr)).closefile;

        bcf_sr_sort_c_662_bcf_sr_sort_remove_reader(
            Some(&mut *files_ptr),
            Some(&mut (*bcf_sr_aux_mut(files_ptr)).sort),
            i,
        );
        let cf = if autoclose.is_null() {
            0
        } else {
            *autoclose.add(i as usize)
        };
        bcf_sr_destroy1_ref(&mut *files.readers.add(i as usize), cf);
        if i + 1 < files.nreaders {
            let n = (files.nreaders - i - 1) as usize;
            std::ptr::copy(
                files.readers.add((i + 1) as usize),
                files.readers.add(i as usize),
                n,
            );
            std::ptr::copy(
                files.has_line.add((i + 1) as usize),
                files.has_line.add(i as usize),
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
        files.nreaders -= 1;
    }
}

// original: bcf_sr_next_line (htslib/synced_bcf_reader.c:869)
pub unsafe fn bcf_sr_next_line(files: *mut bcf_srs_t) -> c_int {
    unsafe {
        let Some(files) = files.as_mut() else {
            return 0;
        };
        bcf_sr_next_line_ref(files)
    }
}

pub(crate) unsafe fn bcf_sr_next_line_ref(files: &mut bcf_srs_t) -> c_int {
    unsafe {
        let files_ptr = files as *mut bcf_srs_t;
        if files.targets_als == 0 {
            return sr_next_line(files_ptr);
        }

        loop {
            let ret = sr_next_line(files_ptr);
            if ret == 0 {
                return ret;
            }

            let mut i = 0;
            while i < files.nreaders {
                if *files.has_line.add(i as usize) != 0 {
                    break;
                }
                i += 1;
            }

            if sr_regions_match_alleles(
                files.targets,
                files.targets_als - 1,
                *(*files.readers.add(i as usize)).buffer,
            ) != 0
            {
                return ret;
            }

            // Check if there are more duplicate lines in the buffers. If not,
            // return this line even if there is a type mismatch.
            i = 0;
            while i < files.nreaders {
                if *files.has_line.add(i as usize) == 0 {
                    i += 1;
                    continue;
                }
                let r = files.readers.add(i as usize);
                if (*r).nbuffer == 0 || (*(*(*r).buffer.add(1))).pos != (*(*(*r).buffer)).pos {
                    i += 1;
                    continue;
                }
                break;
            }
            if i == files.nreaders {
                return ret;
            }
        }
    }
}

pub unsafe fn bcf_sr_has_line(readers: *mut bcf_srs_t, i: c_int) -> c_int {
    unsafe {
        let Some(readers) = readers.as_ref() else {
            return 0;
        };
        bcf_sr_has_line_ref(readers, i)
    }
}

pub(crate) unsafe fn bcf_sr_has_line_ref(readers: &bcf_srs_t, i: c_int) -> c_int {
    unsafe {
        if i < 0 || i >= readers.nreaders || readers.has_line.is_null() {
            return 0;
        }
        *readers.has_line.add(i as usize)
    }
}

pub unsafe fn bcf_sr_get_line(readers: *mut bcf_srs_t, i: c_int) -> *mut bcf1_t {
    unsafe {
        let Some(readers) = readers.as_ref() else {
            return std::ptr::null_mut();
        };
        bcf_sr_get_line_ref(readers, i)
            .map(|line| line as *const bcf1_t as *mut bcf1_t)
            .unwrap_or(std::ptr::null_mut())
    }
}

pub(crate) unsafe fn bcf_sr_get_line_ref(readers: &bcf_srs_t, i: c_int) -> Option<&bcf1_t> {
    unsafe {
        if bcf_sr_has_line_ref(readers, i) == 0 || readers.readers.is_null() {
            return None;
        }
        let reader = readers.readers.add(i as usize);
        if (*reader).buffer.is_null() {
            return None;
        }
        (*(*reader).buffer).as_ref()
    }
}

pub unsafe fn bcf_sr_get_header(readers: *mut bcf_srs_t, i: c_int) -> *mut bcf_hdr_t {
    unsafe {
        let Some(readers) = readers.as_ref() else {
            return std::ptr::null_mut();
        };
        bcf_sr_get_header_ref(readers, i)
            .map(|header| header as *const bcf_hdr_t as *mut bcf_hdr_t)
            .unwrap_or(std::ptr::null_mut())
    }
}

pub(crate) fn bcf_sr_get_header_ref(readers: &bcf_srs_t, i: c_int) -> Option<&bcf_hdr_t> {
    unsafe {
        if i < 0 || i >= readers.nreaders || readers.readers.is_null() {
            return None;
        }
        (*readers.readers.add(i as usize)).header.as_ref()
    }
}

// original: bcf_sr_seek (htslib/synced_bcf_reader.c:911)
pub unsafe fn bcf_sr_seek(readers: *mut bcf_srs_t, seq: *const c_char, pos: hts_pos_t) -> c_int {
    unsafe {
        let Some(readers) = readers.as_mut() else {
            return 0;
        };
        bcf_sr_seek_ref(readers, seq.as_ref().map(|_| CStr::from_ptr(seq).to_bytes()), pos)
    }
}

pub(crate) unsafe fn bcf_sr_seek_ref(
    readers: &mut bcf_srs_t,
    seq: Option<&[u8]>,
    pos: hts_pos_t,
) -> c_int {
    unsafe {
        let readers_ptr = readers as *mut bcf_srs_t;
        if readers.regions.is_null() {
            return 0;
        }
        // Re-NUL-terminate the sequence name at the libhts boundary.
        let seq_c = seq.map(|s| CString::new(s).unwrap());
        let seq_ptr = seq_c.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
        if let Some(aux) = bcf_sr_aux_ref_mut(readers) {
            bcf_sr_sort_c_681_bcf_sr_sort_reset(Some(&mut aux.sort));
        }
        if seq.is_none() && pos == 0 {
            bcf_sr_seek_start(readers_ptr);
            return 0;
        }

        bcf_sr_seek_start(readers_ptr);
        let mut i = -1;
        if crate::sam::khash_str2int_get((*readers.regions).seq_hash, seq_ptr, &mut i) >= 0 {
            (*readers.regions).iseq = i;
        }
        if let Some(seq) = seq {
            bcf_sr_regions_overlap_inner_ref(&mut *readers.regions, seq, pos, pos, 0);
        }

        let mut nret = 0;
        for j in 0..readers.nreaders as usize {
            nret += sr_reader_seek(readers.readers.add(j), seq_ptr, pos, MAX_CSI_COOR - 1);
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
        let Some(files) = files.as_mut() else {
            return 0;
        };
        let Some(fname) = fname.as_ref().map(|_| CStr::from_ptr(fname).to_bytes()) else {
            return 0;
        };
        bcf_sr_set_samples_ref(files, fname, is_file)
    }
}

pub(crate) unsafe fn bcf_sr_set_samples_ref(
    files: &mut bcf_srs_t,
    fname: &[u8],
    is_file: c_int,
) -> c_int {
    unsafe {
        let fname_c = CString::new(fname).unwrap();
        let fname_ptr = fname_c.as_ptr();
        let mut nsmpl = 0;
        let mut free_smpl = 0;
        let mut smpl: *mut *mut c_char = std::ptr::null_mut();

        let exclude = if fname.first().copied() == Some(b'^') {
            crate::sam::khash_str2int_init()
        } else {
            std::ptr::null_mut()
        };
        if !exclude.is_null() || fname != b"-" {
            smpl = crate::hts::hts_readlist(fname_ptr, is_file, &mut nsmpl);
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
            smpl = (*(*files.readers).header).samples;
            nsmpl = bcf_hdr_nsamples_native((*files.readers).header);
        }

        let mut samples = Vec::new();
        for i in 0..nsmpl as usize {
            if !exclude.is_null() && crate::sam::khash_str2int_has_key(exclude, *smpl.add(i)) != 0 {
                continue;
            }
            let mut n_isec = 0;
            for j in 0..files.nreaders as usize {
                if bcf_hdr_id2int(
                    (*files.readers.add(j)).header,
                    BCF_DT_SAMPLE as c_int,
                    *smpl.add(i),
                ) < 0
                {
                    break;
                }
                n_isec += 1;
            }
            if n_isec != files.nreaders {
                continue;
            }
            samples.push(CStr::from_ptr(*smpl.add(i)).to_owned());
        }

        if !exclude.is_null() {
            crate::sam::khash_str2int_destroy(exclude);
        }
        if free_smpl != 0 {
            // `smpl` was returned by hts_readlist; reclaim each NUL-terminated
            // entry and the backing array as owned Rust allocations.
            let list = Box::from_raw(std::ptr::slice_from_raw_parts_mut(smpl, nsmpl as usize));
            for entry in list.iter().copied() {
                drop(CString::from_raw(entry));
            }
        }

        if samples.is_empty() {
            return 0;
        }
        if files.n_smpl > 0 && !files.samples.is_null() {
            let old_samples = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                files.samples,
                files.n_smpl as usize,
            ));
            for sample in old_samples.iter().copied() {
                drop(CString::from_raw(sample));
            }
        }
        files.n_smpl = samples.len() as c_int;
        let mut samples = samples
            .into_iter()
            .map(CString::into_raw)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        files.samples = samples.as_mut_ptr();
        std::mem::forget(samples);

        for i in 0..files.nreaders as usize {
            let reader = files.readers.add(i);
            if !(*reader).samples.is_null() && (*reader).n_smpl > 0 {
                drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                    (*reader).samples,
                    (*reader).n_smpl as usize,
                )));
            }
            let mut sample_ids = Vec::with_capacity(files.n_smpl as usize);
            (*reader).n_smpl = files.n_smpl;
            for j in 0..files.n_smpl as usize {
                sample_ids.push(bcf_hdr_id2int(
                    (*reader).header,
                    BCF_DT_SAMPLE as c_int,
                    *files.samples.add(j),
                ));
            }
            let mut sample_ids = sample_ids.into_boxed_slice();
            (*reader).samples = sample_ids.as_mut_ptr();
            std::mem::forget(sample_ids);
        }
        1
    }
}

// original: bcf_sr_set_targets (htslib/synced_bcf_reader.c:209)
pub unsafe fn bcf_sr_set_targets(
    readers: *mut bcf_srs_t,
    targets: *const c_char,
    is_file: c_int,
    alleles: c_int,
) -> c_int {
    unsafe {
        let Some(readers) = readers.as_mut() else {
            return -1;
        };
        let Some(targets) = targets.as_ref().map(|_| CStr::from_ptr(targets).to_bytes()) else {
            return -1;
        };
        bcf_sr_set_targets_ref(readers, targets, is_file, alleles)
    }
}

pub(crate) unsafe fn bcf_sr_set_targets_ref(
    readers: &mut bcf_srs_t,
    targets: &[u8],
    is_file: c_int,
    alleles: c_int,
) -> c_int {
    unsafe {
        if readers.nreaders != 0 || !readers.targets.is_null() {
            return -1;
        }
        let mut targets = targets;
        if targets.first().copied() == Some(b'^') {
            readers.targets_exclude = 1;
            targets = &targets[1..];
        }
        let Some(regions) = bcf_sr_regions_init_ref(targets, is_file, 0, 1, -2) else {
            return -1;
        };
        readers.targets = regions.as_ptr();
        readers.targets_als = alleles;
        if let Some(targets_overlap) = bcf_sr_aux_ref(readers).map(|aux| aux.targets_overlap) {
            bcf_sr_regions_set_overlap_ref(&mut *readers.targets, targets_overlap);
        }
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
        let Some(readers) = readers.as_mut() else {
            return -1;
        };
        let Some(regions) = regions.as_ref().map(|_| CStr::from_ptr(regions).to_bytes()) else {
            return -1;
        };
        bcf_sr_set_regions_ref(readers, regions, is_file)
    }
}

pub(crate) unsafe fn bcf_sr_set_regions_ref(
    readers: &mut bcf_srs_t,
    regions: &[u8],
    is_file: c_int,
) -> c_int {
    unsafe {
        let readers_ptr = readers as *mut bcf_srs_t;
        if readers.nreaders != 0 || !readers.regions.is_null() {
            if let Some(regions) = readers.regions.as_mut() {
                bcf_sr_regions_destroy_ref(regions);
            }
            readers.regions = bcf_sr_regions_init_ref(regions, is_file, 0, 1, -2)
                .map_or(std::ptr::null_mut(), NonNull::as_ptr);
            bcf_sr_seek_start(readers_ptr);
            return 0;
        }

        let Some(regions) = bcf_sr_regions_init_ref(regions, is_file, 0, 1, -2) else {
            return -1;
        };
        readers.regions = regions.as_ptr();
        readers.explicit_regs = 1;
        readers.require_index = REQUIRE_IDX_;
        if let Some(regions_overlap) = bcf_sr_aux_ref(readers).map(|aux| aux.regions_overlap) {
            bcf_sr_regions_set_overlap_ref(&mut *readers.regions, regions_overlap);
        }
        0
    }
}

// original: bcf_sr_regions_init (htslib/synced_bcf_reader.c:1248)
pub unsafe fn bcf_sr_regions_init(
    regions: *const c_char,
    is_file: c_int,
    ichr: c_int,
    ifrom: c_int,
    ito: c_int,
) -> *mut bcf_sr_regions_t {
    unsafe {
        regions
            .as_ref()
            .map(|_| CStr::from_ptr(regions).to_bytes())
            .and_then(|regions| bcf_sr_regions_init_ref(regions, is_file, ichr, ifrom, ito))
            .map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }
}

pub(crate) unsafe fn bcf_sr_regions_init_ref(
    regions: &[u8],
    is_file: c_int,
    ichr: c_int,
    ifrom: c_int,
    mut ito: c_int,
) -> Option<NonNull<bcf_sr_regions_t>> {
    unsafe {
        // Re-NUL-terminate the path/region string at the libhts boundary.
        let regions_c = CString::new(regions).unwrap();
        if is_file == 0 {
            let reg = regions_init_string(regions_c.as_ptr());
            if let Some(reg) = reg.as_mut() {
                regions_sort_and_merge_ref(reg);
            }
            return NonNull::new(reg);
        }

        let Some(reg) = NonNull::new(bcf_sr_regions_alloc()) else {
            return None;
        };
        let reg = reg.as_ptr();

        (*reg).file = hts_open(regions_c.as_ptr(), c"rb".as_ptr()).cast();
        if (*reg).file.is_null() {
            bcf_sr_regions_destroy_ref(&mut *reg);
            return None;
        }

        (*reg).tbx = crate::tbx::tbx_index_load3(
            regions_c.as_ptr(),
            std::ptr::null(),
            crate::hts::HTS_IDX_SAVE_REMOTE | crate::hts::HTS_IDX_SILENT_FAIL,
        )
        .cast();
        if (*reg).tbx.is_null() {
            let name = regions;
            let is_bed = name
                .get(name.len().saturating_sub(4)..)
                .is_some_and(|suffix| suffix.eq_ignore_ascii_case(b".bed"))
                || name
                    .get(name.len().saturating_sub(7)..)
                    .is_some_and(|suffix| suffix.eq_ignore_ascii_case(b".bed.gz"));
            let is_bed = if is_bed { 1 } else { 0 };

            let rfile: *mut htsFile = (*reg).file.cast();
            let line_ptr: *mut kstring_t = &raw mut (*reg).line;
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
                    (*reg).line.data.as_mut_ptr() as *mut c_char,
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
                            (*reg).line.data.as_mut_ptr() as *mut c_char,
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
                        bcf_sr_regions_destroy_ref(&mut *reg);
                        return None;
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
                bcf_sr_regions_destroy_ref(&mut *reg);
                return None;
            }
            regions_sort_and_merge_ref(&mut *reg);
            return NonNull::new(reg);
        }

        let tbx = &*(*reg).tbx.cast::<crate::tbx::tbx_t>();
        (*reg).seq_names = crate::tbx::tbx_seqnames(tbx, &mut (*reg).nseqs).cast::<*mut c_char>();
        if (*reg).seq_hash.is_null() {
            (*reg).seq_hash = crate::sam::khash_str2int_init();
        }
        for i in 0..(*reg).nseqs as usize {
            crate::sam::khash_str2int_set((*reg).seq_hash, *(*reg).seq_names.add(i), i as c_int);
        }
        (*reg).fname = CString::new(regions).unwrap().into_raw();
        (*reg).is_bin = 1;
        NonNull::new(reg)
    }
}

pub unsafe fn bcf_sr_regions_destroy(regions: *mut bcf_sr_regions_t) {
    unsafe {
        if let Some(regions) = regions.as_mut() {
            bcf_sr_regions_destroy_ref(regions);
        }
    }
}

// original: bcf_sr_regions_seek (htslib/synced_bcf_reader.c:1352)
pub unsafe fn bcf_sr_regions_seek(reg: *mut bcf_sr_regions_t, seq: *const c_char) -> c_int {
    unsafe {
        let Some(reg) = reg.as_mut() else {
            return -1;
        };
        let Some(seq) = seq.as_ref().map(|_| CStr::from_ptr(seq).to_bytes()) else {
            return -1;
        };
        bcf_sr_regions_seek_ref(reg, seq)
    }
}

pub(crate) unsafe fn bcf_sr_regions_seek_ref(reg: &mut bcf_sr_regions_t, seq: &[u8]) -> c_int {
    unsafe {
        let seq_c = CString::new(seq).unwrap();
        let seq_ptr = seq_c.as_ptr();
        reg.iseq = -1;
        reg.start = -1;
        reg.end = -1;
        if crate::sam::khash_str2int_get(reg.seq_hash, seq_ptr, &mut reg.iseq) < 0 {
            return -1; // sequence seq not in regions
        }

        // using in-memory regions
        if !reg.regs.is_null() {
            let regs =
                std::slice::from_raw_parts_mut(reg.regs.cast::<BcfSrRegion>(), reg.nseqs as usize);
            regs[reg.iseq as usize].creg = -1;
            return 0;
        }

        // reading regions from tabix
        if !reg.itr.is_null() {
            crate::hts::hts_itr_destroy(reg.itr.cast());
        }
        reg.itr =
            crate::tbx::tbx_itr_querys1(&mut *reg.tbx.cast::<crate::tbx::tbx_t>(), seq_ptr).cast();
        if !reg.itr.is_null() {
            return 0;
        }
        -1
    }
}

pub unsafe fn bcf_sr_regions_next(reg: *mut bcf_sr_regions_t) -> c_int {
    unsafe {
        let Some(reg) = reg.as_mut() else {
            return -1;
        };
        bcf_sr_regions_next_ref(reg)
    }
}

pub(crate) unsafe fn bcf_sr_regions_next_ref(reg: &mut bcf_sr_regions_t) -> c_int {
    unsafe {
        if reg.iseq < 0 {
            return -1;
        }

        if !reg.regs.is_null() {
            reg.start = -1;
            reg.end = -1;
            reg.nals = 0;

            let regs =
                std::slice::from_raw_parts_mut(reg.regs.cast::<BcfSrRegion>(), reg.nseqs as usize);
            while reg.iseq < reg.nseqs {
                let seq_reg = &mut regs[reg.iseq as usize];
                let intervals = if seq_reg.regs.is_null() || seq_reg.nregs <= 0 {
                    &[][..]
                } else {
                    std::slice::from_raw_parts(seq_reg.regs, seq_reg.nregs as usize)
                };
                let next = (seq_reg.creg + 1..seq_reg.nregs)
                    .find(|&i| intervals[i as usize].start <= intervals[i as usize].end);
                if let Some(next) = next {
                    seq_reg.creg = next;
                    break;
                }
                seq_reg.creg = seq_reg.nregs;
                reg.iseq += 1;
            }
            if reg.iseq >= reg.nseqs {
                reg.iseq = -1;
                return -1;
            }

            let seq_reg = &regs[reg.iseq as usize];
            let creg = &*seq_reg.regs.add(seq_reg.creg as usize);
            reg.start = creg.start;
            reg.end = creg.end;
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
        if !reg.tbx.is_null() {
            ichr = (*reg.tbx).conf.sc - 1;
            ifrom = (*reg.tbx).conf.bc - 1;
            ito = (*reg.tbx).conf.ec - 1;
            if ito < 0 {
                ito = ifrom;
            }
            is_bed = if (*reg.tbx).conf.preset == TBX_UCSC {
                1
            } else {
                0
            };
        }

        let line_ptr: *mut kstring_t = &raw mut reg.line;
        let mut ret = 0;
        while ret == 0 {
            if !reg.itr.is_null() {
                ret = sr_tbx_itr_next(reg.file.cast(), reg.tbx.cast(), reg.itr.cast(), line_ptr);
                if ret < 0 {
                    reg.iseq = -1;
                    return -1;
                }
            } else {
                if reg.is_bin != 0 {
                    // Waited for seek which never came. Reopen in text mode.
                    let _ = hts_close(reg.file.cast());
                    reg.file = hts_open(reg.fname, c"r".as_ptr()).cast();
                    if reg.file.is_null() {
                        bcf_sr_regions_destroy_ref(reg);
                        return -1;
                    }
                    reg.is_bin = 0;
                }
                ret = if !reg.file.is_null() {
                    crate::hts::hts_getline(reg.file.cast(), KS_SEP_LINE as c_int, line_ptr)
                } else {
                    -1
                };
                if ret < 0 {
                    reg.iseq = -1;
                    return -1;
                }
            }
            ret = regions_parse_line(
                reg.line.data.as_mut_ptr() as *mut c_char,
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
        if crate::sam::khash_str2int_get(reg.seq_hash, chr, &mut reg.iseq) < 0 {
            std::process::abort();
        }
        *chr_end = b'\t' as c_char;

        reg.start = from - 1;
        reg.end = to - 1;
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
        let Some(reg) = reg.as_mut() else {
            return -1;
        };
        let Some(seq) = seq.as_ref().map(|_| CStr::from_ptr(seq).to_bytes()) else {
            return -1;
        };
        bcf_sr_regions_overlap_ref(reg, seq, start, end)
    }
}

pub(crate) unsafe fn bcf_sr_regions_overlap_ref(
    reg: &mut bcf_sr_regions_t,
    seq: &[u8],
    start: hts_pos_t,
    end: hts_pos_t,
) -> c_int {
    unsafe { bcf_sr_regions_overlap_inner_ref(reg, seq, start, end, 1) }
}

unsafe fn bcf_sr_regions_overlap_inner_ref(
    reg: &mut bcf_sr_regions_t,
    seq: &[u8],
    start: hts_pos_t,
    end: hts_pos_t,
    mut missed_reg_handler: c_int,
) -> c_int {
    unsafe {
        let seq_c = CString::new(seq).unwrap();
        let mut iseq = -1;
        if crate::sam::khash_str2int_get(reg.seq_hash, seq_c.as_ptr(), &mut iseq) < 0 {
            return -1;
        }
        if missed_reg_handler != 0 && reg.missed_reg_handler.is_none() {
            missed_reg_handler = 0;
        }

        if reg.prev_seq == -1 || iseq != reg.prev_seq || reg.prev_start > start {
            if missed_reg_handler != 0 && reg.prev_seq != -1 && reg.iseq != -1 {
                bcf_sr_regions_flush_ref(reg);
            }
            bcf_sr_regions_seek_ref(reg, seq);
            reg.start = -1;
            reg.end = -1;
        }
        if reg.prev_seq == iseq && reg.iseq != iseq {
            return -2;
        }
        reg.prev_seq = reg.iseq;
        reg.prev_start = start;

        loop {
            if !(iseq == reg.iseq && reg.end < start) {
                break;
            }
            if bcf_sr_regions_next_ref(reg) < 0 {
                return -2;
            }
            if reg.iseq != iseq {
                return -1;
            }
            if missed_reg_handler != 0 && reg.end < start {
                if let Some(handler) = reg.missed_reg_handler {
                    handler(reg as *mut bcf_sr_regions_t, reg.missed_reg_data);
                }
            }
        }
        if reg.start <= end {
            return 0;
        }
        -1
    }
}

// original: bcf_sr_regions_flush (htslib/synced_bcf_reader.c:1559)
pub unsafe fn bcf_sr_regions_flush(reg: *mut bcf_sr_regions_t) -> c_int {
    unsafe {
        let Some(reg) = reg.as_mut() else {
            return -1;
        };
        bcf_sr_regions_flush_ref(reg)
    }
}

pub(crate) unsafe fn bcf_sr_regions_flush_ref(reg: &mut bcf_sr_regions_t) -> c_int {
    unsafe {
        let Some(handler) = reg.missed_reg_handler else {
            return 0;
        };
        if reg.prev_seq == -1 {
            return 0;
        }
        while bcf_sr_regions_next_ref(reg) == 0 {
            handler(reg as *mut bcf_sr_regions_t, reg.missed_reg_data);
        }
        0
    }
}
