#[repr(C)]
pub struct hts_md5_context {
    lo: u32,
    hi: u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    buffer: [u8; 64],
    block: [u32; 16],
}

/// Processes whole 64-byte blocks from `data` (the first `size` bytes, which the
/// caller guarantees to be a non-zero multiple of 64) and returns the remaining
/// slice past the consumed blocks.
pub fn md5_c_119_body<'a>(
    ctx: &mut hts_md5_context,
    data: &'a [u8],
    mut size: usize,
) -> &'a [u8] {
    macro_rules! f {
        ($x:expr, $y:expr, $z:expr) => {
            ($z) ^ (($x) & (($y) ^ ($z)))
        };
    }
    macro_rules! g {
        ($x:expr, $y:expr, $z:expr) => {
            ($y) ^ (($z) & (($x) ^ ($y)))
        };
    }
    macro_rules! h {
        ($x:expr, $y:expr, $z:expr) => {
            (($x) ^ ($y)) ^ ($z)
        };
    }
    macro_rules! h2 {
        ($x:expr, $y:expr, $z:expr) => {
            ($x) ^ (($y) ^ ($z))
        };
    }
    macro_rules! i {
        ($x:expr, $y:expr, $z:expr) => {
            ($y) ^ (($x) | !($z))
        };
    }
    macro_rules! step {
        ($fun:ident, $a:ident, $b:ident, $c:ident, $d:ident, $x:expr, $t:expr, $s:expr) => {{
            $a = $a.wrapping_add($fun!($b, $c, $d).wrapping_add($x).wrapping_add($t));
            $a = ($a << $s) | (($a & 0xffff_ffff) >> (32 - $s));
            $a = $a.wrapping_add($b);
        }};
    }
    macro_rules! set {
        ($block:expr, $ctx:ident, $n:expr) => {{
            let value = ($block[$n * 4] as u32)
                | (($block[$n * 4 + 1] as u32) << 8)
                | (($block[$n * 4 + 2] as u32) << 16)
                | (($block[$n * 4 + 3] as u32) << 24);
            $ctx.block[$n] = value;
            value
        }};
    }
    macro_rules! get {
        ($ctx:ident, $n:expr) => {
            $ctx.block[$n]
        };
    }

    let mut offset = 0usize;
    let mut a = ctx.a;
    let mut b = ctx.b;
    let mut c = ctx.c;
    let mut d = ctx.d;

    loop {
        let saved_a = a;
        let saved_b = b;
        let saved_c = c;
        let saved_d = d;

        let p = &data[offset..offset + 64];

        step!(f, a, b, c, d, set!(p, ctx, 0), 0xd76aa478, 7);
        step!(f, d, a, b, c, set!(p, ctx, 1), 0xe8c7b756, 12);
        step!(f, c, d, a, b, set!(p, ctx, 2), 0x242070db, 17);
        step!(f, b, c, d, a, set!(p, ctx, 3), 0xc1bdceee, 22);
        step!(f, a, b, c, d, set!(p, ctx, 4), 0xf57c0faf, 7);
        step!(f, d, a, b, c, set!(p, ctx, 5), 0x4787c62a, 12);
        step!(f, c, d, a, b, set!(p, ctx, 6), 0xa8304613, 17);
        step!(f, b, c, d, a, set!(p, ctx, 7), 0xfd469501, 22);
        step!(f, a, b, c, d, set!(p, ctx, 8), 0x698098d8, 7);
        step!(f, d, a, b, c, set!(p, ctx, 9), 0x8b44f7af, 12);
        step!(f, c, d, a, b, set!(p, ctx, 10), 0xffff5bb1, 17);
        step!(f, b, c, d, a, set!(p, ctx, 11), 0x895cd7be, 22);
        step!(f, a, b, c, d, set!(p, ctx, 12), 0x6b901122, 7);
        step!(f, d, a, b, c, set!(p, ctx, 13), 0xfd987193, 12);
        step!(f, c, d, a, b, set!(p, ctx, 14), 0xa679438e, 17);
        step!(f, b, c, d, a, set!(p, ctx, 15), 0x49b40821, 22);

        step!(g, a, b, c, d, get!(ctx, 1), 0xf61e2562, 5);
        step!(g, d, a, b, c, get!(ctx, 6), 0xc040b340, 9);
        step!(g, c, d, a, b, get!(ctx, 11), 0x265e5a51, 14);
        step!(g, b, c, d, a, get!(ctx, 0), 0xe9b6c7aa, 20);
        step!(g, a, b, c, d, get!(ctx, 5), 0xd62f105d, 5);
        step!(g, d, a, b, c, get!(ctx, 10), 0x02441453, 9);
        step!(g, c, d, a, b, get!(ctx, 15), 0xd8a1e681, 14);
        step!(g, b, c, d, a, get!(ctx, 4), 0xe7d3fbc8, 20);
        step!(g, a, b, c, d, get!(ctx, 9), 0x21e1cde6, 5);
        step!(g, d, a, b, c, get!(ctx, 14), 0xc33707d6, 9);
        step!(g, c, d, a, b, get!(ctx, 3), 0xf4d50d87, 14);
        step!(g, b, c, d, a, get!(ctx, 8), 0x455a14ed, 20);
        step!(g, a, b, c, d, get!(ctx, 13), 0xa9e3e905, 5);
        step!(g, d, a, b, c, get!(ctx, 2), 0xfcefa3f8, 9);
        step!(g, c, d, a, b, get!(ctx, 7), 0x676f02d9, 14);
        step!(g, b, c, d, a, get!(ctx, 12), 0x8d2a4c8a, 20);

        step!(h, a, b, c, d, get!(ctx, 5), 0xfffa3942, 4);
        step!(h2, d, a, b, c, get!(ctx, 8), 0x8771f681, 11);
        step!(h, c, d, a, b, get!(ctx, 11), 0x6d9d6122, 16);
        step!(h2, b, c, d, a, get!(ctx, 14), 0xfde5380c, 23);
        step!(h, a, b, c, d, get!(ctx, 1), 0xa4beea44, 4);
        step!(h2, d, a, b, c, get!(ctx, 4), 0x4bdecfa9, 11);
        step!(h, c, d, a, b, get!(ctx, 7), 0xf6bb4b60, 16);
        step!(h2, b, c, d, a, get!(ctx, 10), 0xbebfbc70, 23);
        step!(h, a, b, c, d, get!(ctx, 13), 0x289b7ec6, 4);
        step!(h2, d, a, b, c, get!(ctx, 0), 0xeaa127fa, 11);
        step!(h, c, d, a, b, get!(ctx, 3), 0xd4ef3085, 16);
        step!(h2, b, c, d, a, get!(ctx, 6), 0x04881d05, 23);
        step!(h, a, b, c, d, get!(ctx, 9), 0xd9d4d039, 4);
        step!(h2, d, a, b, c, get!(ctx, 12), 0xe6db99e5, 11);
        step!(h, c, d, a, b, get!(ctx, 15), 0x1fa27cf8, 16);
        step!(h2, b, c, d, a, get!(ctx, 2), 0xc4ac5665, 23);

        step!(i, a, b, c, d, get!(ctx, 0), 0xf4292244, 6);
        step!(i, d, a, b, c, get!(ctx, 7), 0x432aff97, 10);
        step!(i, c, d, a, b, get!(ctx, 14), 0xab9423a7, 15);
        step!(i, b, c, d, a, get!(ctx, 5), 0xfc93a039, 21);
        step!(i, a, b, c, d, get!(ctx, 12), 0x655b59c3, 6);
        step!(i, d, a, b, c, get!(ctx, 3), 0x8f0ccc92, 10);
        step!(i, c, d, a, b, get!(ctx, 10), 0xffeff47d, 15);
        step!(i, b, c, d, a, get!(ctx, 1), 0x85845dd1, 21);
        step!(i, a, b, c, d, get!(ctx, 8), 0x6fa87e4f, 6);
        step!(i, d, a, b, c, get!(ctx, 15), 0xfe2ce6e0, 10);
        step!(i, c, d, a, b, get!(ctx, 6), 0xa3014314, 15);
        step!(i, b, c, d, a, get!(ctx, 13), 0x4e0811a1, 21);
        step!(i, a, b, c, d, get!(ctx, 4), 0xf7537e82, 6);
        step!(i, d, a, b, c, get!(ctx, 11), 0xbd3af235, 10);
        step!(i, c, d, a, b, get!(ctx, 2), 0x2ad7d2bb, 15);
        step!(i, b, c, d, a, get!(ctx, 9), 0xeb86d391, 21);

        a = a.wrapping_add(saved_a);
        b = b.wrapping_add(saved_b);
        c = c.wrapping_add(saved_c);
        d = d.wrapping_add(saved_d);

        offset += 64;
        size = size.wrapping_sub(64);
        if size == 0 {
            break;
        }
    }

    ctx.a = a;
    ctx.b = b;
    ctx.c = c;
    ctx.d = d;
    &data[offset..]
}

pub fn md5_c_226_hts_md5_reset(ctx: &mut hts_md5_context) {
    ctx.a = 0x67452301;
    ctx.b = 0xefcdab89;
    ctx.c = 0x98badcfe;
    ctx.d = 0x10325476;
    ctx.lo = 0;
    ctx.hi = 0;
}

pub fn md5_c_237_hts_md5_update(ctx: &mut hts_md5_context, data: &[u8], size: usize) {
    let mut data = data;
    let mut size = size;

    let saved_lo = ctx.lo;
    ctx.lo = saved_lo.wrapping_add(size as u32) & 0x1fff_ffff;
    if ctx.lo < saved_lo {
        ctx.hi = ctx.hi.wrapping_add(1);
    }
    ctx.hi = ctx.hi.wrapping_add((size >> 29) as u32);

    let used = (saved_lo & 0x3f) as usize;
    if used != 0 {
        let available = 64 - used;
        if size < available {
            ctx.buffer[used..used + size].copy_from_slice(&data[..size]);
            return;
        }
        ctx.buffer[used..used + available].copy_from_slice(&data[..available]);
        data = &data[available..];
        size -= available;
        let buffer = ctx.buffer;
        md5_c_119_body(ctx, &buffer, 64);
    }

    if size >= 64 {
        let consumed = size & !0x3f;
        data = md5_c_119_body(ctx, data, consumed);
        size &= 0x3f;
    }

    ctx.buffer[..size].copy_from_slice(&data[..size]);
}

pub fn md5_c_271_hts_md5_final(result: &mut [u8], ctx: &mut hts_md5_context) {
    let mut used = (ctx.lo & 0x3f) as usize;
    ctx.buffer[used] = 0x80;
    used += 1;

    let mut available = 64 - used;
    if available < 8 {
        ctx.buffer[used..used + available].fill(0);
        let buffer = ctx.buffer;
        md5_c_119_body(ctx, &buffer, 64);
        used = 0;
        available = 64;
    }

    ctx.buffer[used..used + (available - 8)].fill(0);

    ctx.lo <<= 3;
    ctx.buffer[56] = ctx.lo as u8;
    ctx.buffer[57] = (ctx.lo >> 8) as u8;
    ctx.buffer[58] = (ctx.lo >> 16) as u8;
    ctx.buffer[59] = (ctx.lo >> 24) as u8;
    ctx.buffer[60] = ctx.hi as u8;
    ctx.buffer[61] = (ctx.hi >> 8) as u8;
    ctx.buffer[62] = (ctx.hi >> 16) as u8;
    ctx.buffer[63] = (ctx.hi >> 24) as u8;

    let buffer = ctx.buffer;
    md5_c_119_body(ctx, &buffer, 64);

    result[0] = ctx.a as u8;
    result[1] = (ctx.a >> 8) as u8;
    result[2] = (ctx.a >> 16) as u8;
    result[3] = (ctx.a >> 24) as u8;
    result[4] = ctx.b as u8;
    result[5] = (ctx.b >> 8) as u8;
    result[6] = (ctx.b >> 16) as u8;
    result[7] = (ctx.b >> 24) as u8;
    result[8] = ctx.c as u8;
    result[9] = (ctx.c >> 8) as u8;
    result[10] = (ctx.c >> 16) as u8;
    result[11] = (ctx.c >> 24) as u8;
    result[12] = ctx.d as u8;
    result[13] = (ctx.d >> 8) as u8;
    result[14] = (ctx.d >> 16) as u8;
    result[15] = (ctx.d >> 24) as u8;

    *ctx = hts_md5_context {
        lo: 0,
        hi: 0,
        a: 0,
        b: 0,
        c: 0,
        d: 0,
        buffer: [0; 64],
        block: [0; 16],
    };
}

pub fn md5_c_323_hts_md5_init() -> Box<hts_md5_context> {
    let mut ctx = Box::new(hts_md5_context {
        lo: 0,
        hi: 0,
        a: 0,
        b: 0,
        c: 0,
        d: 0,
        buffer: [0; 64],
        block: [0; 16],
    });
    md5_c_226_hts_md5_reset(&mut ctx);
    ctx
}

pub fn md5_c_344_hts_md5_init() -> Box<hts_md5_context> {
    md5_c_323_hts_md5_init()
}

pub fn md5_c_355_hts_md5_reset(ctx: &mut hts_md5_context) {
    md5_c_226_hts_md5_reset(ctx)
}

pub fn md5_c_360_hts_md5_update(ctx: &mut hts_md5_context, data: &[u8], size: usize) {
    md5_c_237_hts_md5_update(ctx, data, size)
}

pub fn md5_c_365_hts_md5_final(digest: &mut [u8], ctx: &mut hts_md5_context) {
    md5_c_271_hts_md5_final(digest, ctx)
}

pub fn md5_c_372_hts_md5_destroy(ctx: Option<Box<hts_md5_context>>) {
    drop(ctx);
}

pub fn md5_c_380_hts_md5_hex(hex: &mut [u8], digest: &[u8]) {
    static ALPHABET: &[u8; 16] = b"0123456789abcdef";
    for i in 0..16 {
        hex[i * 2] = ALPHABET[((digest[i] >> 4) & 0xf) as usize];
        hex[i * 2 + 1] = ALPHABET[(digest[i] & 0xf) as usize];
    }
    hex[32] = 0;
}

pub fn hts_md5_init() -> Box<hts_md5_context> {
    md5_c_323_hts_md5_init()
}

pub fn hts_md5_update(ctx: &mut hts_md5_context, data: &[u8], size: usize) {
    md5_c_237_hts_md5_update(ctx, data, size)
}

pub fn hts_md5_final(digest: &mut [u8], ctx: &mut hts_md5_context) {
    md5_c_271_hts_md5_final(digest, ctx)
}

pub fn hts_md5_reset(ctx: &mut hts_md5_context) {
    md5_c_226_hts_md5_reset(ctx)
}

pub fn hts_md5_hex(hex: &mut [u8], digest: &[u8]) {
    md5_c_380_hts_md5_hex(hex, digest)
}

pub fn hts_md5_destroy(ctx: Option<Box<hts_md5_context>>) {
    md5_c_372_hts_md5_destroy(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_digest(payloads: &[&[u8]]) -> Vec<u8> {
        let mut ctx = hts_md5_init();
        for payload in payloads {
            hts_md5_update(&mut ctx, payload, payload.len());
        }
        let mut digest = [0u8; 16];
        hts_md5_final(&mut digest, &mut ctx);
        let mut hex = [0u8; 33];
        hts_md5_hex(&mut hex, &digest);
        hts_md5_destroy(Some(ctx));
        hex[..32].to_vec()
    }

    #[test]
    fn md5_wrappers_hash_known_payload() {
        assert_eq!(hex_digest(&[b"abc"]), b"900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn md5_update_matches_known_chunk_boundaries() {
        assert_eq!(
            hex_digest(&[b"The quick ", b"brown fox jumps over ", b"the lazy dog",]),
            b"9e107d9d372bb6826bd81d3542a419d6"
        );
        assert_eq!(
            hex_digest(&[&[b'a'; 63], b"a", b"bc"]),
            b"9b2dd1476ebcd57011ba6d13ca5b37c4"
        );
    }

    #[test]
    fn md5_final_clears_context_like_htslib_memset() {
        let mut ctx = hts_md5_init();
        hts_md5_update(&mut ctx, b"abc", 3);

        let mut digest = [0u8; 16];
        hts_md5_final(&mut digest, &mut ctx);

        assert_eq!(ctx.lo, 0);
        assert_eq!(ctx.hi, 0);
        assert_eq!(ctx.a, 0);
        assert_eq!(ctx.b, 0);
        assert_eq!(ctx.c, 0);
        assert_eq!(ctx.d, 0);
        assert!(ctx.buffer.iter().all(|&byte| byte == 0));
        assert!(ctx.block.iter().all(|&word| word == 0));
        hts_md5_destroy(Some(ctx));
    }

    #[test]
    fn md5_update_accepts_zero_length_update() {
        assert_eq!(hex_digest(&[b"", b""]), b"d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(
            hex_digest(&[b"", b"abc", b""]),
            b"900150983cd24fb0d6963f7d28e17f72"
        );
    }

    #[test]
    fn md5_update_buffers_short_input_without_transforming_block() {
        let mut ctx = hts_md5_init();
        hts_md5_update(&mut ctx, b"abc", 3);

        assert_eq!(ctx.lo, 3);
        assert_eq!(ctx.hi, 0);
        assert_eq!(&ctx.buffer[..3], b"abc");
        assert_eq!(ctx.a, 0x67452301);
        assert_eq!(ctx.b, 0xefcdab89);
        assert_eq!(ctx.c, 0x98badcfe);
        assert_eq!(ctx.d, 0x10325476);

        hts_md5_destroy(Some(ctx));
    }

    #[test]
    fn md5_c_wrappers_accept_zero_length_and_null_destroy() {
        let mut ctx = md5_c_344_hts_md5_init();
        md5_c_360_hts_md5_update(&mut ctx, b"abc", 3);
        md5_c_360_hts_md5_update(&mut ctx, b"", 0);

        let mut digest = [0u8; 16];
        md5_c_365_hts_md5_final(&mut digest, &mut ctx);
        let mut hex = [0u8; 33];
        md5_c_380_hts_md5_hex(&mut hex, &digest);
        assert_eq!(&hex[..32], b"900150983cd24fb0d6963f7d28e17f72");

        md5_c_372_hts_md5_destroy(Some(ctx));
        md5_c_372_hts_md5_destroy(None);
    }

    #[test]
    fn md5_reset_discards_prior_partial_update() {
        let mut ctx = hts_md5_init();

        hts_md5_update(&mut ctx, b"prefix that must be discarded", 29);
        hts_md5_reset(&mut ctx);
        hts_md5_update(&mut ctx, b"abc", 3);

        let mut digest = [0u8; 16];
        hts_md5_final(&mut digest, &mut ctx);
        let mut hex = [0u8; 33];
        hts_md5_hex(&mut hex, &digest);
        assert_eq!(&hex[..32], b"900150983cd24fb0d6963f7d28e17f72");
        hts_md5_destroy(Some(ctx));
    }

    #[test]
    fn md5_hex_writes_lowercase_digits_and_nul_terminator() {
        let digest = [
            0x00, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76,
            0x54, 0x32,
        ];
        let mut hex = [b'X'; 34];
        hts_md5_hex(&mut hex, &digest);

        assert_eq!(&hex[..32], b"000123456789abcdeffedcba98765432");
        assert_eq!(hex[32], 0);
        assert_eq!(hex[33], b'X');
    }

    #[test]
    fn md5_split_padding_boundary_matches_one_shot() {
        let payload = [b'a'; 120];
        assert_eq!(
            hex_digest(&[
                &payload[..55],
                &payload[55..56],
                &payload[56..64],
                &payload[64..]
            ]),
            hex_digest(&[&payload])
        );
    }

    #[test]
    fn md5_padding_edge_lengths_are_chunking_invariant() {
        for len in [55, 56, 57, 64, 65] {
            let payload = vec![b'x'; len];
            assert_eq!(
                hex_digest(&[&payload[..len / 2], &payload[len / 2..]]),
                hex_digest(&[&payload]),
                "len={len}"
            );
        }
    }

    #[test]
    fn md5_update_carries_low_count_at_29_bit_boundary() {
        let mut ctx = hts_md5_context {
            lo: 0x1fff_ffff,
            hi: 7,
            a: 0x67452301,
            b: 0xefcdab89,
            c: 0x98badcfe,
            d: 0x10325476,
            buffer: [0; 64],
            block: [0; 16],
        };

        md5_c_237_hts_md5_update(&mut ctx, b"x", 1);
        assert_eq!(ctx.lo, 0);
        assert_eq!(ctx.hi, 8);
    }
}
