use std::ffi::{c_char, c_uchar, c_ulong, c_void};
use std::mem::size_of;
use std::ptr;

#[repr(C)]
pub struct hts_md5_context {
    lo: u32,
    hi: u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    buffer: [c_uchar; 64],
    block: [u32; 16],
}

pub unsafe fn md5_c_119_body(
    ctx: *mut hts_md5_context,
    data: *const c_void,
    mut size: c_ulong,
) -> *const c_void {
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
        ($ptr:ident, $ctx:ident, $n:expr) => {{
            let value = (*$ptr.add($n * 4) as u32)
                | ((*$ptr.add($n * 4 + 1) as u32) << 8)
                | ((*$ptr.add($n * 4 + 2) as u32) << 16)
                | ((*$ptr.add($n * 4 + 3) as u32) << 24);
            (*$ctx).block[$n] = value;
            value
        }};
    }
    macro_rules! get {
        ($ctx:ident, $n:expr) => {
            (*$ctx).block[$n]
        };
    }

    let mut ptr = data.cast::<c_uchar>();
    let mut a = (*ctx).a;
    let mut b = (*ctx).b;
    let mut c = (*ctx).c;
    let mut d = (*ctx).d;

    loop {
        let saved_a = a;
        let saved_b = b;
        let saved_c = c;
        let saved_d = d;

        step!(f, a, b, c, d, set!(ptr, ctx, 0), 0xd76aa478, 7);
        step!(f, d, a, b, c, set!(ptr, ctx, 1), 0xe8c7b756, 12);
        step!(f, c, d, a, b, set!(ptr, ctx, 2), 0x242070db, 17);
        step!(f, b, c, d, a, set!(ptr, ctx, 3), 0xc1bdceee, 22);
        step!(f, a, b, c, d, set!(ptr, ctx, 4), 0xf57c0faf, 7);
        step!(f, d, a, b, c, set!(ptr, ctx, 5), 0x4787c62a, 12);
        step!(f, c, d, a, b, set!(ptr, ctx, 6), 0xa8304613, 17);
        step!(f, b, c, d, a, set!(ptr, ctx, 7), 0xfd469501, 22);
        step!(f, a, b, c, d, set!(ptr, ctx, 8), 0x698098d8, 7);
        step!(f, d, a, b, c, set!(ptr, ctx, 9), 0x8b44f7af, 12);
        step!(f, c, d, a, b, set!(ptr, ctx, 10), 0xffff5bb1, 17);
        step!(f, b, c, d, a, set!(ptr, ctx, 11), 0x895cd7be, 22);
        step!(f, a, b, c, d, set!(ptr, ctx, 12), 0x6b901122, 7);
        step!(f, d, a, b, c, set!(ptr, ctx, 13), 0xfd987193, 12);
        step!(f, c, d, a, b, set!(ptr, ctx, 14), 0xa679438e, 17);
        step!(f, b, c, d, a, set!(ptr, ctx, 15), 0x49b40821, 22);

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

        ptr = ptr.add(64);
        size = size.wrapping_sub(64);
        if size == 0 {
            break;
        }
    }

    (*ctx).a = a;
    (*ctx).b = b;
    (*ctx).c = c;
    (*ctx).d = d;
    ptr.cast()
}

pub unsafe fn md5_c_226_hts_md5_reset(ctx: *mut hts_md5_context) {
    (*ctx).a = 0x67452301;
    (*ctx).b = 0xefcdab89;
    (*ctx).c = 0x98badcfe;
    (*ctx).d = 0x10325476;
    (*ctx).lo = 0;
    (*ctx).hi = 0;
}

pub unsafe fn md5_c_237_hts_md5_update(
    ctx: *mut hts_md5_context,
    mut data: *const c_void,
    mut size: c_ulong,
) {
    let saved_lo = (*ctx).lo;
    (*ctx).lo = saved_lo.wrapping_add(size as u32) & 0x1fff_ffff;
    if (*ctx).lo < saved_lo {
        (*ctx).hi = (*ctx).hi.wrapping_add(1);
    }
    (*ctx).hi = (*ctx).hi.wrapping_add((size >> 29) as u32);

    let mut used = (saved_lo & 0x3f) as c_ulong;
    if used != 0 {
        let available = 64 - used;
        if size < available {
            ptr::copy_nonoverlapping(
                data.cast::<c_uchar>(),
                (*ctx).buffer.as_mut_ptr().add(used as usize),
                size as usize,
            );
            return;
        }
        ptr::copy_nonoverlapping(
            data.cast::<c_uchar>(),
            (*ctx).buffer.as_mut_ptr().add(used as usize),
            available as usize,
        );
        data = data.cast::<c_uchar>().add(available as usize).cast();
        size -= available;
        md5_c_119_body(ctx, (*ctx).buffer.as_ptr().cast(), 64);
        used = 0;
        let _ = used;
    }

    if size >= 64 {
        data = md5_c_119_body(ctx, data, size & !0x3f).cast();
        size &= 0x3f;
    }

    ptr::copy_nonoverlapping(
        data.cast::<c_uchar>(),
        (*ctx).buffer.as_mut_ptr(),
        size as usize,
    );
}

pub unsafe fn md5_c_271_hts_md5_final(result: *mut c_uchar, ctx: *mut hts_md5_context) {
    let mut used = ((*ctx).lo & 0x3f) as usize;
    (*ctx).buffer[used] = 0x80;
    used += 1;

    let mut available = 64 - used;
    if available < 8 {
        ptr::write_bytes((*ctx).buffer.as_mut_ptr().add(used), 0, available);
        md5_c_119_body(ctx, (*ctx).buffer.as_ptr().cast(), 64);
        used = 0;
        available = 64;
    }

    ptr::write_bytes((*ctx).buffer.as_mut_ptr().add(used), 0, available - 8);

    (*ctx).lo <<= 3;
    (*ctx).buffer[56] = (*ctx).lo as c_uchar;
    (*ctx).buffer[57] = ((*ctx).lo >> 8) as c_uchar;
    (*ctx).buffer[58] = ((*ctx).lo >> 16) as c_uchar;
    (*ctx).buffer[59] = ((*ctx).lo >> 24) as c_uchar;
    (*ctx).buffer[60] = (*ctx).hi as c_uchar;
    (*ctx).buffer[61] = ((*ctx).hi >> 8) as c_uchar;
    (*ctx).buffer[62] = ((*ctx).hi >> 16) as c_uchar;
    (*ctx).buffer[63] = ((*ctx).hi >> 24) as c_uchar;

    md5_c_119_body(ctx, (*ctx).buffer.as_ptr().cast(), 64);

    *result.add(0) = (*ctx).a as c_uchar;
    *result.add(1) = ((*ctx).a >> 8) as c_uchar;
    *result.add(2) = ((*ctx).a >> 16) as c_uchar;
    *result.add(3) = ((*ctx).a >> 24) as c_uchar;
    *result.add(4) = (*ctx).b as c_uchar;
    *result.add(5) = ((*ctx).b >> 8) as c_uchar;
    *result.add(6) = ((*ctx).b >> 16) as c_uchar;
    *result.add(7) = ((*ctx).b >> 24) as c_uchar;
    *result.add(8) = (*ctx).c as c_uchar;
    *result.add(9) = ((*ctx).c >> 8) as c_uchar;
    *result.add(10) = ((*ctx).c >> 16) as c_uchar;
    *result.add(11) = ((*ctx).c >> 24) as c_uchar;
    *result.add(12) = (*ctx).d as c_uchar;
    *result.add(13) = ((*ctx).d >> 8) as c_uchar;
    *result.add(14) = ((*ctx).d >> 16) as c_uchar;
    *result.add(15) = ((*ctx).d >> 24) as c_uchar;

    ptr::write_bytes(ctx.cast::<u8>(), 0, size_of::<hts_md5_context>());
}

pub unsafe fn md5_c_323_hts_md5_init() -> *mut hts_md5_context {
    let ctx = libc::malloc(size_of::<hts_md5_context>()).cast::<hts_md5_context>();
    if ctx.is_null() {
        return ptr::null_mut();
    }
    md5_c_226_hts_md5_reset(ctx);
    ctx
}

pub unsafe fn md5_c_344_hts_md5_init() -> *mut hts_md5_context {
    md5_c_323_hts_md5_init()
}

pub unsafe fn md5_c_355_hts_md5_reset(ctx: *mut hts_md5_context) {
    md5_c_226_hts_md5_reset(ctx)
}

pub unsafe fn md5_c_360_hts_md5_update(
    ctx: *mut hts_md5_context,
    data: *const c_void,
    size: c_ulong,
) {
    md5_c_237_hts_md5_update(ctx, data, size)
}

pub unsafe fn md5_c_365_hts_md5_final(digest: *mut c_uchar, ctx: *mut hts_md5_context) {
    md5_c_271_hts_md5_final(digest, ctx)
}

pub unsafe fn md5_c_372_hts_md5_destroy(ctx: *mut hts_md5_context) {
    if ctx.is_null() {
        return;
    }
    libc::free(ctx.cast());
}

pub unsafe fn md5_c_380_hts_md5_hex(hex: *mut c_char, digest: *const c_uchar) {
    static ALPHABET: &[u8; 16] = b"0123456789abcdef";
    for i in 0..16 {
        *hex.add(i * 2) = ALPHABET[((*digest.add(i) >> 4) & 0xf) as usize] as c_char;
        *hex.add(i * 2 + 1) = ALPHABET[(*digest.add(i) & 0xf) as usize] as c_char;
    }
    *hex.add(32) = 0;
}

pub unsafe fn hts_md5_init() -> *mut hts_md5_context {
    md5_c_323_hts_md5_init()
}

pub unsafe fn hts_md5_update(ctx: *mut hts_md5_context, data: *const c_void, size: c_ulong) {
    md5_c_237_hts_md5_update(ctx, data, size)
}

pub unsafe fn hts_md5_final(digest: *mut c_uchar, ctx: *mut hts_md5_context) {
    md5_c_271_hts_md5_final(digest, ctx)
}

pub unsafe fn hts_md5_reset(ctx: *mut hts_md5_context) {
    md5_c_226_hts_md5_reset(ctx)
}

pub unsafe fn hts_md5_hex(hex: *mut c_char, digest: *const c_uchar) {
    md5_c_380_hts_md5_hex(hex, digest)
}

pub unsafe fn hts_md5_destroy(ctx: *mut hts_md5_context) {
    md5_c_372_hts_md5_destroy(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe fn hex_digest(payloads: &[&[u8]]) -> Vec<u8> {
        let ctx = hts_md5_init();
        assert!(!ctx.is_null());
        for payload in payloads {
            hts_md5_update(ctx, payload.as_ptr().cast(), payload.len() as c_ulong);
        }
        let mut digest = [0u8; 16];
        hts_md5_final(digest.as_mut_ptr(), ctx);
        let mut hex = [0 as c_char; 33];
        hts_md5_hex(hex.as_mut_ptr(), digest.as_ptr());
        hts_md5_destroy(ctx);
        hex[..32].iter().map(|&ch| ch as u8).collect()
    }

    #[test]
    fn md5_wrappers_hash_known_payload() {
        unsafe {
            assert_eq!(hex_digest(&[b"abc"]), b"900150983cd24fb0d6963f7d28e17f72");
        }
    }

    #[test]
    fn md5_update_matches_known_chunk_boundaries() {
        unsafe {
            assert_eq!(
                hex_digest(&[b"The quick ", b"brown fox jumps over ", b"the lazy dog",]),
                b"9e107d9d372bb6826bd81d3542a419d6"
            );
            assert_eq!(
                hex_digest(&[&[b'a'; 63], b"a", b"bc"]),
                b"9b2dd1476ebcd57011ba6d13ca5b37c4"
            );
        }
    }
}
