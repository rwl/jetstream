// The number of samples per message required before using simple-8b encoding.
pub(crate) const SIMPLE8B_THRESHOLD_SAMPLES: usize = 16;

// The default number of layers of delta encoding. 0 is no delta encoding (just use varint),
// 1 is delta encoding, etc.
pub(crate) const DEFAULT_DELTA_ENCODING_LAYERS: usize = 3;

// The number of layers of delta encoding for high sampling rate scenarios.
pub(crate) const HIGH_DELTA_ENCODING_LAYERS: usize = 3;

// The size of the message header in bytes.
pub(crate) const MAX_HEADER_SIZE: usize = 36;

// The minimum number of samples per message to use gzip on the payload.
pub(crate) const USE_GZIP_THRESHOLD_SAMPLES: usize = 4096;

/// Lists of variables to be encoded.
#[derive(Clone)]
pub struct Dataset {
    pub i32s: Vec<i32>,
    // can extend with other data types
}

impl Dataset {
    pub(crate) fn new(count: usize) -> Self {
        Self {
            i32s: vec![0; count],
        }
    }
}

/// Lists of decoded variables with a timestamp and quality
#[derive(Clone)]
pub struct DatasetWithQuality {
    pub t: u64,
    pub i32s: Vec<i32>,
    pub q: Vec<u32>,
}

impl DatasetWithQuality {
    pub fn new(count: usize) -> Self {
        Self {
            t: 0,
            i32s: vec![0; count],
            q: vec![0; count],
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct QualityHistory {
    pub(crate) value: u32,
    pub(crate) samples: u32,
}

pub(crate) fn create_spatial_refs(
    count: usize,
    count_v: usize,
    count_i: usize,
    include_neutral: bool,
) -> Vec<Option<usize>> {
    let mut refs: Vec<Option<usize>> = vec![None; count as usize];

    let inc = if include_neutral { 4 } else { 3 };

    for i in 0..count {
        if i >= inc {
            if i < count_v * inc {
                refs[i] = Some(i - inc);
            } else if i >= (count_v + 1) * inc && i < (count_v + count_i) * inc {
                refs[i] = Some(i - inc);
            }
        }
    }
    // println!("{:?}", refs);
    refs
}

pub(crate) fn get_delta_encoding(sampling_rate: usize) -> usize {
    if sampling_rate > 100_000 {
        HIGH_DELTA_ENCODING_LAYERS
    } else {
        DEFAULT_DELTA_ENCODING_LAYERS
    }
}

// TODO: Use "integer-encoding" crate

/// Copied from encoding/binary/varint.go to provide 32-bit version to avoid casting.
pub(crate) fn uvarint32(buf: &[u8]) -> (u32, usize) {
    let mut x: u32 = 0;
    let mut s: usize = 0;
    for i in 0..buf.len() {
        let b = buf[i];
        if b < 0x80 {
            if i > 9 || i == 9 && b > 1 {
                panic!("uvarint32: overflow")
                // return (0, -(i + 1)); // overflow  FIXME: Result
            }
            return (x | (b as u32) << s, i + 1);
        }
        x |= ((b & 0x7f) as u32) << s;
        s += 7
    }
    (0, 0)
}

pub(crate) fn varint32(buf: &[u8]) -> (i32, usize) {
    let (ux, n) = uvarint32(buf); // ok to continue in presence of error
    let mut x = (ux >> 1) as i32;
    if ux & 1 != 0 {
        x = !x;
    }
    (x, n)
}

/// Encodes a `u32` into `buf` and returns the number of bytes written.
/// If the buffer is too small, `put_uvarint32` will panic.
pub(crate) fn put_uvarint32(buf: &mut [u8], mut x: u32) -> usize {
    let mut i = 0;
    while x >= 0x80 {
        buf[i] = (x as u8) | 0x80;
        x >>= 7;
        i += 1;
    }
    buf[i] = x as u8;
    i + 1
}

/// Encodes an `i32` into `buf` and returns the number of bytes written.
/// If the buffer is too small, `put_varint32` will panic.
pub(crate) fn put_varint32(buf: &mut [u8], x: i32) -> usize {
    let mut ux = (x as u32) << 1;
    if x < 0 {
        ux = !ux
    }
    put_uvarint32(buf, ux)
}
