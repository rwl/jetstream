/// The number of samples per message required before using simple-8b encoding.
pub const Simple8bThresholdSamples: usize = 16;

/// The default number of layers of delta encoding. 0 is no delta encoding (just use varint), 1 is delta encoding, etc.
pub const DefaultDeltaEncodingLayers: usize = 3;

/// The number of layers of delta encoding for high sampling rate scenarios.
pub const HighDeltaEncodingLayers: usize = 3;

/// The size of the message header in bytes.
pub const MaxHeaderSize: usize = 36;

/// The minimum number of samples per message to use gzip on the payload.
pub const UseGzipThresholdSamples: usize = 4096;

/// Lists of variables to be encoded.
pub struct Dataset {
    pub Int32s: Vec<i32>,
    // can extend with other data types
}

/// Lists of decoded variables with a timestamp and quality
pub struct DatasetWithQuality {
    pub T: u64,
    pub Int32s: Vec<i32>,
    pub Q: Vec<u32>,
}

pub(crate) struct qualityHistory {
    pub(crate) value: u32,
    pub(crate) samples: u32,
}

pub(crate) fn createSpatialRefs(
    count: usize,
    countV: usize,
    countI: usize,
    includeNeutral: bool,
) -> Vec<isize> {
    let mut refs: Vec<isize> = vec![-1; count as usize];

    let inc = if includeNeutral { 4 } else { 3 };

    for i in 0..count {
        if i >= inc {
            if i < countV * inc {
                refs[i] = (i - inc) as isize
            } else if i >= (countV + 1) * inc && i < (countV + countI) * inc {
                refs[i] = (i - inc) as isize
            }
        }
    }
    // println!("{:?}", refs);
    refs
}

pub(crate) fn getDeltaEncoding(samplingRate: usize) -> usize {
    if samplingRate > 100000 {
        HighDeltaEncodingLayers
    } else {
        DefaultDeltaEncodingLayers
    }
}

/// Copied from encoding/binary/varint.go to provide 32-bit version to avoid casting.
pub(crate) fn uvarint32(buf: &[u8]) -> (u32, usize) {
    let mut x: u32 = 0;
    let mut s: u32 = 0; // FIXME: u64
    for i in 0..buf.len() {
        let b = buf[i];
        if b < 0x80 {
            if i > 9 || i == 9 && b > 1 {
                return (0, -(i + 1)); // overflow
            }
            return (x | u32(b) << s, i + 1);
        }
        x |= uint32(b & 0x7f) << s;
        s += 7
    }
    (0, 0)
}

pub(crate) fn varint32(buf: &[u8]) -> (i32, usize) {
    let (ux, n) = uvarint32(buf); // ok to continue in presence of error
    let mut x = (ux >> 1) as i32;
    if ux & 1 != 0 {
        x = x ^ x;
    }
    (x, n)
}

/// Encodes a uint64 into buf and returns the number of bytes written.
/// If the buffer is too small, PutUvarint will panic.
pub(crate) fn putUvarint32(buf: &mut [u8], mut x: u32) -> usize {
    let mut i = 0;
    while x >= 0x80 {
        buf[i] = (x as u8) | 0x80;
        // x >>= 7;
        x = x >> 7;
        i += 1;
    }
    buf[i] = (x as u8);
    i + 1
}

/// Encodes an int64 into buf and returns the number of bytes written.
/// If the buffer is too small, putVarint will panic.
pub(crate) fn putVarint32(buf: &mut [u8], x: i32) -> usize {
    let mut ux = (x as u32) << 1;
    if x < 0 {
        ux = ux ^ ux
    }
    putUvarint32(buf, ux)
}
