#!/usr/bin/env python3
"""Audit and optionally repair stale CCC translation metadata.

The CCC scaffold originally routed untranslated entries to
src/original_stubs.rs.  That file is no longer present in this tree, and many
of those generated Rust symbols now exist in real source modules.  This script
reports those stale stub mappings and, with --fix, updates only entries whose
recorded Rust symbol exists uniquely under src/.
"""

from __future__ import annotations

import argparse
import csv
import pathlib
import re
from dataclasses import dataclass


ROOT = pathlib.Path(__file__).resolve().parents[1]
STUB_PATH = "src/original_stubs.rs"
SAFE_AMBIGUOUS_ORIGINALS = {
    ("cram_set_voption", "htslib/cram/cram_io.c", "5692"): "src/cram.rs",
}
REVIEWED_PROMOTABLE_ALIASES = {
    # Rust cannot define the exported C variadic wrapper directly.  The upstream
    # wrapper only forwards to cram_set_voption, whose translated body is the
    # implementation to track for CCC purposes.
    ("cram_set_option", "htslib/cram/cram_io.c", "5674"): (
        "cram_cram_io_c_5692_cram_set_voption",
        "src/cram.rs",
    ),
    ("bgzf_thread_pool", "htslib/bgzf.c", "1740"): ("bgzf_thread_pool", "src/bgzf.rs"),
    ("bgzf_mt", "htslib/bgzf.c", "1783"): ("bgzf_mt", "src/bgzf.rs"),
    ("bgzf_mt", "htslib/bgzf.c", "1937"): ("bgzf_mt", "src/bgzf.rs"),
    ("bgzf_encode_func", "htslib/bgzf.c", "1330"): ("bgzf_encode_func", "src/bgzf.rs"),
    ("bgzf_encode_level0_func", "htslib/bgzf.c", "1345"): (
        "bgzf_encode_level0_func",
        "src/bgzf.rs",
    ),
    ("bgzf_decode_func", "htslib/bgzf.c", "1373"): ("bgzf_decode_func", "src/bgzf.rs"),
    ("lazy_flush", "htslib/bgzf.c", "1927"): ("lazy_flush", "src/bgzf.rs"),
    ("doit_square_u", "htslib/thread_pool.c", "1171"): (
        "unordered_square_in_only",
        "src/thread_pool.rs",
    ),
    ("test_square_u", "htslib/thread_pool.c", "1182"): (
        "original_test_square_u_in_only_jobs_flush_before_destroy",
        "src/thread_pool.rs",
    ),
    ("doit_square", "htslib/thread_pool.c", "1209"): (
        "ordered_square_with_slow_serial",
        "src/thread_pool.rs",
    ),
    ("test_square", "htslib/thread_pool.c", "1224"): (
        "original_test_square_nonblocking_dispatch_drains_ordered_results",
        "src/thread_pool.rs",
    ),
    ("test_squareB_dispatcher", "htslib/thread_pool.c", "1293"): (
        "test_squareb_dispatcher",
        "src/thread_pool.rs",
    ),
    ("test_squareB", "htslib/thread_pool.c", "1310"): (
        "original_test_squareb_dispatch_thread_consumes_until_sentinel",
        "src/thread_pool.rs",
    ),
    ("pipe_input_thread", "htslib/thread_pool.c", "1387"): (
        "pipe_input_thread",
        "src/thread_pool.rs",
    ),
    ("pipe_stage1", "htslib/thread_pool.c", "1408"): ("pipe_stage1", "src/thread_pool.rs"),
    ("pipe_stage1to2", "htslib/thread_pool.c", "1418"): (
        "pipe_stage1to2",
        "src/thread_pool.rs",
    ),
    ("pipe_stage2", "htslib/thread_pool.c", "1434"): ("pipe_stage2", "src/thread_pool.rs"),
    ("pipe_stage2to3", "htslib/thread_pool.c", "1444"): (
        "pipe_stage2to3",
        "src/thread_pool.rs",
    ),
    ("pipe_stage3", "htslib/thread_pool.c", "1460"): ("pipe_stage3", "src/thread_pool.rs"),
    ("pipe_output_thread", "htslib/thread_pool.c", "1468"): (
        "pipe_output_thread",
        "src/thread_pool.rs",
    ),
    ("test_pipe", "htslib/thread_pool.c", "1484"): (
        "original_test_pipe_runs_three_ordered_stages_to_eof",
        "src/thread_pool.rs",
    ),
    ("main", "htslib/thread_pool.c", "1517"): (
        "original_thread_pool_test_main_policy_routes_only_demo_modes",
        "src/thread_pool.rs",
    ),
    ("bcf_sr_set_opt", "htslib/synced_bcf_reader.c", "114"): (
        "bcf_sr_set_opt",
        "src/vcf.rs",
    ),
    ("init_filters", "htslib/synced_bcf_reader.c", "150"): (
        "init_filters",
        "src/vcf.rs",
    ),
    ("bcf_sr_seek_start", "htslib/synced_bcf_reader.c", "893"): (
        "bcf_sr_seek_start",
        "src/vcf.rs",
    ),
    ("bcf_sr_regions_alloc", "htslib/synced_bcf_reader.c", "1012"): (
        "bcf_sr_regions_alloc",
        "src/vcf.rs",
    ),
    ("_regions_add", "htslib/synced_bcf_reader.c", "1024"): (
        "bcf_sr_regions_add",
        "src/vcf.rs",
    ),
    ("_regions_init_string", "htslib/synced_bcf_reader.c", "1101"): (
        "regions_init_string",
        "src/vcf.rs",
    ),
    ("_regions_parse_line", "htslib/synced_bcf_reader.c", "1184"): (
        "regions_parse_line",
        "src/vcf.rs",
    ),
    ("bcf_sr_regions_destroy", "htslib/synced_bcf_reader.c", "1323"): (
        "bcf_sr_regions_destroy_translated",
        "src/vcf.rs",
    ),
    ("advance_creg", "htslib/synced_bcf_reader.c", "1369"): (
        "advance_creg",
        "src/vcf.rs",
    ),
    ("bcf_format_gt_v2", "htslib/vcf.c", "6345"): (
        "bcf_format_gt_v2",
        "src/vcf.rs",
    ),
    ("share_lock", "htslib/hfile_s3.c", "1133"): (
        "hfile_libcurl_c_309_share_lock",
        "src/hfile_libcurl.rs",
    ),
    ("share_unlock", "htslib/hfile_s3.c", "1138"): (
        "hfile_libcurl_c_314_share_unlock",
        "src/hfile_libcurl.rs",
    ),
    ("handle_bad_request", "htslib/hfile_s3.c", "1762"): (
        "hfile_s3_c_1055_handle_400_response",
        "src/hfile_s3.rs",
    ),
    ("hts_detect_format", "htslib/hts.c", "551"): ("hts_detect_format", "src/hts.rs"),
    ("hts_set_threads", "htslib/hts.c", "1922"): ("hts_set_threads", "src/hts.rs"),
    ("hts_set_thread_pool", "htslib/hts.c", "1934"): ("hts_set_thread_pool", "src/hts.rs"),
    ("hts_set_cache_size", "htslib/hts.c", "1945"): ("hts_set_cache_size", "src/hts.rs"),
    ("hts_set_fai_filename", "htslib/hts.c", "1951"): ("hts_set_fai_filename", "src/hts.rs"),
    ("hts_set_filter_expression", "htslib/hts.c", "1967"): (
        "hts_set_filter_expression",
        "src/hts.rs",
    ),
    ("hgetln_wrapper", "htslib/hts.c", "2031"): ("hgetln_wrapper", "src/hts.rs"),
    ("hts_getline", "htslib/hts.c", "2035"): ("hts_getline", "src/hts.rs"),
    ("compare_regions", "htslib/hts.c", "3361"): ("compare_regions_ref", "src/hts.rs"),
    ("hts_itr_querys", "htslib/hts.c", "4201"): ("hts_itr_querys", "src/hts.rs"),
    ("hts_itr_regions", "htslib/hts.c", "4217"): ("hts_itr_regions", "src/hts.rs"),
    ("hts_realloc_or_die", "htslib/hts.c", "5035"): ("hts_realloc_or_die", "src/hts.rs"),
    ("hts_itr_multi_cram", "htslib/hts.c", "3746"): ("hts_itr_multi_cram", "src/hts.rs"),
    ("sam_hdr_find_line_id", "htslib/header.c", "1725"): ("sam_hdr_find_line_id", "src/sam.rs"),
    ("sam_hdr_find_line_pos", "htslib/header.c", "1749"): ("sam_hdr_find_line_pos", "src/sam.rs"),
    ("sam_hdr_remove_line_id", "htslib/header.c", "1783"): ("sam_hdr_remove_line_id", "src/sam.rs"),
    ("sam_hdr_remove_line_pos", "htslib/header.c", "1822"): ("sam_hdr_remove_line_pos", "src/sam.rs"),
    ("sam_hdr_remove_lines", "htslib/header.c", "2070"): ("sam_hdr_remove_lines", "src/sam.rs"),
    ("sam_hdr_count_lines", "htslib/header.c", "2142"): ("sam_hdr_count_lines", "src/sam.rs"),
    ("sam_hdr_line_index", "htslib/header.c", "2185"): ("sam_hdr_line_index", "src/sam.rs"),
    ("sam_hdr_line_name", "htslib/header.c", "2235"): ("sam_hdr_line_name", "src/sam.rs"),
    ("sam_hdr_find_tag_id", "htslib/header.c", "2282"): ("sam_hdr_find_tag_id", "src/sam.rs"),
    ("sam_hdr_find_tag_pos", "htslib/header.c", "2314"): ("sam_hdr_find_tag_pos", "src/sam.rs"),
    ("bcf_sr_seek", "htslib/synced_bcf_reader.c", "908"): ("bcf_sr_seek", "src/vcf.rs"),
    ("bcf_sr_regions_init", "htslib/synced_bcf_reader.c", "1244"): (
        "bcf_sr_regions_init",
        "src/vcf.rs",
    ),
    ("bcf_sr_regions_seek", "htslib/synced_bcf_reader.c", "1348"): (
        "bcf_sr_regions_seek",
        "src/vcf.rs",
    ),
    ("bcf_sr_regions_next", "htslib/synced_bcf_reader.c", "1378"): (
        "bcf_sr_regions_next",
        "src/vcf.rs",
    ),
    ("bcf_sr_regions_overlap", "htslib/synced_bcf_reader.c", "1521"): (
        "bcf_sr_regions_overlap",
        "src/vcf.rs",
    ),
    ("bcf_sr_regions_flush", "htslib/synced_bcf_reader.c", "1555"): (
        "bcf_sr_regions_flush",
        "src/vcf.rs",
    ),
    ("bcf_hdr_sync", "htslib/vcf.c", "316"): ("bcf_hdr_sync", "src/vcf.rs"),
    ("bcf_hdr_parse_sample_line", "htslib/vcf.c", "286"): (
        "vcf_c_286_bcf_hdr_parse_sample_line",
        "src/vcf.rs",
    ),
    ("bcf_hdr_check_sanity", "htslib/vcf.c", "1269"): (
        "bcf_hdr_check_sanity",
        "src/vcf.rs",
    ),
    ("bcf_hdr_parse", "htslib/vcf.c", "1410"): ("bcf_hdr_parse", "src/vcf.rs"),
    ("bcf_hdr_append", "htslib/vcf.c", "1491"): ("bcf_hdr_append", "src/vcf.rs"),
    ("bcf_hdr_read", "htslib/vcf.c", "1710"): ("bcf_hdr_read", "src/vcf.rs"),
    ("bcf_read", "htslib/vcf.c", "2256"): ("bcf_read", "src/vcf.rs"),
    ("bcf_subset_format", "htslib/vcf.c", "2215"): ("bcf_subset_format", "src/vcf.rs"),
    ("vcf_parse", "htslib/vcf.c", "3987"): ("vcf_parse", "src/vcf.rs"),
    ("vcf_read", "htslib/vcf.c", "4170"): ("vcf_read", "src/vcf.rs"),
    ("vcf_format", "htslib/vcf.c", "4304"): ("vcf_format", "src/vcf.rs"),
    ("bcf_translate", "htslib/vcf.c", "5020"): ("bcf_translate", "src/vcf.rs"),
    ("bcf_update_info", "htslib/vcf.c", "5546"): ("bcf_update_info", "src/vcf.rs"),
    ("bcf_update_format", "htslib/vcf.c", "5710"): ("bcf_update_format", "src/vcf.rs"),
    ("bcf_update_alleles_str", "htslib/vcf.c", "5970"): (
        "bcf_update_alleles_str",
        "src/vcf.rs",
    ),
    ("bcf_hdr_name2id", "htslib/htslib/vcf.h", "1221"): ("bcf_hdr_name2id", "src/vcf.rs"),
    ("bcf_hdr_id2name", "htslib/htslib/vcf.h", "1222"): ("bcf_hdr_id2name", "src/vcf.rs"),
    ("bcf_seqname", "htslib/htslib/vcf.h", "1227"): ("bcf_seqname", "src/vcf.rs"),
    ("bcf_seqname_safe", "htslib/htslib/vcf.h", "1238"): ("bcf_seqname_safe", "src/vcf.rs"),
    ("bcf_update_info_int64", "htslib/htslib/vcf.h", "994"): (
        "bcf_update_info_int64",
        "src/vcf.rs",
    ),
    ("bcf_get_info_int64", "htslib/htslib/vcf.h", "1127"): (
        "bcf_get_info_int64",
        "src/vcf.rs",
    ),
    ("bcf_format_gt", "htslib/htslib/vcf.h", "1535"): ("bcf_format_gt", "src/vcf.rs"),
    ("bcf_idx_save", "htslib/vcf.c", "4822"): ("bcf_idx_save", "src/vcf.rs"),
    ("bcf_itr_querys1", "htslib/vcf.c", "4831"): ("bcf_itr_querys1", "src/tabix.rs"),
    ("bcf_sr_add_hreader", "htslib/synced_bcf_reader.c", "274"): (
        "bcf_sr_add_hreader",
        "src/vcf.rs",
    ),
    ("hts_lib_shutdown", "htslib/hts.c", "5151"): ("hts_lib_shutdown", "src/hts.rs"),
    ("sam_hdr_add_line", "htslib/header.c", "1692"): ("sam_hdr_add_line", "src/sam.rs"),
    ("sam_hdr_add_pg", "htslib/header.c", "2612"): ("sam_hdr_add_pg", "src/sam.rs"),
    ("sam_hrecs_update_hashes", "htslib/header.c", "141"): (
        "sam_hrecs_update_hashes",
        "src/sam.rs",
    ),
    ("sam_hrecs_remove_hash_entry", "htslib/header.c", "413"): (
        "sam_hrecs_remove_hash_entry",
        "src/sam.rs",
    ),
    ("sam_hrecs_global_list_add", "htslib/header.c", "500"): (
        "sam_hrecs_global_list_add",
        "src/sam.rs",
    ),
    ("sam_hrecs_free_tags", "htslib/header.c", "694"): (
        "sam_hrecs_free_tags",
        "src/sam.rs",
    ),
    ("sam_hrecs_remove_line", "htslib/header.c", "703"): (
        "sam_hrecs_remove_line",
        "src/sam.rs",
    ),
    ("build_header_line", "htslib/header.c", "743"): ("build_header_line", "src/sam.rs"),
    ("parse_comment_line", "htslib/header.c", "800"): ("parse_comment_line", "src/sam.rs"),
    ("parse_noncomment_line", "htslib/header.c", "826"): ("parse_noncomment_line", "src/sam.rs"),
    ("sam_hrecs_parse_single_line", "htslib/header.c", "906"): (
        "sam_hrecs_parse_single_line",
        "src/sam.rs",
    ),
    ("sam_hrecs_parse_lines", "htslib/header.c", "995"): (
        "sam_hrecs_parse_lines",
        "src/sam.rs",
    ),
    ("sam_hdr_update_target_arrays", "htslib/header.c", "1073"): (
        "sam_hdr_update_target_arrays",
        "src/sam.rs",
    ),
    ("rebuild_target_arrays", "htslib/header.c", "1153"): (
        "rebuild_target_arrays",
        "src/sam.rs",
    ),
    ("sam_hrecs_refs_from_targets_array", "htslib/header.c", "1181"): (
        "sam_hrecs_refs_from_targets_array",
        "src/sam.rs",
    ),
    ("add_stub_ref_sq_lines", "htslib/header.c", "1266"): (
        "add_stub_ref_sq_lines",
        "src/sam.rs",
    ),
    ("sam_hdr_fill_hrecs", "htslib/header.c", "1289"): (
        "sam_hdr_fill_hrecs",
        "src/sam.rs",
    ),
    ("sam_hdr_build_from_sam_file", "htslib/header.c", "1353"): (
        "sam_hdr_build_from_sam_file",
        "src/sam.rs",
    ),
    ("rebuild_hash", "htslib/header.c", "1983"): ("rebuild_hash", "src/sam.rs"),
    ("sam_hrecs_dup", "htslib/header.c", "2801"): ("sam_hrecs_dup", "src/sam.rs"),
    ("sam_hrecs_find_rg", "htslib/header.c", "3068"): ("sam_hrecs_find_rg", "src/sam.rs"),
    ("sam_hrecs_dump", "htslib/header.c", "3076"): ("sam_hrecs_dump", "src/sam.rs"),
    ("sam_hrecs_sort_order", "htslib/header.c", "3128"): (
        "sam_hrecs_sort_order",
        "src/sam.rs",
    ),
    ("sam_hrecs_group_order", "htslib/header.c", "3154"): (
        "sam_hrecs_group_order",
        "src/sam.rs",
    ),
}
REVIEWED_NONPROMOTABLE_ORIGINALS = {
    # Macro declaration marker; the nearby TYPEKEY helper is not the KHASH_DECLARE
    # implementation and should not receive this mapping.
    ("KHASH_DECLARE", "htslib/header.c", "44"),
}
REVIEWED_NONPROMOTABLE_STUBS = {
    # Logging / printf macro metadata folded into direct Rust call sites.
    ("debug", "htslib/test/test-regidx.c", "45"),
    ("info", "htslib/test/test-regidx.c", "55"),
    ("HTS_FORMAT", "htslib/test/test-regidx.c", "64"),
    ("HTS_FORMAT", "htslib/bgzip.c", "51"),
    ("HTS_FORMAT", "htslib/annot-tsv.c", "119"),
    # Test-only debug-printing macro; not production behavior.
    ("DBG_OUT", "htslib/thread_pool.c", "69"),
    # S3 helper state folded into Rust data-layout initialization/cleanup.
    ("initialise_authorisation_values", "htslib/hfile_s3.c", "1143"),
    ("clear_authorisation_values", "htslib/hfile_s3.c", "1153"),
    ("free_authorisation_values", "htslib/hfile_s3.c", "1163"),
    ("initialise_local", "htslib/hfile_s3.c", "1268"),
    # Direct S3 callbacks/backends are either bridged write-side callbacks or
    # superseded by translated hfile/libcurl read dispatch.
    ("response_callback", "htslib/hfile_s3.c", "1292"),
    ("add_header", "htslib/hfile_s3.c", "1304"),
    ("set_html_headers", "htslib/hfile_s3.c", "1318"),
    ("upload_callback", "htslib/hfile_s3.c", "1545"),
    ("get_part", "htslib/hfile_s3.c", "1878"),
    ("s3_read", "htslib/hfile_s3.c", "1949"),
    ("initialise_download", "htslib/hfile_s3.c", "2074"),
    ("s3_close", "htslib/hfile_s3.c", "2083"),
    ("s3_read_open", "htslib/hfile_s3.c", "2230"),
    # Synced-reader and sort debug/internal rows covered by higher-level local
    # paths or intentionally not represented as public Rust functions.
    ("debug_buffer", "htslib/synced_bcf_reader.c", "518"),
    ("debug_buffers", "htslib/synced_bcf_reader.c", "531"),
    ("has_filter", "htslib/synced_bcf_reader.c", "543"),
    ("regions_cmp", "htslib/synced_bcf_reader.c", "1060"),
    ("kbs_logical_and", "htslib/bcf_sr_sort.c", "43"),
    ("kbs_bitwise_or", "htslib/bcf_sr_sort.c", "54"),
    ("bcf_sr_init_scores", "htslib/bcf_sr_sort.c", "61"),
    ("multi_is_exact", "htslib/bcf_sr_sort.c", "105"),
    ("multi_is_subset", "htslib/bcf_sr_sort.c", "133"),
    ("pairing_score", "htslib/bcf_sr_sort.c", "153"),
    ("remove_vset", "htslib/bcf_sr_sort.c", "193"),
    ("merge_vsets", "htslib/bcf_sr_sort.c", "208"),
    ("push_vset", "htslib/bcf_sr_sort.c", "233"),
    ("cmpstringp", "htslib/bcf_sr_sort.c", "258"),
    ("debug_vsets", "htslib/bcf_sr_sort.c", "265"),
    ("debug_vbuf", "htslib/bcf_sr_sort.c", "287"),
    ("grp_create_key", "htslib/bcf_sr_sort.c", "303"),
    # VCF header inline helper rows are not standalone public Rust API symbols.
    ("bcf_gt2alleles", "htslib/htslib/vcf.h", "1042"),
    ("bcf_itr_next", "htslib/htslib/vcf.h", "1327"),
    ("bcf_float_set", "htslib/htslib/vcf.h", "1498"),
    ("bcf_float_is_missing", "htslib/htslib/vcf.h", "1506"),
    ("bcf_float_is_vector_end", "htslib/htslib/vcf.h", "1512"),
    ("bcf_enc_size", "htslib/htslib/vcf.h", "1540"),
    ("bcf_enc_inttype", "htslib/htslib/vcf.h", "1575"),
    ("bcf_enc_int1", "htslib/htslib/vcf.h", "1582"),
    ("bcf_dec_int1", "htslib/htslib/vcf.h", "1630"),
    ("bcf_dec_typed_int1", "htslib/htslib/vcf.h", "1665"),
    ("bcf_dec_size", "htslib/htslib/vcf.h", "1670"),
    # Static VCF helpers/debug fragments folded into exported parser/formatter
    # implementations or intentionally not represented as standalone Rust APIs.
    ("xstreq", "htslib/vcf.c", "63"),
    ("get_hdr_aux", "htslib/vcf.c", "125"),
    ("hdr_bgzf_private_data_cleanup", "htslib/vcf.c", "212"),
    ("find_chrom_header_line", "htslib/vcf.c", "218"),
    ("bcf_hrec_debug", "htslib/vcf.c", "413"),
    ("bcf_header_debug", "htslib/vcf.c", "422"),
    ("bcf_hrec_set_type", "htslib/vcf.c", "527"),
    ("bcf_hrec_check", "htslib/vcf.c", "596"),
    ("is_escaped", "htslib/vcf.c", "647"),
    ("bcf_dec_typed_int1_safe", "htslib/vcf.c", "1918"),
    ("bcf_dec_size_safe", "htslib/vcf.c", "1950"),
    ("get_type_name", "htslib/vcf.c", "1965"),
    ("updatephasing", "htslib/vcf.c", "1985"),
    ("bcf_record_check_err", "htslib/vcf.c", "2031"),
    ("add_missing_contig_hrec", "htslib/vcf.c", "2571"),
    ("_bcf_hrec_format", "htslib/vcf.c", "2712"),
    ("bcf_enc_long1", "htslib/vcf.c", "2925"),
    ("serialize_float_array", "htslib/vcf.c", "2944"),
    ("bcf_fmt_array1", "htslib/vcf.c", "2979"),
    ("align_mem", "htslib/vcf.c", "3124"),
    ("vcf_parse_format_empty1", "htslib/vcf.c", "3137"),
    ("vcf_parse_format_check7", "htslib/vcf.c", "3664"),
    ("bcf_unpack_fmt_core1", "htslib/vcf.c", "4178"),
    ("bcf_unpack_info_core1", "htslib/vcf.c", "4192"),
    ("idx_calc_n_lvls_ids", "htslib/vcf.c", "4636"),
    ("bcf_hdr_name2id_wrapper", "htslib/vcf.c", "4827"),
    ("add_desc_to_buffer", "htslib/vcf.c", "6287"),
    ("bcf_get_version", "htslib/vcf.c", "145"),
    ("bcf_hdr_incr_ref", "htslib/vcf.c", "196"),
    ("bcf_hdr_decr_ref", "htslib/vcf.c", "202"),
    ("bcf_hdr_add_sample_len", "htslib/vcf.c", "232"),
    # Disabled debug helper in htslib/region.c.
    ("reg_print", "htslib/region.c", "57"),
}

FN_RE = re.compile(
    r"\b(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"C\"\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b"
)
FIELD_RE = re.compile(r'^(rust|rust_path|other|other_path|other_line)\s*=\s*(?:"([^"]*)"|([0-9]+))', re.M)
ORIGINAL_RE = re.compile(r"//\s*original:\s*([^\s(]+)\s*\(([^:]+):(\d+)\)")


@dataclass
class FunctionHit:
    name: str
    path: str
    line: int


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def scan_rust_functions() -> tuple[dict[str, list[FunctionHit]], dict[tuple[str, str, str], list[FunctionHit]]]:
    by_name: dict[str, list[FunctionHit]] = {}
    by_original: dict[tuple[str, str, str], list[FunctionHit]] = {}

    for path in sorted((ROOT / "src").glob("**/*.rs")):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(errors="ignore")

        for match in FN_RE.finditer(text):
            by_name.setdefault(match.group(1), []).append(
                FunctionHit(match.group(1), rel, line_number(text, match.start()))
            )

        for match in ORIGINAL_RE.finditer(text):
            rest = text[match.end() : match.end() + 600]
            fn_match = FN_RE.search(rest)
            if not fn_match:
                continue
            by_original.setdefault((match.group(1), match.group(2), match.group(3)), []).append(
                FunctionHit(fn_match.group(1), rel, line_number(text, match.start()))
            )

    return by_name, by_original


def parse_mapping_entries(mapping_text: str) -> list[dict[str, str]]:
    entries = []
    for block in mapping_text.split("[[entries]]")[1:]:
        fields = {}
        for key, quoted, number in FIELD_RE.findall(block):
            fields[key] = quoted or number
        entries.append(fields)
    return entries


def load_order_rows(path: pathlib.Path) -> list[dict[str, str]]:
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle))


def write_order_rows(path: pathlib.Path, rows: list[dict[str, str]]) -> None:
    with path.open("w", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=["name", "file", "line_start", "scc_id", "scc_kind", "translated"],
            lineterminator="\n",
        )
        writer.writeheader()
        writer.writerows(rows)


def audit_entries(
    entries: list[dict[str, str]],
    by_name: dict[str, list[FunctionHit]],
    by_original: dict[tuple[str, str, str], list[FunctionHit]],
    order_rows: list[dict[str, str]],
) -> tuple[list[dict[str, str]], set[tuple[str, str, str]]]:
    order_state = {(r["name"], r["file"], r["line_start"]): r["translated"] for r in order_rows}
    exact_keys: set[tuple[str, str, str]] = set()
    report: list[dict[str, str]] = []

    for entry in entries:
        if entry.get("rust_path") != STUB_PATH:
            continue

        key = (entry["other"], entry["other_path"], entry["other_line"])
        exact_hits = by_name.get(entry["rust"], [])
        original_hits = by_original.get(key, [])
        original_name_hits = by_name.get(entry["other"], [])

        if len(exact_hits) == 1:
            status = "exact_rust_symbol_found"
            candidate = exact_hits[0]
            exact_keys.add(key)
        elif len(exact_hits) > 1 and len(original_hits) == 1:
            status = "ambiguous_rust_symbol_original_comment_candidate"
            candidate = original_hits[0]
            exact_keys.add(key)
        elif len(exact_hits) > 1 and key in SAFE_AMBIGUOUS_ORIGINALS:
            preferred_path = SAFE_AMBIGUOUS_ORIGINALS[key]
            preferred_hits = [hit for hit in original_hits if hit.path == preferred_path]
            if len(preferred_hits) == 1:
                status = "ambiguous_rust_symbol_original_comment_candidate"
                candidate = preferred_hits[0]
                exact_keys.add(key)
            else:
                status = "ambiguous_rust_symbol"
                candidate = exact_hits[0]
        elif len(exact_hits) > 1:
            status = "ambiguous_rust_symbol"
            candidate = exact_hits[0]
        elif key in REVIEWED_PROMOTABLE_ALIASES:
            status = "reviewed_promotable_alias"
            alias_rust, alias_path = REVIEWED_PROMOTABLE_ALIASES[key]
            alias_hits = [
                hit for hit in by_name.get(alias_rust, []) if hit.path == alias_path
            ]
            candidate = alias_hits[0] if len(alias_hits) == 1 else FunctionHit(alias_rust, alias_path, 0)
        elif order_state.get(key) == "TRUE" and len(original_name_hits) == 1:
            status = "order_true_exact_original_symbol"
            candidate = original_name_hits[0]
        elif key in REVIEWED_NONPROMOTABLE_STUBS:
            status = "reviewed_nonpromotable_stub"
            candidate = FunctionHit("", "", 0)
        elif len(original_hits) == 1 and key in REVIEWED_NONPROMOTABLE_ORIGINALS:
            status = "reviewed_nonpromotable_original_comment"
            candidate = original_hits[0]
        elif len(original_hits) == 1:
            status = "original_comment_candidate"
            candidate = original_hits[0]
        elif len(original_hits) > 1:
            status = "ambiguous_original_comment"
            candidate = original_hits[0]
        else:
            status = "unresolved_stub"
            candidate = FunctionHit("", "", 0)

        report.append(
            {
                "status": status,
                "other_path": entry["other_path"],
                "other_line": entry["other_line"],
                "other": entry["other"],
                "current_rust": entry["rust"],
                "current_rust_path": entry["rust_path"],
                "candidate_rust": candidate.name,
                "candidate_path": candidate.path,
                "candidate_line": str(candidate.line) if candidate.line else "",
                "order_translated": order_state.get(key, ""),
            }
        )

    return report, exact_keys


def write_report(path: pathlib.Path, rows: list[dict[str, str]]) -> None:
    fields = [
        "status",
        "other_path",
        "other_line",
        "other",
        "current_rust",
        "current_rust_path",
        "candidate_rust",
        "candidate_path",
        "candidate_line",
        "order_translated",
    ]
    with path.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def promoted_mapping(row: dict[str, str]) -> tuple[str, str] | None:
    if row["status"] == "exact_rust_symbol_found":
        return row["current_rust"], row["candidate_path"]
    if row["status"] == "original_comment_candidate" and row["candidate_rust"] == row["other"]:
        return row["candidate_rust"], row["candidate_path"]
    if row["status"] == "ambiguous_rust_symbol_original_comment_candidate":
        return row["candidate_rust"], row["candidate_path"]
    if row["status"] == "reviewed_promotable_alias":
        return row["candidate_rust"], row["candidate_path"]
    if row["status"] == "order_true_exact_original_symbol":
        return row["candidate_rust"], row["candidate_path"]
    return None


def update_mapping_text(mapping_text: str, report: list[dict[str, str]]) -> tuple[str, int]:
    promoted_by_key = {}
    for row in report:
        promoted = promoted_mapping(row)
        if promoted:
            promoted_by_key[(row["other"], row["other_path"], row["other_line"])] = promoted

    changed = 0
    blocks = mapping_text.split("[[entries]]")
    output = [blocks[0]]
    for block in blocks[1:]:
        fields = {}
        for key, quoted, number in FIELD_RE.findall(block):
            fields[key] = quoted or number
        promoted = promoted_by_key.get((fields.get("other", ""), fields.get("other_path", ""), fields.get("other_line", "")))
        if fields.get("rust_path") == STUB_PATH and promoted:
            candidate_rust, candidate_path = promoted
            block, rust_count = re.subn(
                r'rust = "[^"]*"',
                f'rust = "{candidate_rust}"',
                block,
                count=1,
            )
            block, path_count = re.subn(
                rf'rust_path = "{re.escape(STUB_PATH)}"',
                f'rust_path = "{candidate_path}"',
                block,
                count=1,
            )
            if rust_count or path_count:
                changed += 1
        output.append("[[entries]]" + block)
    return "".join(output), changed


def update_order_rows(rows: list[dict[str, str]], exact_keys: set[tuple[str, str, str]]) -> int:
    changed = 0
    for row in rows:
        key = (row["name"], row["file"], row["line_start"])
        if key in exact_keys and row["translated"] != "TRUE":
            row["translated"] = "TRUE"
            changed += 1
    return changed


def promoted_order_keys(report: list[dict[str, str]], exact_keys: set[tuple[str, str, str]]) -> set[tuple[str, str, str]]:
    keys = set(exact_keys)
    for row in report:
        if promoted_mapping(row):
            keys.add((row["other"], row["other_path"], row["other_line"]))
    return keys


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fix", action="store_true", help="update exact stale stub mappings and order flags")
    parser.add_argument(
        "--report",
        default="ccc_stale_stub_report.csv",
        help="CSV report path relative to the repository root",
    )
    args = parser.parse_args()

    mapping_path = ROOT / "ccc_mapping.toml"
    order_path = ROOT / "ccc_order_current.csv"
    report_path = ROOT / args.report

    mapping_text = mapping_path.read_text()
    entries = parse_mapping_entries(mapping_text)
    order_rows = load_order_rows(order_path)
    by_name, by_original = scan_rust_functions()
    report, exact_keys = audit_entries(entries, by_name, by_original, order_rows)
    write_report(report_path, report)

    status_counts: dict[str, int] = {}
    for row in report:
        status_counts[row["status"]] = status_counts.get(row["status"], 0) + 1

    mapping_updates = 0
    order_updates = 0
    if args.fix:
        updated_mapping, mapping_updates = update_mapping_text(mapping_text, report)
        mapping_path.write_text(updated_mapping)
        order_updates = update_order_rows(order_rows, promoted_order_keys(report, exact_keys))
        write_order_rows(order_path, order_rows)

    print(f"stub_entries={len(report)}")
    for status, count in sorted(status_counts.items()):
        print(f"{status}={count}")
    print(f"mapping_updates={mapping_updates}")
    print(f"order_updates={order_updates}")
    print(f"report={report_path.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
