/// Package simple8b implements the 64bit integer encoding algoritm as published
/// by Ann and Moffat in "Index compression using 64-bit words", Softw. Pract. Exper. 2010; 40:131–147
///
/// It is capable of encoding multiple integers with values betweeen 0 and to 1^60 -1, in a single word.

/// Simple8b is 64bit word-sized encoder that packs multiple integers into a single word using
/// a 4 bit selector values and up to 60 bits for the remaining values.  Integers are encoded using
/// the following table:
///
/// ┌──────────────┬─────────────────────────────────────────────────────────────┐
/// │   Selector   │       0    1   2   3   4   5   6   7  8  9  0 11 12 13 14 15│
/// ├──────────────┼─────────────────────────────────────────────────────────────┤
/// │     Bits     │       0    0   1   2   3   4   5   6  7  8 10 12 15 20 30 60│
/// ├──────────────┼─────────────────────────────────────────────────────────────┤
/// │      N       │     240  120  60  30  20  15  12  10  8  7  6  5  4  3  2  1│
/// ├──────────────┼─────────────────────────────────────────────────────────────┤
/// │   Wasted Bits│      60   60   0   0   0   0  12   0  4  4  0  0  0  0  0  0│
/// └──────────────┴─────────────────────────────────────────────────────────────┘
///
/// For example, when the number of values can be encoded using 4 bits, selected 5 is encoded in the
/// 4 most significant bits followed by 15 values encoded used 4 bits each in the remaing 60 bits.

/// Converts a stream of unsigned 64bit integers to a compressed byte slice.
pub struct Encoder {
    // most recently written integers that have not been flushed
    buf: Vec<u64>,

    // index in buf of the head of the buf
    h: usize,

    // index in buf of the tail of the buf
    t: usize,

    // index into bytes of written bytes
    bp: usize,

    // current bytes written and flushed
    bytes: Vec<byte>,
    b: Vec<byte>,
}

impl Encoder {
    /// Returns an Encoder able to convert uint64s to compressed byte slices.
    pub fn new() -> Self {
        Self {
            buf: vec![0; 240],
            h: 0,
            t: 0,
            bp: 0,
            bytes: vec![0; 128],
            b: vec![0; 8],
        }
    }

    pub fn SetValues(&mut self, v: Vec<u64>) {
        self.buf = v;
        self.t = v.len();
        self.h = 0;
        // e.bytes = e.bytes[:0]
        self.bytes.clear()
    }

    pub fn Reset(&mut self) {
        self.t = 0;
        self.h = 0;
        self.bp = 0;

        self.buf = self.buf[..240].to_vec();
        self.b = self.b[..8].to_vec();
        self.bytes = self.bytes[..128].to_vec();
    }

    pub fn Write(&mut self, v: u64) -> Result<(), String> {
        if self.t >= self.buf.len() {
            self.flush()?;
        }

        // The buf is full but there is space at the front, just shift
        // the values down for now. TODO: use ring buffer
        if self.t >= self.buf.len() {
            self.buf.copy_from_slice(&self.buf[self.h..]);
            self.t -= self.h;
            self.h = 0;
        }
        self.buf[self.t] = v;
        self.t += 1;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        if self.t == 0 {
            return Ok(());
        }

        // encode as many values into one as we can
        let (encoded, n) = Encode(self.buf[self.h..self.t])?;

        binary.BigEndian.PutUint64(self.b, encoded);
        if self.bp + 8 > self.bytes.len() {
            self.bytes.extend(self.b[..]);
            self.bp = self.bytes.len();
        } else {
            self.bytes[self.bp..self.bp + 8].copy_from_slice(&self.b);
            self.bp += 8;
        }

        // Move the head forward since we encoded those values
        self.h += n;

        // If we encoded them all, reset the head/tail pointers to the beginning
        if self.h == self.t {
            self.h = 0;
            self.t = 0;
        }

        Ok(())
    }

    pub fn Bytes(&mut self) -> Result<Vec<u8>, String> {
        while self.t > 0 {
            self.flush()?;
        }

        Ok(self.bytes[..self.bp].to_vec())
    }
}

// Decoder converts a compressed byte slice to a stream of unsigned 64bit integers.
pub struct Decoder {
    bytes: Vec<u8>,
    buf: [u64; 240],
    i: usize,
    n: usize,
}

impl Decoder {
    /// Returns a Decoder from a byte vector.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            buf: [u64; 240],
            i: 0,
            n: 0,
        }
    }

    // Next returns true if there are remaining values to be read.  Successive
    // calls to Next advance the current element pointer.
    pub fn Next(&mut self) -> bool {
        self.i += 1;

        if self.i >= self.n {
            self.read()
        }

        self.bytes.len() >= 8 || (self.i >= 0 && self.i < self.n)
    }

    pub fn SetBytes(&mut self, b: Vec<u8>) {
        d.bytes = b;
        d.i = 0;
        d.n = 0;
    }

    /// Read returns the current value.  Successive calls to Read return the same value.
    pub fn Read(&self) -> u64 {
        let v = self.buf[self.i];
        v
    }

    fn read(&mut self) {
        if self.bytes.len() < 8 {
            return;
        }

        let v = binary.BigEndian.Uint64(self.bytes[..8]);
        self.bytes = self.bytes[8..];
        let (n, _) = Decode(&self.buf, v);
        self.n = n;
        self.i = 0;
    }
}

struct packing {
    n: usize,
    bit: usize,
    unpack: fn(u64, &mut [u64; 240]),
    pack: fn(&[u64]) -> u64,
}

const selector: [packing; 16] = [
    packing {
        n: 240,
        bit: 0,
        unpack: unpack240,
        pack: pack240,
    },
    packing {
        n: 120,
        bit: 0,
        unpack: unpack120,
        pack: pack120,
    },
    packing {
        n: 60,
        bit: 1,
        unpack: unpack60,
        pack: pack60,
    },
    packing {
        n: 30,
        bit: 2,
        unpack: unpack30,
        pack: pack30,
    },
    packing {
        n: 20,
        bit: 3,
        unpack: unpack20,
        pack: pack20,
    },
    packing {
        n: 15,
        bit: 4,
        unpack: unpack15,
        pack: pack15,
    },
    packing {
        n: 12,
        bit: 5,
        unpack: unpack12,
        pack: pack12,
    },
    packing {
        n: 10,
        bit: 6,
        unpack: unpack10,
        pack: pack10,
    },
    packing {
        n: 8,
        bit: 7,
        unpack: unpack8,
        pack: pack8,
    },
    packing {
        n: 7,
        bit: 8,
        unpack: unpack7,
        pack: pack7,
    },
    packing {
        n: 6,
        bit: 10,
        unpack: unpack6,
        pack: pack6,
    },
    packing {
        n: 5,
        bit: 12,
        unpack: unpack5,
        pack: pack5,
    },
    packing {
        n: 4,
        bit: 15,
        unpack: unpack4,
        pack: pack4,
    },
    packing {
        n: 3,
        bit: 20,
        unpack: unpack3,
        pack: pack3,
    },
    packing {
        n: 2,
        bit: 30,
        unpack: unpack2,
        pack: pack2,
    },
    packing {
        n: 1,
        bit: 60,
        unpack: unpack1,
        pack: pack1,
    },
];

/// Returns the number of integers encoded in the byte slice.
pub fn CountBytes(mut b: &[u8]) -> Result<usize, String> {
    let mut count = 0;
    while b.len() >= 8 {
        let v = binary.BigEndian.Uint64(b[..8]);
        b = &b[8..];

        let sel = v >> 60;
        if sel >= 16 {
            return Err(format!("invalid selector value: {}", sel));
        }
        count += selector[sel].n;
    }

    if b.len() > 0 {
        return Err(format!("invalid slice len remaining: {}", b.len()));
    }
    Ok(count)
}

/// Returns the number of integers encoded within an u64.
pub fn Count(v: u64) -> Result<usize, String> {
    let sel = v >> 60;
    if sel >= 16 {
        return Err(format!("invalid selector value: {}", sel));
    }
    Ok(selector[sel].n)
}

pub fn ForEach(mut b: &[u8], fn_: fn(v: u64) -> bool) -> Result<usize, String> {
    let mut count = 0;
    while b.len() >= 8 {
        let v = binary.BigEndian.Uint64(b[..8]);
        b = &b[8..];
        count += 1;

        let sel = v >> 60;
        if sel >= 16 {
            return Err(format!("invalid selector value: {}", sel));
        }

        let n = selector[sel].n;
        let bits = selector[sel].bit; // as usize;

        // let mask = uint64(^(int64(^0) << bits))
        unimplemented!("mask"); // FIXME

        for i in 0..n {
            let val = v & mask;
            if !fn_(val) {
                Ok(count)
            }
            v = v >> bits
        }
    }
    Ok(count)
}

pub fn CountBytesBetween(mut b: &[u8], min: u64, max: u64) -> Result<usize, String> {
    let mut count = 0;
    while b.len() >= 8 {
        let v = binary.BigEndian.Uint64(&b[..8]);
        b = &b[8..];

        let sel = v >> 60;
        if sel >= 16 {
            return Err(format!("invalid selector value: {}", sel));
        }
        // If the max value that could be encoded by the uint64 is less than the min
        // skip the whole thing.
        let maxValue = ((1 << (selector[sel].bit as u64)) - 1) as u64;
        if maxValue < min {
            continue;
        }

        // mask := uint64(^(int64(^0) << uint(selector[sel].bit)))
        unimplemented!("mask"); // FIXME

        for i in 0..selector[sel].n {
            let val = v & mask;
            if val >= min && val < max {
                count += 1;
            } else if val > max {
                break;
            }

            v = v >> selector[sel].bit; // as usize
        }
    }

    if b.len() > 0 {
        Err(format!("invalid slice len remaining: {}", b.len()))
    } else {
        Ok(count)
    }
}

/// Encode packs as many values into a single uint64.  It returns the packed
/// u64, how many values from src were packed, or an error if the values exceed
/// the maximum value range.
pub fn Encode(src: &[u64]) -> Result<(u64, usize), String> {
    if canPack(src, 240, 0) {
        Ok((0u64, 240))
    } else if canPack(src, 120, 0) {
        Ok((1 << 60, 120))
    } else if canPack(src, 60, 1) {
        Ok((pack60(&src[..60]), 60))
    } else if canPack(src, 30, 2) {
        Ok((pack30(&src[..30]), 30))
    } else if canPack(src, 20, 3) {
        Ok((pack20(&src[..20]), 20))
    } else if canPack(src, 15, 4) {
        Ok((pack15(&src[..15]), 15))
    } else if canPack(src, 12, 5) {
        Ok((pack12(&src[..12]), 12))
    } else if canPack(src, 10, 6) {
        Ok((pack10(&src[..10]), 10))
    } else if canPack(src, 8, 7) {
        Ok((pack8(&src[..8]), 8))
    } else if canPack(src, 7, 8) {
        Ok((pack7(&src[..7]), 7))
    } else if canPack(src, 6, 10) {
        Ok((pack6(&src[..6]), 6))
    } else if canPack(src, 5, 12) {
        Ok((pack5(&src[..5]), 5))
    } else if canPack(src, 4, 15) {
        Ok((pack4(&src[..4]), 4))
    } else if canPack(src, 3, 20) {
        Ok((pack3(&src[..3]), 3))
    } else if canPack(src, 2, 30) {
        Ok((pack2(&src[..2]), 2))
    } else if canPack(src, 1, 60) {
        Ok((pack1(&src[..1]), 1))
    } else {
        if src.len() > 0 {
            Err(format!("value out of bounds: {}", src.len()))
        } else {
            Ok((0, 0))
        }
    }
}

/// Returns a packed slice of the values from src. If a value is over
/// `1 << 60`, an error is returned. The input src is modified to avoid
/// extra allocations. If you need to re-use, use a copy.
pub fn EncodeAll(src: &[u64]) -> Result<Vec<u64>, String> {
    let mut i = 0;

    // Re-use the input slice and write encoded values back in place
    let mut dst = src.to_vec();
    let mut j = 0;

    loop {
        if i >= src.len() {
            break;
        }
        let remaining = &src[i..];

        if canPack(remaining, 240, 0) {
            dst[j] = 0;
            i += 240;
        } else if canPack(remaining, 120, 0) {
            dst[j] = 1 << 60;
            i += 120;
        } else if canPack(remaining, 60, 1) {
            dst[j] = pack60(&src[i..i + 60]);
            i += 60;
        } else if canPack(remaining, 30, 2) {
            dst[j] = pack30(&src[i..i + 30]);
            i += 30;
        } else if canPack(remaining, 20, 3) {
            dst[j] = pack20(&src[i..i + 20]);
            i += 20;
        } else if canPack(remaining, 15, 4) {
            dst[j] = pack15(&src[i..i + 15]);
            i += 15;
        } else if canPack(remaining, 12, 5) {
            dst[j] = pack12(&src[i..i + 12]);
            i += 12;
        } else if canPack(remaining, 10, 6) {
            dst[j] = pack10(&src[i..i + 10]);
            i += 10;
        } else if canPack(remaining, 8, 7) {
            dst[j] = pack8(&src[i..i + 8]);
            i += 8;
        } else if canPack(remaining, 7, 8) {
            dst[j] = pack7(&src[i..i + 7]);
            i += 7;
        } else if canPack(remaining, 6, 10) {
            dst[j] = pack6(&src[i..i + 6]);
            i += 6;
        } else if canPack(remaining, 5, 12) {
            dst[j] = pack5(&src[i..i + 5]);
            i += 5;
        } else if canPack(remaining, 4, 15) {
            dst[j] = pack4(&src[i..i + 4]);
            i += 4;
        } else if canPack(remaining, 3, 20) {
            dst[j] = pack3(&src[i..i + 3]);
            i += 3;
        } else if canPack(remaining, 2, 30) {
            dst[j] = pack2(&src[i..i + 2]);
            i += 2;
        } else if canPack(remaining, 1, 60) {
            dst[j] = pack1(&src[i..i + 1]);
            i += 1;
        } else {
            return Err("value out of bounds".to_string());
        }
        j += 1;
    }
    Ok(&dst[..j])
}

/// Returns a packed slice of the values from src.  If a value is over
/// 1 << 60, an error is returned.
pub fn EncodeAllRef(dst: &mut [u64], src: &[u64]) -> Result<usize, String> {
    let mut i = 0;
    let mut j = 0;

    loop {
        if i >= src.len() {
            break;
        }
        let remaining = &src[i..];

        if canPack(remaining, 240, 0) {
            (*dst)[j] = 0;
            i += 240;
        } else if canPack(remaining, 120, 0) {
            (*dst)[j] = 1 << 60;
            i += 120;
        } else if canPack(remaining, 60, 1) {
            (*dst)[j] = pack60(src[i: i + 60]);
            i += 60;
        } else if canPack(remaining, 30, 2) {
            (*dst)[j] = pack30(src[i: i + 30]);
            i += 30;
        } else if canPack(remaining, 20, 3) {
            (*dst)[j] = pack20(src[i: i + 20]);
            i += 20;
        } else if canPack(remaining, 15, 4) {
            (*dst)[j] = pack15(src[i: i + 15]);
            i += 15;
        } else if canPack(remaining, 12, 5) {
            (*dst)[j] = pack12(src[i: i + 12]);
            i += 12;
        } else if canPack(remaining, 10, 6) {
            (*dst)[j] = pack10(src[i: i + 10]);
            i += 10;
        } else if canPack(remaining, 8, 7) {
            (*dst)[j] = pack8(src[i: i + 8]);
            i += 8;
        } else if canPack(remaining, 7, 8) {
            (*dst)[j] = pack7(src[i: i + 7]);
            i += 7;
        } else if canPack(remaining, 6, 10) {
            (*dst)[j] = pack6(src[i: i + 6]);
            i += 6;
        } else if canPack(remaining, 5, 12) {
            (*dst)[j] = pack5(src[i: i + 5]);
            i += 5;
        } else if canPack(remaining, 4, 15) {
            (*dst)[j] = pack4(src[i: i + 4]);
            i += 4;
        } else if canPack(remaining, 3, 20) {
            (*dst)[j] = pack3(src[i: i + 3]);
            i += 3;
        } else if canPack(remaining, 2, 30) {
            (*dst)[j] = pack2(src[i: i + 2]);
            i += 2;
        } else if canPack(remaining, 1, 60) {
            (*dst)[j] = pack1(src[i: i + 1]);
            i += 1;
        } else {
            return Err("value out of bounds".to_string());
        }
        j += 1;
    }
    Ok(j)
}

pub fn Decode(dst: &mut [u64; 240], v: u64) -> Result<usize, String> {
    let sel = v >> 60;
    if sel >= 16 {
        return Err(format!("invalid selector value: {}", sel));
    }
    selector[sel].unpack(v, dst);
    Ok(selector[sel].n)
}

/// Writes the uncompressed values from src to dst. It returns the number
/// of values written or an error.
pub fn DecodeAll(dst: &mut [u64], src: &[u64]) -> Result<usize, String> {
    let mut j = 0;
    src.iter().try_for_each(|v| {
        let sel = v >> 60;
        if sel >= 16 {
            return Err(format!("invalid selector value: {}", sel));
        }
        // selector[sel].unpack(v, (*[240]uint64)(unsafe.Pointer(&dst[j])))
        unimplemented!("unpack"); // FIXME
        j += selector[sel].n;
        Ok(())
    })?;
    Ok(j)
}

// Returns true if n elements from in can be stored using bits per element.
fn canPack(src: &[u64], n: usize, bits: usize) -> bool {
    if src.len() < n {
        return false;
    }

    let mut end = src.len();
    if n < end {
        end = n;
    }

    // Selector 0,1 are special and use 0 bits to encode runs of 1's
    if bits == 0 {
        for i in 0..src.len() {
            if src[i] != 1 {
                return false;
            }
        }
        return true;
    }

    let max = ((1 << (bits as u64)) - 1) as u64;

    for i in 0..end {
        if src[i] > max {
            return false;
        }
    }

    true
}

// Packs 240 ones from in using 1 bit each.
fn pack240(src: &[u64]) -> u64 {
    0
}

// Packs 120 ones from in using 1 bit each.
fn pack120(src: &[u64]) -> u64 {
    0
}

// Packs 60 values from in using 1 bit each.
fn pack60(src: &[u64]) -> u64 {
    2 << 60
        | src[0]
        | src[1] << 1
        | src[2] << 2
        | src[3] << 3
        | src[4] << 4
        | src[5] << 5
        | src[6] << 6
        | src[7] << 7
        | src[8] << 8
        | src[9] << 9
        | src[10] << 10
        | src[11] << 11
        | src[12] << 12
        | src[13] << 13
        | src[14] << 14
        | src[15] << 15
        | src[16] << 16
        | src[17] << 17
        | src[18] << 18
        | src[19] << 19
        | src[20] << 20
        | src[21] << 21
        | src[22] << 22
        | src[23] << 23
        | src[24] << 24
        | src[25] << 25
        | src[26] << 26
        | src[27] << 27
        | src[28] << 28
        | src[29] << 29
        | src[30] << 30
        | src[31] << 31
        | src[32] << 32
        | src[33] << 33
        | src[34] << 34
        | src[35] << 35
        | src[36] << 36
        | src[37] << 37
        | src[38] << 38
        | src[39] << 39
        | src[40] << 40
        | src[41] << 41
        | src[42] << 42
        | src[43] << 43
        | src[44] << 44
        | src[45] << 45
        | src[46] << 46
        | src[47] << 47
        | src[48] << 48
        | src[49] << 49
        | src[50] << 50
        | src[51] << 51
        | src[52] << 52
        | src[53] << 53
        | src[54] << 54
        | src[55] << 55
        | src[56] << 56
        | src[57] << 57
        | src[58] << 58
        | src[59] << 59
}

// Packs 30 values from in using 2 bits each.
fn pack30(src: &[u64]) -> u64 {
    3 << 60
        | src[0]
        | src[1] << 2
        | src[2] << 4
        | src[3] << 6
        | src[4] << 8
        | src[5] << 10
        | src[6] << 12
        | src[7] << 14
        | src[8] << 16
        | src[9] << 18
        | src[10] << 20
        | src[11] << 22
        | src[12] << 24
        | src[13] << 26
        | src[14] << 28
        | src[15] << 30
        | src[16] << 32
        | src[17] << 34
        | src[18] << 36
        | src[19] << 38
        | src[20] << 40
        | src[21] << 42
        | src[22] << 44
        | src[23] << 46
        | src[24] << 48
        | src[25] << 50
        | src[26] << 52
        | src[27] << 54
        | src[28] << 56
        | src[29] << 58
}

// Packs 20 values from in using 3 bits each.
fn pack20(src: &[u64]) -> u64 {
    4 << 60
        | src[0]
        | src[1] << 3
        | src[2] << 6
        | src[3] << 9
        | src[4] << 12
        | src[5] << 15
        | src[6] << 18
        | src[7] << 21
        | src[8] << 24
        | src[9] << 27
        | src[10] << 30
        | src[11] << 33
        | src[12] << 36
        | src[13] << 39
        | src[14] << 42
        | src[15] << 45
        | src[16] << 48
        | src[17] << 51
        | src[18] << 54
        | src[19] << 57
}

// Packs 15 values from in using 3 bits each.
fn pack15(src: &[u64]) -> u64 {
    5 << 60
        | src[0]
        | src[1] << 4
        | src[2] << 8
        | src[3] << 12
        | src[4] << 16
        | src[5] << 20
        | src[6] << 24
        | src[7] << 28
        | src[8] << 32
        | src[9] << 36
        | src[10] << 40
        | src[11] << 44
        | src[12] << 48
        | src[13] << 52
        | src[14] << 56
}

// Packs 12 values from in using 5 bits each.
fn pack12(src: &[u64]) -> u64 {
    6 << 60
        | src[0]
        | src[1] << 5
        | src[2] << 10
        | src[3] << 15
        | src[4] << 20
        | src[5] << 25
        | src[6] << 30
        | src[7] << 35
        | src[8] << 40
        | src[9] << 45
        | src[10] << 50
        | src[11] << 55
}

// Packs 10 values from in using 6 bits each.
fn pack10(src: &[u64]) -> u64 {
    7 << 60
        | src[0]
        | src[1] << 6
        | src[2] << 12
        | src[3] << 18
        | src[4] << 24
        | src[5] << 30
        | src[6] << 36
        | src[7] << 42
        | src[8] << 48
        | src[9] << 54
}

// Packs 8 values from in using 7 bits each.
fn pack8(src: &[u64]) -> u64 {
    8 << 60
        | src[0]
        | src[1] << 7
        | src[2] << 14
        | src[3] << 21
        | src[4] << 28
        | src[5] << 35
        | src[6] << 42
        | src[7] << 49
}

// Packs 7 values from in using 8 bits each.
fn pack7(src: &[u64]) -> u64 {
    9 << 60
        | src[0]
        | src[1] << 8
        | src[2] << 16
        | src[3] << 24
        | src[4] << 32
        | src[5] << 40
        | src[6] << 48
}

// Packs 6 values from in using 10 bits each.
fn pack6(src: &[u64]) -> u64 {
    10 << 60 | src[0] | src[1] << 10 | src[2] << 20 | src[3] << 30 | src[4] << 40 | src[5] << 50
}

// Packs 5 values from in using 12 bits each.
fn pack5(src: &[u64]) -> u64 {
    11 << 60 | src[0] | src[1] << 12 | src[2] << 24 | src[3] << 36 | src[4] << 48
}

// Packs 4 values from in using 15 bits each.
fn pack4(src: &[u64]) -> u64 {
    12 << 60 | src[0] | src[1] << 15 | src[2] << 30 | src[3] << 45
}

// Packs 3 values from in using 20 bits each.
fn pack3(src: &[u64]) -> u64 {
    13 << 60 | src[0] | src[1] << 20 | src[2] << 40
}

// Packs 2 values from in using 30 bits each.
fn pack2(src: &[u64]) -> u64 {
    14 << 60 | src[0] | src[1] << 30
}

// Packs 1 values from in using 60 bits each.
fn pack1(src: &[u64]) -> u64 {
    15 << 60 | src[0]
}

fn unpack240(v: u64, dst: &mut [u64; 240]) {
    dst.fill(1)
    // dst.iter_mut().for_each(|v|{
    //     *v = 1;
    // });
}

fn unpack120(v: u64, dst: &mut [u64; 240]) {
    dst.fill(1);
}

fn unpack60(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 1;
    dst[1] = (v >> 1) & 1;
    dst[2] = (v >> 2) & 1;
    dst[3] = (v >> 3) & 1;
    dst[4] = (v >> 4) & 1;
    dst[5] = (v >> 5) & 1;
    dst[6] = (v >> 6) & 1;
    dst[7] = (v >> 7) & 1;
    dst[8] = (v >> 8) & 1;
    dst[9] = (v >> 9) & 1;
    dst[10] = (v >> 10) & 1;
    dst[11] = (v >> 11) & 1;
    dst[12] = (v >> 12) & 1;
    dst[13] = (v >> 13) & 1;
    dst[14] = (v >> 14) & 1;
    dst[15] = (v >> 15) & 1;
    dst[16] = (v >> 16) & 1;
    dst[17] = (v >> 17) & 1;
    dst[18] = (v >> 18) & 1;
    dst[19] = (v >> 19) & 1;
    dst[20] = (v >> 20) & 1;
    dst[21] = (v >> 21) & 1;
    dst[22] = (v >> 22) & 1;
    dst[23] = (v >> 23) & 1;
    dst[24] = (v >> 24) & 1;
    dst[25] = (v >> 25) & 1;
    dst[26] = (v >> 26) & 1;
    dst[27] = (v >> 27) & 1;
    dst[28] = (v >> 28) & 1;
    dst[29] = (v >> 29) & 1;
    dst[30] = (v >> 30) & 1;
    dst[31] = (v >> 31) & 1;
    dst[32] = (v >> 32) & 1;
    dst[33] = (v >> 33) & 1;
    dst[34] = (v >> 34) & 1;
    dst[35] = (v >> 35) & 1;
    dst[36] = (v >> 36) & 1;
    dst[37] = (v >> 37) & 1;
    dst[38] = (v >> 38) & 1;
    dst[39] = (v >> 39) & 1;
    dst[40] = (v >> 40) & 1;
    dst[41] = (v >> 41) & 1;
    dst[42] = (v >> 42) & 1;
    dst[43] = (v >> 43) & 1;
    dst[44] = (v >> 44) & 1;
    dst[45] = (v >> 45) & 1;
    dst[46] = (v >> 46) & 1;
    dst[47] = (v >> 47) & 1;
    dst[48] = (v >> 48) & 1;
    dst[49] = (v >> 49) & 1;
    dst[50] = (v >> 50) & 1;
    dst[51] = (v >> 51) & 1;
    dst[52] = (v >> 52) & 1;
    dst[53] = (v >> 53) & 1;
    dst[54] = (v >> 54) & 1;
    dst[55] = (v >> 55) & 1;
    dst[56] = (v >> 56) & 1;
    dst[57] = (v >> 57) & 1;
    dst[58] = (v >> 58) & 1;
    dst[59] = (v >> 59) & 1;
}

fn unpack30(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 3;
    dst[1] = (v >> 2) & 3;
    dst[2] = (v >> 4) & 3;
    dst[3] = (v >> 6) & 3;
    dst[4] = (v >> 8) & 3;
    dst[5] = (v >> 10) & 3;
    dst[6] = (v >> 12) & 3;
    dst[7] = (v >> 14) & 3;
    dst[8] = (v >> 16) & 3;
    dst[9] = (v >> 18) & 3;
    dst[10] = (v >> 20) & 3;
    dst[11] = (v >> 22) & 3;
    dst[12] = (v >> 24) & 3;
    dst[13] = (v >> 26) & 3;
    dst[14] = (v >> 28) & 3;
    dst[15] = (v >> 30) & 3;
    dst[16] = (v >> 32) & 3;
    dst[17] = (v >> 34) & 3;
    dst[18] = (v >> 36) & 3;
    dst[19] = (v >> 38) & 3;
    dst[20] = (v >> 40) & 3;
    dst[21] = (v >> 42) & 3;
    dst[22] = (v >> 44) & 3;
    dst[23] = (v >> 46) & 3;
    dst[24] = (v >> 48) & 3;
    dst[25] = (v >> 50) & 3;
    dst[26] = (v >> 52) & 3;
    dst[27] = (v >> 54) & 3;
    dst[28] = (v >> 56) & 3;
    dst[29] = (v >> 58) & 3;
}

fn unpack20(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 7;
    dst[1] = (v >> 3) & 7;
    dst[2] = (v >> 6) & 7;
    dst[3] = (v >> 9) & 7;
    dst[4] = (v >> 12) & 7;
    dst[5] = (v >> 15) & 7;
    dst[6] = (v >> 18) & 7;
    dst[7] = (v >> 21) & 7;
    dst[8] = (v >> 24) & 7;
    dst[9] = (v >> 27) & 7;
    dst[10] = (v >> 30) & 7;
    dst[11] = (v >> 33) & 7;
    dst[12] = (v >> 36) & 7;
    dst[13] = (v >> 39) & 7;
    dst[14] = (v >> 42) & 7;
    dst[15] = (v >> 45) & 7;
    dst[16] = (v >> 48) & 7;
    dst[17] = (v >> 51) & 7;
    dst[18] = (v >> 54) & 7;
    dst[19] = (v >> 57) & 7;
}

fn unpack15(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 15;
    dst[1] = (v >> 4) & 15;
    dst[2] = (v >> 8) & 15;
    dst[3] = (v >> 12) & 15;
    dst[4] = (v >> 16) & 15;
    dst[5] = (v >> 20) & 15;
    dst[6] = (v >> 24) & 15;
    dst[7] = (v >> 28) & 15;
    dst[8] = (v >> 32) & 15;
    dst[9] = (v >> 36) & 15;
    dst[10] = (v >> 40) & 15;
    dst[11] = (v >> 44) & 15;
    dst[12] = (v >> 48) & 15;
    dst[13] = (v >> 52) & 15;
    dst[14] = (v >> 56) & 15;
}

fn unpack12(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 31;
    dst[1] = (v >> 5) & 31;
    dst[2] = (v >> 10) & 31;
    dst[3] = (v >> 15) & 31;
    dst[4] = (v >> 20) & 31;
    dst[5] = (v >> 25) & 31;
    dst[6] = (v >> 30) & 31;
    dst[7] = (v >> 35) & 31;
    dst[8] = (v >> 40) & 31;
    dst[9] = (v >> 45) & 31;
    dst[10] = (v >> 50) & 31;
    dst[11] = (v >> 55) & 31;
}

fn unpack10(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 63;
    dst[1] = (v >> 6) & 63;
    dst[2] = (v >> 12) & 63;
    dst[3] = (v >> 18) & 63;
    dst[4] = (v >> 24) & 63;
    dst[5] = (v >> 30) & 63;
    dst[6] = (v >> 36) & 63;
    dst[7] = (v >> 42) & 63;
    dst[8] = (v >> 48) & 63;
    dst[9] = (v >> 54) & 63;
}

fn unpack8(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 127;
    dst[1] = (v >> 7) & 127;
    dst[2] = (v >> 14) & 127;
    dst[3] = (v >> 21) & 127;
    dst[4] = (v >> 28) & 127;
    dst[5] = (v >> 35) & 127;
    dst[6] = (v >> 42) & 127;
    dst[7] = (v >> 49) & 127;
}

fn unpack7(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 255;
    dst[1] = (v >> 8) & 255;
    dst[2] = (v >> 16) & 255;
    dst[3] = (v >> 24) & 255;
    dst[4] = (v >> 32) & 255;
    dst[5] = (v >> 40) & 255;
    dst[6] = (v >> 48) & 255;
}

fn unpack6(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 1023;
    dst[1] = (v >> 10) & 1023;
    dst[2] = (v >> 20) & 1023;
    dst[3] = (v >> 30) & 1023;
    dst[4] = (v >> 40) & 1023;
    dst[5] = (v >> 50) & 1023;
}

fn unpack5(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 4095;
    dst[1] = (v >> 12) & 4095;
    dst[2] = (v >> 24) & 4095;
    dst[3] = (v >> 36) & 4095;
    dst[4] = (v >> 48) & 4095;
}

fn unpack4(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 32767;
    dst[1] = (v >> 15) & 32767;
    dst[2] = (v >> 30) & 32767;
    dst[3] = (v >> 45) & 32767;
}

fn unpack3(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 1048575;
    dst[1] = (v >> 20) & 1048575;
    dst[2] = (v >> 40) & 1048575;
}

fn unpack2(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 1073741823;
    dst[1] = (v >> 30) & 1073741823;
}

fn unpack1(v: u64, dst: &mut [u64; 240]) {
    dst[0] = v & 1152921504606846975;
}
