pub fn zig_zag_encode64(x: i64) -> u64 {
    ((x << 1) as u64 ^ ((x as i64) >> 63) as u64) as u64
}

pub fn zig_zag_decode64(v: u64) -> i64 {
    ((v >> 1) ^ ((((v & 1) as i64) << 63) >> 63) as u64) as i64
}
