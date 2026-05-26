#include <stdint.h>
#include <stdio.h>

#include "htslib/sam.h"

int main(int argc, char **argv) {
    if (argc < 2 || argc > 3) {
        fprintf(stderr, "usage: count_bam_c <bam> [region]\n");
        return 2;
    }

    htsFile *fp = hts_open(argv[1], "r");
    if (!fp) return 1;

    sam_hdr_t *hdr = sam_hdr_read(fp);
    if (!hdr) return 1;

    bam1_t *rec = bam_init1();
    if (!rec) return 1;

    uint64_t count = 0;
    if (argc == 3) {
        hts_idx_t *idx = sam_index_load(fp, argv[1]);
        if (!idx) return 1;

        hts_itr_t *itr = sam_itr_querys(idx, hdr, argv[2]);
        if (!itr) return 1;

        while (sam_itr_next(fp, itr, rec) >= 0) count++;
        hts_itr_destroy(itr);
        hts_idx_destroy(idx);
    } else {
        while (sam_read1(fp, hdr, rec) >= 0) count++;
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    if (hts_close(fp) != 0) return 1;

    printf("%llu\n", (unsigned long long)count);
    return 0;
}
