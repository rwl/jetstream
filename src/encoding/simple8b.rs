use rand_distr::num_traits::ToPrimitive;

/// Implements the 64bit integer encoding algoritm as published by Ann and Moffat in
/// "Index compression using 64-bit words", Softw. Pract. Exper. 2010; 40:131–147
///
/// It is capable of encoding multiple integers with values between `0` and `1^60 -1`,
/// in a single word.

/// Simple8b is 64bit word-sized encoder that packs multiple integers into a single word using
/// a 4 bit selector values and up to 60 bits for the remaining values. Integers are encoded using
/// the following table:
///
/// ```txt
///     ┌──────────────┬─────────────────────────────────────────────────────────────┐
///     │  Selector    │      0    1   2   3   4   5   6   7  8  9  0 11 12 13 14 15 │
///     ├──────────────┼─────────────────────────────────────────────────────────────┤
///     │    Bits      │      0    0   1   2   3   4   5   6  7  8 10 12 15 20 30 60 │
///     ├──────────────┼─────────────────────────────────────────────────────────────┤
///     │     N        │    240  120  60  30  20  15  12  10  8  7  6  5  4  3  2  1 │
///     ├──────────────┼─────────────────────────────────────────────────────────────┤
///     │  Wasted Bits │     60   60   0   0   0   0  12   0  4  4  0  0  0  0  0  0 │
///     └──────────────┴─────────────────────────────────────────────────────────────┘
/// ```
///
/// For example, when the number of values can be encoded using 4 bits, selected 5 is encoded in the
/// 4 most significant bits followed by 15 values encoded used 4 bits each in the remaining 60 bits.

struct Packing {
    n: usize,
    bit: usize,
}

const SELECTOR: [Packing; 16] = [
    Packing { n: 240, bit: 0 },
    Packing { n: 120, bit: 0 },
    Packing { n: 60, bit: 1 },
    Packing { n: 30, bit: 2 },
    Packing { n: 20, bit: 3 },
    Packing { n: 15, bit: 4 },
    Packing { n: 12, bit: 5 },
    Packing { n: 10, bit: 6 },
    Packing { n: 8, bit: 7 },
    Packing { n: 7, bit: 8 },
    Packing { n: 6, bit: 10 },
    Packing { n: 5, bit: 12 },
    Packing { n: 4, bit: 15 },
    Packing { n: 3, bit: 20 },
    Packing { n: 2, bit: 30 },
    Packing { n: 1, bit: 60 },
];

pub fn for_each<F>(mut b: &[u8], mut f: F) -> Result<usize, String>
where
    F: FnMut(u64) -> bool,
{
    let mut count = 0;
    while b.len() >= 8 {
        // let v = binary.BigEndian.Uint64(b[..8]);
        let mut v = u64::from_be_bytes(b[..8].try_into().unwrap());
        b = &b[8..];
        // b = b.drain(0..8);
        count += 1;

        let sel = (v >> 60).to_usize().unwrap();
        if sel >= 16 {
            return Err(format!("invalid selector value: {}", sel));
        }

        let n = SELECTOR[sel].n;
        let bits = SELECTOR[sel].bit; // as usize;

        let mask = (!((!0 as i64) << bits)) as u64;

        for _ in 0..n {
            let val = v & mask;
            if !f(val) {
                return Ok(count);
            }
            v = v >> bits
        }
    }
    Ok(count)
}

/// Returns a packed slice of the values from src.  If a value is over
/// 1 << 60, an error is returned.
pub fn encode_all_ref(dst: &mut [u64], src: &[u64]) -> Result<usize, String> {
    let mut i = 0;
    let mut j = 0;

    loop {
        if i >= src.len() {
            break;
        }
        let remaining = &src[i..];

        if can_pack(remaining, 240, 0) {
            dst[j] = 0;
            i += 240;
        } else if can_pack(remaining, 120, 0) {
            dst[j] = 1 << 60;
            i += 120;
        } else if can_pack(remaining, 60, 1) {
            dst[j] = pack60(&src[i..i + 60]);
            i += 60;
        } else if can_pack(remaining, 30, 2) {
            dst[j] = pack30(&src[i..i + 30]);
            i += 30;
        } else if can_pack(remaining, 20, 3) {
            dst[j] = pack20(&src[i..i + 20]);
            i += 20;
        } else if can_pack(remaining, 15, 4) {
            dst[j] = pack15(&src[i..i + 15]);
            i += 15;
        } else if can_pack(remaining, 12, 5) {
            dst[j] = pack12(&src[i..i + 12]);
            i += 12;
        } else if can_pack(remaining, 10, 6) {
            dst[j] = pack10(&src[i..i + 10]);
            i += 10;
        } else if can_pack(remaining, 8, 7) {
            dst[j] = pack8(&src[i..i + 8]);
            i += 8;
        } else if can_pack(remaining, 7, 8) {
            dst[j] = pack7(&src[i..i + 7]);
            i += 7;
        } else if can_pack(remaining, 6, 10) {
            dst[j] = pack6(&src[i..i + 6]);
            i += 6;
        } else if can_pack(remaining, 5, 12) {
            dst[j] = pack5(&src[i..i + 5]);
            i += 5;
        } else if can_pack(remaining, 4, 15) {
            dst[j] = pack4(&src[i..i + 4]);
            i += 4;
        } else if can_pack(remaining, 3, 20) {
            dst[j] = pack3(&src[i..i + 3]);
            i += 3;
        } else if can_pack(remaining, 2, 30) {
            dst[j] = pack2(&src[i..i + 2]);
            i += 2;
        } else if can_pack(remaining, 1, 60) {
            dst[j] = pack1(&src[i..i + 1]);
            i += 1;
        } else {
            return Err("value out of bounds".to_string());
        }
        j += 1;
    }
    Ok(j)
}

// Returns true if n elements from in can be stored using bits per element.
fn can_pack(src: &[u64], n: usize, bits: usize) -> bool {
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

    let max = ((1_u64.wrapping_shl(bits as u32)) - 1) as u64;

    for i in 0..end {
        if src[i] > max {
            return false;
        }
    }

    true
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
