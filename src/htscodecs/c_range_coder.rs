//! Translation of htscodecs `c_range_coder.h`.
//!
//! Header-only range coder (Eugene Shelwien, modifications by James Bonfield).
//! Each `static inline` function in the header becomes one stub here.
//!
//! The C struct uses raw `uc *` pointers walking over caller-owned input and
//! output buffers.  We keep that representation verbatim (`*mut u8`) so the
//! carry/renormalisation byte arithmetic is bit-for-bit identical to C.

// #define TOP (1<<24)                       // c_range_coder.h:21
pub const TOP: u32 = 1 << 24;

// #define Thres (unsigned)255*TOP           // c_range_coder.h:22
pub const THRES: u32 = 255u32.wrapping_mul(TOP);

// typedef unsigned char uc;                 // c_range_coder.h:24

/// ```c
/// typedef struct {
///     uint32_t low, code, range;
///     uint32_t FFNum;  // Number of consecutive FFs
///     uint32_t Cache;  // Top 8-bits of low ready to emit
///     uint32_t Carry;  // Flag to indicate if we emit Cache or Cache+1
///     uc *in_buf;
///     uc *out_buf;
///     uc *in_end;
///     uc *out_end;
///     int err;
/// } RangeCoder;
/// ```
// c_range_coder.h:26
#[repr(C)]
pub struct RangeCoder {
    pub low: u32,
    pub code: u32,
    pub range: u32,
    pub ff_num: u32,
    pub cache: u32,
    pub carry: u32,
    pub in_buf: *mut u8,
    pub out_buf: *mut u8,
    pub in_end: *mut u8,
    pub out_end: *mut u8,
    pub err: i32,
}

impl RangeCoder {
    /// A zeroed RangeCoder, matching an uninitialised C struct before the
    /// SetInput/SetOutput + StartEncode/StartDecode calls populate it.
    pub fn new() -> Self {
        RangeCoder {
            low: 0,
            code: 0,
            range: 0,
            ff_num: 0,
            cache: 0,
            carry: 0,
            in_buf: std::ptr::null_mut(),
            out_buf: std::ptr::null_mut(),
            in_end: std::ptr::null_mut(),
            out_end: std::ptr::null_mut(),
            err: 0,
        }
    }
}

impl Default for RangeCoder {
    fn default() -> Self {
        Self::new()
    }
}

/// `static inline void RC_SetInput(RangeCoder *rc, char *in, char *in_end)`
// c_range_coder.h:38
pub fn RC_SetInput(rc: &mut RangeCoder, r#in: *mut u8, in_end: *mut u8) {
    rc.in_buf = r#in;
    rc.out_buf = r#in;
    rc.in_end = in_end;
}

/// `static inline void RC_SetOutput(RangeCoder *rc, char *out)`
// c_range_coder.h:44
pub fn RC_SetOutput(rc: &mut RangeCoder, out: *mut u8) {
    rc.in_buf = out;
    rc.out_buf = out;
    rc.out_end = std::ptr::null_mut();
}

/// `static inline void RC_SetOutputEnd(RangeCoder *rc, char *out_end)`
// c_range_coder.h:45
pub fn RC_SetOutputEnd(rc: &mut RangeCoder, out_end: *mut u8) {
    rc.out_end = out_end;
}

/// `static inline char *RC_GetInput(RangeCoder *rc)`
// c_range_coder.h:46
pub fn RC_GetInput(rc: &RangeCoder) -> *mut u8 {
    rc.in_buf
}

/// `static inline char *RC_GetOutput(RangeCoder *rc)`
// c_range_coder.h:47
pub fn RC_GetOutput(rc: &RangeCoder) -> *mut u8 {
    rc.out_buf
}

/// `static inline size_t RC_OutSize(RangeCoder *rc)`
// c_range_coder.h:48
pub fn RC_OutSize(rc: &RangeCoder) -> usize {
    (rc.out_buf as usize) - (rc.in_buf as usize)
}

/// `static inline size_t RC_InSize(RangeCoder *rc)`
// c_range_coder.h:49
pub fn RC_InSize(rc: &RangeCoder) -> usize {
    (rc.in_buf as usize) - (rc.out_buf as usize)
}

/// `static inline void RC_StartEncode(RangeCoder *rc)`
// c_range_coder.h:51
pub fn RC_StartEncode(rc: &mut RangeCoder) {
    rc.range = 0xFFFFFFFF;
    rc.low = 0;
    rc.ff_num = 0;
    rc.carry = 0;
    rc.cache = 0;
    rc.code = 0;
    rc.err = 0;
}

/// `static inline void RC_StartDecode(RangeCoder *rc)`
// c_range_coder.h:62
pub fn RC_StartDecode(rc: &mut RangeCoder) {
    rc.range = 0xFFFFFFFF;
    rc.low = 0;
    rc.ff_num = 0;
    rc.carry = 0;
    rc.cache = 0;
    rc.code = 0;
    rc.err = 0;
    // if (rc->in_buf+5 > rc->in_end) { rc->in_buf = rc->in_end; return; }
    if (rc.in_buf as usize) + 5 > (rc.in_end as usize) {
        rc.in_buf = rc.in_end; // prevent decode
        return;
    }
    // DO(5) rc->code = (rc->code<<8) | *rc->in_buf++;
    for _ in 0..5 {
        unsafe {
            rc.code = (rc.code << 8) | (*rc.in_buf as u32);
            rc.in_buf = rc.in_buf.add(1);
        }
    }
}

/// `static inline void RC_ShiftLowCheck(RangeCoder *rc)`
// c_range_coder.h:78
pub fn RC_ShiftLowCheck(rc: &mut RangeCoder) {
    if rc.low < THRES || rc.carry != 0 {
        // if (rc->out_end && rc->FFNum >= rc->out_end - rc->out_buf) {err=-1; return;}
        if !rc.out_end.is_null()
            && rc.ff_num as usize >= (rc.out_end as usize) - (rc.out_buf as usize)
        {
            rc.err = -1;
            return;
        }

        unsafe {
            *rc.out_buf = (rc.cache.wrapping_add(rc.carry)) as u8;
            rc.out_buf = rc.out_buf.add(1);

            // Flush any stored FFs
            while rc.ff_num != 0 {
                *rc.out_buf = (rc.carry.wrapping_sub(1)) as u8; // (Carry-1)&255
                rc.out_buf = rc.out_buf.add(1);
                rc.ff_num -= 1;
            }
        }

        // Take copy of top byte ready for next flush
        rc.cache = rc.low >> 24;
        rc.carry = 0;
    } else {
        // Low is FFxx xxxx.  Bump FF count and shift in as before
        rc.ff_num += 1;
    }
    rc.low <<= 8;
}

/// `static inline void RC_ShiftLow(RangeCoder *rc)`
// c_range_coder.h:103
pub fn RC_ShiftLow(rc: &mut RangeCoder) {
    if rc.low < THRES || rc.carry != 0 {
        unsafe {
            *rc.out_buf = (rc.cache.wrapping_add(rc.carry)) as u8;
            rc.out_buf = rc.out_buf.add(1);

            // Flush any stored FFs
            while rc.ff_num != 0 {
                *rc.out_buf = (rc.carry.wrapping_sub(1)) as u8; // (Carry-1)&255
                rc.out_buf = rc.out_buf.add(1);
                rc.ff_num -= 1;
            }
        }

        // Take copy of top byte ready for next flush
        rc.cache = rc.low >> 24;
        rc.carry = 0;
    } else {
        // Low is FFxx xxxx.  Bump FF count and shift in as before
        rc.ff_num += 1;
    }
    rc.low <<= 8;
}

/// `static inline int RC_FinishEncode(RangeCoder *rc)`
// c_range_coder.h:123
pub fn RC_FinishEncode(rc: &mut RangeCoder) -> i32 {
    for _ in 0..5 {
        RC_ShiftLowCheck(rc);
    }
    rc.err
}

/// `static inline int RC_FinishDecode(RangeCoder *rc)`
// c_range_coder.h:129
pub fn RC_FinishDecode(rc: &mut RangeCoder) -> i32 {
    rc.err
}

/// `static inline void RC_Encode(RangeCoder *rc, uint32_t cumFreq, uint32_t freq, uint32_t totFreq)`
// c_range_coder.h:133
pub fn RC_Encode(rc: &mut RangeCoder, cum_freq: u32, freq: u32, tot_freq: u32) {
    let tmp = rc.low;
    rc.range /= tot_freq;
    rc.low = rc.low.wrapping_add(cum_freq.wrapping_mul(rc.range));
    rc.range = rc.range.wrapping_mul(freq);

    // rc->Carry += rc->low<tmp;  // Overflow
    rc.carry = rc.carry.wrapping_add((rc.low < tmp) as u32);

    while rc.range < TOP {
        rc.range <<= 8;
        RC_ShiftLowCheck(rc);
    }
}

/// `static inline uint32_t RC_GetFreq(RangeCoder *rc, uint32_t totFreq)`
// c_range_coder.h:147
pub fn RC_GetFreq(rc: &mut RangeCoder, tot_freq: u32) -> u32 {
    // return (totFreq && rc->range >= totFreq) ? rc->code/(rc->range/=totFreq) : 0;
    if tot_freq != 0 && rc.range >= tot_freq {
        rc.range /= tot_freq;
        rc.code / rc.range
    } else {
        0
    }
}

/// `static inline void RC_Decode(RangeCoder *rc, uint32_t cumFreq, uint32_t freq, uint32_t totFreq)`
// c_range_coder.h:152
pub fn RC_Decode(rc: &mut RangeCoder, cum_freq: u32, freq: u32, _tot_freq: u32) {
    rc.code = rc.code.wrapping_sub(cum_freq.wrapping_mul(rc.range));
    rc.range = rc.range.wrapping_mul(freq);
    while rc.range < TOP {
        if (rc.in_buf as usize) >= (rc.in_end as usize) {
            rc.err = -1;
            return;
        }
        unsafe {
            rc.code = (rc.code << 8).wrapping_add(*rc.in_buf as u32);
            rc.in_buf = rc.in_buf.add(1);
        }
        rc.range <<= 8;
    }
}
