/* Reference C implementation for the bam_to_cram benchmark workload.
 *
 * Converts a BAM file to a CRAM file using `cram,no_ref` so a reference
 * isn't required. Mirrors the Rust src/bin/bam_to_cram.rs binary used in
 * the same perf comparison.
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include "htslib/sam.h"

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "usage: bam_to_cram_c <in.bam> <out.cram>\n");
        return 2;
    }
    htsFile *in_fp = hts_open(argv[1], "rb");
    if (!in_fp) { fprintf(stderr, "open in failed\n"); return 1; }
    sam_hdr_t *hdr = sam_hdr_read(in_fp);
    if (!hdr) { fprintf(stderr, "header read failed\n"); return 1; }

    htsFormat fmt = {0};
    if (hts_parse_format(&fmt, "cram,no_ref") < 0) {
        fprintf(stderr, "parse_format failed\n"); return 1;
    }
    htsFile *out_fp = hts_open_format(argv[2], "wc", &fmt);
    if (!out_fp) { fprintf(stderr, "open out failed\n"); return 1; }
    if (sam_hdr_write(out_fp, hdr) < 0) {
        fprintf(stderr, "sam_hdr_write failed\n"); return 1;
    }

    bam1_t *rec = bam_init1();
    if (!rec) return 1;
    uint64_t n = 0;
    while (sam_read1(in_fp, hdr, rec) >= 0) {
        if (sam_write1(out_fp, hdr, rec) < 0) {
            fprintf(stderr, "sam_write1 failed at record %llu\n",
                    (unsigned long long)n);
            return 1;
        }
        n++;
    }
    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    if (hts_close(out_fp) != 0) return 1;
    if (hts_close(in_fp) != 0) return 1;
    fprintf(stderr, "done, %llu records\n", (unsigned long long)n);
    return 0;
}
