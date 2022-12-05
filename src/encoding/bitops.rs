// Returns the number of bits required to store the value x.
pub(crate) fn msb32(mut x: u32) -> usize {
    let mut pos = 32;
    let temp = x >> 16;
    if temp != 0 {
        pos -= 16;
        x = temp;
    }
    let temp = x >> 8;
    if temp != 0 {
        pos -= 8;
        x = temp;
    }
    let temp = x >> 4;
    if temp != 0 {
        pos -= 4;
        x = temp;
    }
    let temp = x >> 2;
    if temp != 0 {
        pos -= 2;
        x = temp;
    }
    let temp = x >> 1;
    if temp != 0 {
        pos - 2
    } else {
        ((pos as u32) - x) as usize
    }
}

// Returns the number of bits required to store the value x.
fn msb64(n: u64) -> usize {
    if n <= 0 {
        return -1;
    }
    let mut r: usize = 0;
    let mut v: usize = 0;
    if n >= 1 << 32 {
        r += 32;
        v = uint(n >> 32);
    } else {
        v = uint(n);
    }
    if v >= 1 << 16 {
        r += 16;
        v >>= 16;
    }
    if v >= 1 << 8 {
        r += 8;
        v >>= 8;
    }
    if v >= 1 << 4 {
        r += 4;
        v >>= 4;
    }
    if v >= 1 << 2 {
        r += 2;
        v >>= 2;
    }
    r += v >> 1;
    r as usize
}

pub fn zig_zag_encode64(x: i64) -> u64 {
    u64(u64(x << 1) ^ u64((i64(x) >> 63)))
}

pub fn zig_zag_decode64(v: u64) -> i64 {
    i64((v >> 1) ^ u64((i64(v & 1) << 63) >> 63))
}
