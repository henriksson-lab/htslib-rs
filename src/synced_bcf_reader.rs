// Functions translated from htslib/synced_bcf_reader.c.
// Extracted from src/vcf.rs.

use std::ffi::c_char;
use std::mem::size_of;
use std::ptr::NonNull;

use crate::htslib_rs::hts::{
    htsFile, hts_close, hts_open, hts_pos_t, kstring_t, HTS_FORMAT_VCF, KS_SEP_LINE,
};
use crate::htslib_rs::vcf::*;

// (extracted functions in src/vcf.rs order)

pub(crate) unsafe fn regions_sort_and_merge(reg: &mut bcf_sr_regions_t) {
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
            regions_merge(seq_reg);
        }
    }
}

unsafe fn regions_merge(reg: &mut BcfSrRegion) {
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

pub(crate) unsafe fn bcf_sr_add_hreader(
    readers: &mut bcf_srs_t,
    file: &mut htsFile,
    autoclose: i32,
    idxname: Option<&[u8]>,
) -> i32 {
    unsafe {
        // The callee still takes a NUL-terminated C string (or null).
        let idx_c: Option<Vec<u8>> = idxname.map(|s| {
            let mut v = s.to_vec();
            v.push(0);
            v
        });
        let idx_ptr = idx_c
            .as_ref()
            .map_or(std::ptr::null(), |v| v.as_ptr().cast::<c_char>());
        bcf_sr_add_hreader_impl(readers as *mut bcf_srs_t, file, autoclose, idx_ptr)
    }
}

unsafe fn bcf_sr_aux_borrow(readers: &bcf_srs_t) -> Option<&BcfSrAux> {
    unsafe { readers.aux.cast::<BcfSrAux>().as_ref() }
}

unsafe fn bcf_sr_aux_borrow_mut(readers: &mut bcf_srs_t) -> Option<&mut BcfSrAux> {
    unsafe { readers.aux.cast::<BcfSrAux>().as_mut() }
}

pub unsafe fn bcf_sr_init() -> *mut bcf_srs_t {
    unsafe {
        let mut files = Box::new(bcf_srs_t::default());
        let aux = Box::new(BcfSrAux::default());
        files.aux = Box::into_raw(aux);
        let files = Box::into_raw(files);
        bcf_sr_sort_init(&mut (*bcf_sr_aux_mut(files)).sort);
        bcf_sr_set_opt(&mut *files, BCF_SR_REGIONS_OVERLAP, 1);
        bcf_sr_set_opt(&mut *files, BCF_SR_TARGETS_OVERLAP, 0);
        files
    }
}

pub(crate) unsafe fn bcf_sr_destroy1(reader: &mut bcf_sr_t, closefile: i32) {
    unsafe {
        if !reader.file.is_null() && closefile != 0 {
            let _ = hts_close(reader.file.cast());
        }
        if !reader.fname.is_null() {
            // `fname` is a libc::strdup'd C string (see bcf_sr_add_hreader_impl).
            libc::free(reader.fname.cast());
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

pub(crate) unsafe fn bcf_sr_regions_set_overlap(
    regions: &mut bcf_sr_regions_t,
    overlap: i32,
) {
    unsafe {
        let overlap_ptr = (regions as *mut bcf_sr_regions_t)
            .cast::<u8>()
            .add(size_of::<bcf_sr_regions_t>())
            .cast::<i32>();
        *overlap_ptr = overlap;
    }
}

pub(crate) unsafe fn bcf_sr_regions_destroy(regions: &mut bcf_sr_regions_t) {
    unsafe { bcf_sr_regions_destroy_translated(regions as *mut bcf_sr_regions_t) }
}

pub unsafe fn bcf_sr_destroy(files: *mut bcf_srs_t) {
    unsafe {
        let Some(files_ptr) = NonNull::new(files) else {
            return;
        };
        let mut files = Box::from_raw(files_ptr.as_ptr());
        let autoclose = bcf_sr_aux_borrow(&files).map_or(std::ptr::null_mut(), |aux| aux.closefile);
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
            bcf_sr_destroy1(reader, cf);
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
        // `files.samples` is a malloc'd array of malloc'd C strings.
        if !files.samples.is_null() {
            for i in 0..files.n_smpl.max(0) as usize {
                libc::free((*files.samples.add(i)).cast());
            }
            libc::free(files.samples.cast());
            files.samples = std::ptr::null_mut();
        }
        if let Some(targets) = files.targets.as_mut() {
            bcf_sr_regions_destroy(targets);
        }
        if let Some(regions) = files.regions.as_mut() {
            bcf_sr_regions_destroy(regions);
        }
        if files.tmps.data.capacity() != 0 {
            files.tmps.data = Vec::new();
        }
        if files.n_threads != 0 {
            bcf_sr_destroy_threads(&mut files);
        }
        if let Some(aux) = bcf_sr_aux_borrow_mut(&mut files) {
            bcf_sr_sort_destroy(&mut aux.sort);
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

pub unsafe fn bcf_sr_strerror(errnum: i32) -> Vec<u8> {
    match errnum {
        x if x == bcf_sr_error_open_failed as i32 => {
            std::io::Error::last_os_error().to_string().into_bytes()
        }
        x if x == bcf_sr_error_not_bgzf as i32 => b"not compressed with bgzip".to_vec(),
        x if x == bcf_sr_error_idx_load_failed as i32 => b"could not load index".to_vec(),
        x if x == bcf_sr_error_file_type_error as i32 => b"unknown file type".to_vec(),
        x if x == bcf_sr_error_api_usage_error as i32 => b"API usage error".to_vec(),
        x if x == bcf_sr_error_header_error as i32 => b"could not parse header".to_vec(),
        x if x == bcf_sr_error_no_eof as i32 => {
            b"no BGZF EOF marker; file may be truncated".to_vec()
        }
        x if x == bcf_sr_error_no_memory as i32 => b"Out of memory".to_vec(),
        x if x == bcf_sr_error_vcf_parse_error as i32 => b"VCF parse error".to_vec(),
        x if x == bcf_sr_error_bcf_read_error as i32 => b"BCF read error".to_vec(),
        BCF_SR_ERROR_NOIDX_ERROR => b"merge of unindexed files failed".to_vec(),
        _ => Vec::new(),
    }
}

// original: bcf_sr_set_threads (htslib/synced_bcf_reader.c:228)
pub(crate) unsafe fn bcf_sr_set_threads(files: &mut bcf_srs_t, n_threads: i32) -> i32 {
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

pub(crate) fn bcf_sr_set_opt_require_idx(readers: &mut bcf_srs_t) -> i32 {
    readers.require_index = REQUIRE_IDX_;
    0
}

pub fn bcf_sr_set_opt_allow_no_idx(readers: &mut bcf_srs_t) -> i32 {
    readers.require_index = ALLOW_NO_IDX_;
    0
}

pub unsafe fn bcf_sr_set_opt_pair_logic(
    readers: &mut bcf_srs_t,
    pair_logic: i32,
) -> i32 {
    unsafe {
        let Some(aux) = bcf_sr_aux_borrow_mut(readers) else {
            return -1;
        };
        aux.sort.pair = pair_logic;
    }
    0
}

pub(crate) unsafe fn bcf_sr_set_opt_regions_overlap(
    readers: &mut bcf_srs_t,
    overlap: i32,
) -> i32 {
    unsafe {
        let Some(aux) = bcf_sr_aux_borrow_mut(readers) else {
            return -1;
        };
        aux.regions_overlap = overlap;
        if let Some(regions) = readers.regions.as_mut() {
            bcf_sr_regions_set_overlap(regions, overlap);
        }
    }
    0
}

pub(crate) unsafe fn bcf_sr_set_opt_targets_overlap(
    readers: &mut bcf_srs_t,
    overlap: i32,
) -> i32 {
    unsafe {
        let Some(aux) = bcf_sr_aux_borrow_mut(readers) else {
            return -1;
        };
        aux.targets_overlap = overlap;
        if let Some(targets) = readers.targets.as_mut() {
            bcf_sr_regions_set_overlap(targets, overlap);
        }
    }
    0
}

pub(crate) unsafe fn bcf_sr_set_opt(
    readers: &mut bcf_srs_t,
    opt: bcf_sr_opt_t,
    value: i32,
) -> i32 {
    match opt {
        BCF_SR_REQUIRE_IDX => bcf_sr_set_opt_require_idx(readers),
        BCF_SR_ALLOW_NO_IDX => bcf_sr_set_opt_allow_no_idx(readers),
        BCF_SR_PAIR_LOGIC => unsafe { bcf_sr_set_opt_pair_logic(readers, value) },
        BCF_SR_REGIONS_OVERLAP => unsafe { bcf_sr_set_opt_regions_overlap(readers, value) },
        BCF_SR_TARGETS_OVERLAP => unsafe { bcf_sr_set_opt_targets_overlap(readers, value) },
        _ => 1,
    }
}

// original: bcf_sr_destroy_threads (htslib/synced_bcf_reader.c:244)
pub(crate) unsafe fn bcf_sr_destroy_threads(files: &mut bcf_srs_t) {
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

pub unsafe fn bcf_sr_add_reader(files: &mut bcf_srs_t, fname: &[u8]) -> i32 {
    unsafe {
        let mut fname_c = fname.to_vec();
        fname_c.push(0);
        let mut fmode = [0u8; 5];
        fmode[0] = b'r';
        vcf_open_mode(
            fmode[1..].as_mut_ptr().cast::<c_char>(),
            fname_c.as_ptr().cast::<c_char>(),
            std::ptr::null(),
        );
        let file_ptr = hts_open(
            fname_c.as_ptr().cast::<c_char>(),
            fmode.as_ptr().cast::<c_char>(),
        );
        if file_ptr.is_null() {
            files.errnum = bcf_sr_error_open_failed;
            return 0;
        }
        // get idx name and pass to add_hreader
        let idxname = fname
            .windows(HTS_IDX_DELIM.len())
            .position(|w| w == HTS_IDX_DELIM)
            .map(|pos| &fname[pos + HTS_IDX_DELIM.len()..]);
        let ret = bcf_sr_add_hreader(files, &mut *file_ptr, 1, idxname);
        if ret == 0 {
            let _ = hts_close(file_ptr);
        }
        ret
    }
}

// original: bcf_sr_remove_reader (htslib/synced_bcf_reader.c:504)
pub(crate) unsafe fn bcf_sr_remove_reader(files: &mut bcf_srs_t, i: i32) {
    unsafe {
        // assert( !files->samples );  // not ready for this yet
        let files_ptr = files as *mut bcf_srs_t;
        let autoclose = (*bcf_sr_aux_mut(files_ptr)).closefile;

        bcf_sr_sort_remove_reader(&mut (*bcf_sr_aux_mut(files_ptr)).sort, i);
        let cf = if autoclose.is_null() {
            0
        } else {
            *autoclose.add(i as usize)
        };
        bcf_sr_destroy1(&mut *files.readers.add(i as usize), cf);
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
pub unsafe fn bcf_sr_next_line(files: &mut bcf_srs_t) -> i32 {
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

pub unsafe fn bcf_sr_has_line(readers: &bcf_srs_t, i: i32) -> i32 {
    unsafe {
        if i < 0 || i >= readers.nreaders || readers.has_line.is_null() {
            return 0;
        }
        *readers.has_line.add(i as usize)
    }
}

pub unsafe fn bcf_sr_get_line(readers: &bcf_srs_t, i: i32) -> Option<&bcf1_t> {
    unsafe {
        if bcf_sr_has_line(readers, i) == 0 || readers.readers.is_null() {
            return None;
        }
        let reader = readers.readers.add(i as usize);
        if (*reader).buffer.is_null() {
            return None;
        }
        (*(*reader).buffer).as_ref()
    }
}

pub fn bcf_sr_get_header(readers: &bcf_srs_t, i: i32) -> Option<&bcf_hdr_t> {
    unsafe {
        if i < 0 || i >= readers.nreaders || readers.readers.is_null() {
            return None;
        }
        (*readers.readers.add(i as usize)).header.as_ref()
    }
}

// original: bcf_sr_seek (htslib/synced_bcf_reader.c:911)
pub(crate) unsafe fn bcf_sr_seek(
    readers: &mut bcf_srs_t,
    seq: Option<&[u8]>,
    pos: hts_pos_t,
) -> i32 {
    unsafe {
        let readers_ptr = readers as *mut bcf_srs_t;
        if readers.regions.is_null() {
            return 0;
        }
        if let Some(aux) = bcf_sr_aux_borrow_mut(readers) {
            bcf_sr_sort_reset(&mut aux.sort);
        }
        if seq.is_none() && pos == 0 {
            bcf_sr_seek_start(readers_ptr);
            return 0;
        }

        bcf_sr_seek_start(readers_ptr);
        // NUL-terminated copy of `seq` for the C-string callees (null when None).
        let seq_c: Option<Vec<u8>> = seq.map(|s| {
            let mut v = s.to_vec();
            v.push(0);
            v
        });
        let seq_ptr = seq_c
            .as_ref()
            .map_or(std::ptr::null(), |v| v.as_ptr().cast::<c_char>());
        let mut i = -1;
        if crate::sam::khash_str2int_get(
            (*readers.regions).seq_hash.cast::<()>(),
            seq_ptr.cast::<u8>(),
            &mut i,
        ) >= 0
        {
            (*readers.regions).iseq = i;
        }
        if let Some(seq) = seq {
            bcf_sr_regions_overlap_inner(&mut *readers.regions, seq, pos, pos, 0);
        }

        let mut nret = 0;
        for j in 0..readers.nreaders as usize {
            nret += sr_reader_seek(readers.readers.add(j), seq_ptr, pos, MAX_CSI_COOR - 1);
        }
        nret
    }
}

// original: bcf_sr_set_samples (htslib/synced_bcf_reader.c:940)
pub(crate) unsafe fn bcf_sr_set_samples(
    files: &mut bcf_srs_t,
    fname: &[u8],
    is_file: i32,
) -> i32 {
    unsafe {
        // `smpl` is hts_readlist's malloc'd array of malloc'd C strings.
        let mut smpl: *mut *mut c_char = std::ptr::null_mut();
        let mut nsmpl: i32 = 0;

        let exclude: *mut () = if fname.first().copied() == Some(b'^') {
            crate::sam::khash_str2int_init()
        } else {
            std::ptr::null_mut()
        };
        let read_list = !exclude.is_null() || fname != b"-";
        if read_list {
            let mut fname_c = fname.to_vec();
            fname_c.push(0);
            smpl = crate::hts::hts_readlist(fname_c.as_ptr().cast::<c_char>(), is_file, &mut nsmpl);
            if smpl.is_null() {
                return 0;
            }
            if !exclude.is_null() {
                for i in 0..nsmpl as usize {
                    crate::sam::khash_str2int_inc(exclude, (*smpl.add(i)).cast::<u8>());
                }
            }
        }
        // The candidate sample names, as NUL-terminated C strings.
        let names: *mut *mut c_char = if read_list {
            smpl
        } else {
            (*(*files.readers).header).samples
        };
        let nnames: i32 = if read_list {
            nsmpl
        } else {
            bcf_hdr_nsamples_native((*files.readers).header)
        };

        // Collect the intersecting sample names as a malloc'd C-string array.
        let mut samples: *mut *mut c_char = std::ptr::null_mut();
        let mut n_smpl: i32 = 0;
        for k in 0..nnames as usize {
            let name = *names.add(k);
            if !exclude.is_null() && crate::sam::khash_str2int_has_key(exclude, name.cast::<u8>()) != 0
            {
                continue;
            }
            let mut n_isec = 0;
            for j in 0..files.nreaders as usize {
                if bcf_hdr_id2int(
                    (*files.readers.add(j)).header,
                    BCF_DT_SAMPLE as i32,
                    name,
                ) < 0
                {
                    break;
                }
                n_isec += 1;
            }
            if n_isec != files.nreaders {
                continue;
            }
            samples = libc::realloc(
                samples.cast(),
                (n_smpl as usize + 1) * size_of::<*mut c_char>(),
            )
            .cast::<*mut c_char>();
            *samples.add(n_smpl as usize) = libc::strdup(name);
            n_smpl += 1;
        }

        if !smpl.is_null() {
            for i in 0..nsmpl as usize {
                libc::free((*smpl.add(i)).cast());
            }
            libc::free(smpl.cast());
        }
        if !exclude.is_null() {
            crate::sam::khash_str2int_destroy(exclude);
        }

        if n_smpl == 0 {
            if !samples.is_null() {
                libc::free(samples.cast());
            }
            return 0;
        }
        files.n_smpl = n_smpl;
        files.samples = samples;

        for i in 0..files.nreaders as usize {
            let reader = files.readers.add(i);
            (*reader).n_smpl = files.n_smpl;
            let sample_ids = libc::malloc(files.n_smpl as usize * size_of::<i32>()).cast::<i32>();
            for s in 0..files.n_smpl as usize {
                *sample_ids.add(s) = bcf_hdr_id2int(
                    (*reader).header,
                    BCF_DT_SAMPLE as i32,
                    *files.samples.add(s),
                );
            }
            (*reader).samples = sample_ids;
        }
        1
    }
}

// original: bcf_sr_set_targets (htslib/synced_bcf_reader.c:209)
pub unsafe fn bcf_sr_set_targets(
    readers: &mut bcf_srs_t,
    targets: &[u8],
    is_file: i32,
    alleles: i32,
) -> i32 {
    unsafe {
        if readers.nreaders != 0 || !readers.targets.is_null() {
            return -1;
        }
        let mut targets = targets;
        if targets.first().copied() == Some(b'^') {
            readers.targets_exclude = 1;
            targets = &targets[1..];
        }
        let Some(regions) = bcf_sr_regions_init(targets, is_file, 0, 1, -2) else {
            return -1;
        };
        readers.targets = regions.as_ptr();
        readers.targets_als = alleles;
        if let Some(targets_overlap) = bcf_sr_aux_borrow(readers).map(|aux| aux.targets_overlap) {
            bcf_sr_regions_set_overlap(&mut *readers.targets, targets_overlap);
        }
        0
    }
}

// original: bcf_sr_set_regions (htslib/synced_bcf_reader.c:191)
pub unsafe fn bcf_sr_set_regions(
    readers: &mut bcf_srs_t,
    regions: &[u8],
    is_file: i32,
) -> i32 {
    unsafe {
        let readers_ptr = readers as *mut bcf_srs_t;
        if readers.nreaders != 0 || !readers.regions.is_null() {
            if let Some(regions) = readers.regions.as_mut() {
                bcf_sr_regions_destroy(regions);
            }
            readers.regions = bcf_sr_regions_init(regions, is_file, 0, 1, -2)
                .map_or(std::ptr::null_mut(), NonNull::as_ptr);
            bcf_sr_seek_start(readers_ptr);
            return 0;
        }

        let Some(regions) = bcf_sr_regions_init(regions, is_file, 0, 1, -2) else {
            return -1;
        };
        readers.regions = regions.as_ptr();
        readers.explicit_regs = 1;
        readers.require_index = REQUIRE_IDX_;
        if let Some(regions_overlap) = bcf_sr_aux_borrow(readers).map(|aux| aux.regions_overlap) {
            bcf_sr_regions_set_overlap(&mut *readers.regions, regions_overlap);
        }
        0
    }
}

// original: bcf_sr_regions_init (htslib/synced_bcf_reader.c:1248)
pub(crate) unsafe fn bcf_sr_regions_init(
    regions: &[u8],
    is_file: i32,
    ichr: i32,
    ifrom: i32,
    mut ito: i32,
) -> Option<NonNull<bcf_sr_regions_t>> {
    unsafe {
        // NUL-terminated copy of `regions` for the C-string callees.
        let mut regions_c = regions.to_vec();
        regions_c.push(0);
        let regions_ptr = regions_c.as_ptr().cast::<c_char>();

        if is_file == 0 {
            let reg = regions_init_string(regions_ptr);
            if let Some(reg) = reg.as_mut() {
                regions_sort_and_merge(reg);
            }
            return NonNull::new(reg);
        }

        let Some(reg) = NonNull::new(bcf_sr_regions_alloc()) else {
            return None;
        };
        let reg = reg.as_ptr();

        (*reg).file = hts_open(regions_ptr, b"rb\0".as_ptr().cast::<c_char>()).cast();
        if (*reg).file.is_null() {
            bcf_sr_regions_destroy(&mut *reg);
            return None;
        }

        (*reg).tbx = crate::tbx::tbx_index_load3(
            regions,
            None,
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
                if crate::hts::hts_getline(rfile, KS_SEP_LINE as i32, line_ptr) <= 0 {
                    break;
                }
                let mut chr: *mut c_char = std::ptr::null_mut();
                let mut chr_end: *mut c_char = std::ptr::null_mut();
                let mut from: hts_pos_t = 0;
                let mut to: hts_pos_t = 0;
                let mut ret = regions_parse_line(
                    (*reg).line.data.as_mut_ptr().cast::<c_char>(),
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
                            (*reg).line.data.as_mut_ptr().cast::<c_char>(),
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
                        bcf_sr_regions_destroy(&mut *reg);
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
                bcf_sr_regions_destroy(&mut *reg);
                return None;
            }
            regions_sort_and_merge(&mut *reg);
            return NonNull::new(reg);
        }

        // `tbx_seqnames` now returns owned byte strings; materialize them into the
        // struct's malloc'd C-string array (freed by bcf_sr_regions_destroy).
        let names = crate::tbx::tbx_seqnames((*reg).tbx.cast::<crate::tbx::tbx_t>());
        (*reg).nseqs = names.len() as i32;
        (*reg).seq_names = libc::malloc(names.len() * size_of::<*mut c_char>()).cast::<*mut c_char>();
        if (*reg).seq_hash.is_null() {
            (*reg).seq_hash = crate::sam::khash_str2int_init().cast();
        }
        for (i, name) in names.iter().enumerate() {
            let mut name_c = name.clone();
            name_c.push(0);
            let dup = libc::strdup(name_c.as_ptr().cast::<c_char>());
            *(*reg).seq_names.add(i) = dup;
            crate::sam::khash_str2int_set((*reg).seq_hash.cast::<()>(), dup.cast::<u8>(), i as i32);
        }
        let mut fname_c = regions.to_vec();
        fname_c.push(0);
        (*reg).fname = libc::strdup(fname_c.as_ptr().cast::<c_char>());
        (*reg).is_bin = 1;
        NonNull::new(reg)
    }
}

// original: bcf_sr_regions_seek (htslib/synced_bcf_reader.c:1352)
pub(crate) unsafe fn bcf_sr_regions_seek(reg: &mut bcf_sr_regions_t, seq: &[u8]) -> i32 {
    unsafe {
        reg.iseq = -1;
        reg.start = -1;
        reg.end = -1;
        let mut seq_c = seq.to_vec();
        seq_c.push(0);
        if crate::sam::khash_str2int_get(
            reg.seq_hash.cast::<()>(),
            seq_c.as_ptr(),
            &mut reg.iseq,
        ) < 0
        {
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
            crate::tbx::tbx_itr_querys1(&mut *reg.tbx.cast::<crate::tbx::tbx_t>(), seq).cast();
        if !reg.itr.is_null() {
            return 0;
        }
        -1
    }
}

pub(crate) unsafe fn bcf_sr_regions_next(reg: &mut bcf_sr_regions_t) -> i32 {
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
        const TBX_UCSC: i32 = 0x10000;
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
                    reg.file = hts_open(reg.fname, b"r\0".as_ptr().cast::<c_char>()).cast();
                    if reg.file.is_null() {
                        bcf_sr_regions_destroy(reg);
                        return -1;
                    }
                    reg.is_bin = 0;
                }
                ret = if !reg.file.is_null() {
                    crate::hts::hts_getline(reg.file.cast(), KS_SEP_LINE as i32, line_ptr)
                } else {
                    -1
                };
                if ret < 0 {
                    reg.iseq = -1;
                    return -1;
                }
            }
            ret = regions_parse_line(
                reg.line.data.as_mut_ptr().cast::<c_char>(),
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
        // `chr` is now a NUL-terminated C string (chr_end was set to 0 above).
        if crate::sam::khash_str2int_get(reg.seq_hash.cast::<()>(), chr.cast::<u8>(), &mut reg.iseq)
            < 0
        {
            std::process::abort();
        }
        *chr_end = b'\t' as c_char;

        reg.start = from - 1;
        reg.end = to - 1;
        0
    }
}

// original: bcf_sr_regions_overlap (htslib/synced_bcf_reader.c:1525)
pub(crate) unsafe fn bcf_sr_regions_overlap(
    reg: &mut bcf_sr_regions_t,
    seq: &[u8],
    start: hts_pos_t,
    end: hts_pos_t,
) -> i32 {
    unsafe { bcf_sr_regions_overlap_inner(reg, seq, start, end, 1) }
}

unsafe fn bcf_sr_regions_overlap_inner(
    reg: &mut bcf_sr_regions_t,
    seq: &[u8],
    start: hts_pos_t,
    end: hts_pos_t,
    mut missed_reg_handler: i32,
) -> i32 {
    unsafe {
        let mut iseq = -1;
        let mut seq_c = seq.to_vec();
        seq_c.push(0);
        if crate::sam::khash_str2int_get(reg.seq_hash.cast::<()>(), seq_c.as_ptr(), &mut iseq) < 0 {
            return -1;
        }
        if missed_reg_handler != 0 && reg.missed_reg_handler.is_none() {
            missed_reg_handler = 0;
        }

        if reg.prev_seq == -1 || iseq != reg.prev_seq || reg.prev_start > start {
            if missed_reg_handler != 0 && reg.prev_seq != -1 && reg.iseq != -1 {
                bcf_sr_regions_flush(reg);
            }
            bcf_sr_regions_seek(reg, seq);
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
            if bcf_sr_regions_next(reg) < 0 {
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
pub(crate) unsafe fn bcf_sr_regions_flush(reg: &mut bcf_sr_regions_t) -> i32 {
    unsafe {
        let Some(handler) = reg.missed_reg_handler else {
            return 0;
        };
        if reg.prev_seq == -1 {
            return 0;
        }
        while bcf_sr_regions_next(reg) == 0 {
            handler(reg as *mut bcf_sr_regions_t, reg.missed_reg_data);
        }
        0
    }
}
