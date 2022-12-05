use crate::encoding::simple8b;

#[test]
fn test_encode_no_values(t: testing::T) {
    let inp: Vec<u64> = vec![];
    let (encoded, _) = simple8b.EncodeAll(inp);

    let decoded = vec![0; inp.len()];
    let (n, _) = simple8b.DecodeAll(decoded, encoded);

    if inp.len() != decoded[..n].len() {
        t.Fatalf("Len mismatch: got {}, exp {}", decoded.len(), inp.len());
    }
}

#[test]
fn test_too_big(t: testing::T) {
    let values = 1;
    let mut inp: Vec<u64> = vec![0; values];
    for i in 0..values {
        inp[i] = 2 << 61 - 1;
    }
    let (_, err) = simple8b.EncodeAll(inp);
    if err == nil {
        t.Fatalf("expected error, got nil")
    }
}

#[test]
fn test_few_values(t: testing::T) {
    test_encode(t, 20, 2);
}

#[test]
fn test_encode_multiple_zeros(t: testing::T) {
    test_encode(t, 250, 0);
}

#[test]
fn test_encode_multiple_ones(t: testing::T) {
    test_encode(t, 250, 1);
}

#[test]
fn test_encode_multiple_large(t: testing::T) {
    test_encode(t, 250, 134);
}

#[test]
fn test_encode_240_ones(t: testing::T) {
    test_encode(t, 240, 1);
}

#[test]
fn test_encode_120_ones(t: testing::T) {
    test_encode(t, 120, 1);
}

#[test]
fn test_encode_60(t: testing::T) {
    test_encode(t, 60, 1);
}

#[test]
fn test_encode_30(t: testing::T) {
    test_encode(t, 30, 3);
}

#[test]
fn test_encode_20(t: testing::T) {
    test_encode(t, 20, 7);
}

#[test]
fn test_encode_15(t: testing::T) {
    test_encode(t, 15, 15);
}

#[test]
fn test_encode_12(t: testing::T) {
    test_encode(t, 12, 31);
}

#[test]
fn test_encode_10(t: testing::T) {
    test_encode(t, 10, 63);
}

#[test]
fn test_encode_8(t: testing::T) {
    test_encode(t, 8, 127);
}

#[test]
fn test_encode_7(t: testing::T) {
    test_encode(t, 7, 255);
}

#[test]
fn test_encode_6(t: testing::T) {
    test_encode(t, 6, 1023);
}

#[test]
fn test_encode_5(t: testing::T) {
    test_encode(t, 5, 4095);
}

#[test]
fn test_encode_4(t: testing::T) {
    test_encode(t, 4, 32767);
}

#[test]
fn test_encode_3(t: testing::T) {
    test_encode(t, 3, 1048575);
}

#[test]
fn test_encode_2(t: testing::T) {
    test_encode(t, 2, 1073741823);
}

#[test]
fn test_encode_1(t: testing::T) {
    test_encode(t, 1, 1152921504606846975);
}

fn test_encode(t: testing::T, n: usize, val: u64) {
    let mut enc = simple8b::Encoder::new();
    let mut inp = vec![0; n];
    for i in 0..n {
        inp[i] = val;
        enc.write(inp[i]);
    }

    let (encoded, err) = enc.bytes();
    if err != nil {
        t.Fatalf("Unexpected error: %v", err);
    }

    let mut dec = simple8b::Decoder::new(encoded);
    let mut i = 0;
    while dec.next() {
        if i >= inp.len() {
            t.Fatalf("Decoded too many values: got {}, exp {}", i, inp.len());
        }

        if dec.read() != inp[i] {
            t.Fatalf("Decoded[{}] != {}, got {}", i, inp[i], dec.read());
        }
        i += 1;
    }

    {
        let (exp, got) = (n, i);
        if got != exp {
            t.Fatalf("Decode len mismatch: exp {}, got {}", exp, got);
        }
    }

    let (got, err) = simple8b::count_bytes(encoded);
    if err != nil {
        t.Fatalf("Unexpected error in count: {}", err);
    }
    if got != n {
        t.Fatalf("Count mismatch: got {}, exp {}", got, n);
    }
}

#[test]
fn test_bytes(t: testing::T) {
    let mut enc = simple8b::Encoder::new();
    for i in 0..30 {
        enc.write(i as u64);
    }
    let (b, _) = enc.bytes();

    let mut dec = simple8b::Decoder::new(b);
    let mut x = (0 as u64);
    while dec.next() {
        if x != dec.read() {
            t.Fatalf("mismatch: got {}, exp {}", dec.read(), x);
        }
        x += 1;
    }
}

#[test]
fn test_encode__value_too_large(t: testing::T) {
    let mut enc = simple8b::Encoder::new();

    let values: [u64; 2] = [1442369134000000000, 0];

    values.iter().for_each(|&v| {
        enc.write(v);
    });

    let (_, err) = enc.bytes();
    if err == nil {
        t.Fatalf("Expected error, got nil")
    }
}

#[test]
fn test_decode__not_enough_bytes(t: testing::T) {
    let mut dec = simple8b::Decoder::new(vec![]);
    if dec.next() {
        t.Fatalf("Expected next to return false but it returned true")
    }
}

#[test]
fn test_count_bytes_between(t: testing::T) {
    let mut enc = simple8b::Encoder::new();
    let mut inp: Vec<u64> = vec![0; 8];
    for i in 0..inp.len() {
        inp[i] = i as u64;
        enc.write(inp[i]);
    }

    let (encoded, err) = enc.bytes();
    if err != nil {
        t.Fatalf("Unexpected error: {}", err);
    }

    let mut dec = simple8b::Decoder::new(encoded);
    let mut i = 0;
    while dec.next() {
        if i >= inp.len() {
            t.Fatalf("Decoded too many values: got {}, exp {}", i, inp.len());
        }

        if dec.read() != inp[i] {
            t.Fatalf("Decoded[{}] != {}, got {}", i, inp[i], dec.read());
        }
        i += 1;
    }

    {
        let (exp, got) = (inp.len(), i);
        if got != exp {
            t.Fatalf("Decode len mismatch: exp {}, got {}", exp, got);
        }
    }

    let (got, err) = simple8b::count_bytes_between(encoded, 2, 6);
    if err != nil {
        t.Fatalf("Unexpected error in count: {}", err);
    }
    if got != 4 {
        t.Fatalf("Count mismatch: got {}, exp {}", got, 4);
    }
}

#[test]
fn test_count_bytes_between__skip_min(t: testing::T) {
    let mut enc = simple8b::Encoder::new();
    let mut inp: Vec<u64> = vec![0; 8];
    for i in 0..inp.len() {
        inp[i] = (i as u64);
        enc.write(inp[i]);
    }
    inp.push(100000);
    enc.write(100000);

    let (encoded, err) = enc.bytes();
    if err != nil {
        t.Fatalf("Unexpected error: {}", err);
    }

    let mut dec = simple8b::Decoder::new(encoded);
    let mut i = 0;
    while dec.next() {
        if i >= inp.len() {
            t.Fatalf("Decoded too many values: got {}, exp {}", i, inp.len());
        }

        if dec.read() != inp[i] {
            t.Fatalf("Decoded[{}] != {}, got {}", i, inp[i], dec.read());
        }
        i += 1;
    }

    {
        let (exp, got) = (inp.len(), i);
        if got != exp {
            t.Fatalf("Decode len mismatch: exp {}, got {}", exp, got);
        }
    }

    let (got, err) = simple8b::count_bytes_between(encoded, 100000, 100001);
    if err != nil {
        t.Fatalf("Unexpected error in count: {}", err);
    }
    if got != 1 {
        t.Fatalf("Count mismatch: got {}, exp {}", got, 1);
    }
}

#[test]
fn benchmark_encode(b: testing::B) {
    let mut total = 0;
    let x: Vec<u64> = vec![15; 1024];

    b.ResetTimer();
    for i in 0..b.N {
        simple8b::encode_all(&x);
        b.SetBytes((x.len() * 8) as i64);
        total += x.len();
    }
}

#[test]
fn benchmark_encoder(b: testing::B) {
    let x: Vec<u64> = vec![15; 1024];
    // for i := 0; i < len(x); i++ {
    // 	x[i] = uint64(15)
    // }

    let mut enc = simple8b::Encoder::new();
    b.ResetTimer();
    for i in 0..b.N {
        enc.set_values(x.clone());
        enc.bytes();
        b.SetBytes((x.len() as i64) * 8);
    }
}

#[test]
fn benchmark_decode(b: testing::B) {
    let mut total = 0;

    let x: Vec<u64> = vec![10; 1024];
    // for i := 0; i < len(x); i++ {
    // 	x[i] = uint64(10)
    // }
    let (y, _) = simple8b::encode_all(&x);

    let mut decoded = vec![0; x.len()];

    b.ResetTimer();

    for i in 0..b.N {
        let (_, _) = simple8b::decode_all(&mut decoded, y);
        b.SetBytes((decoded.len() * 8) as i64);
        total += decoded.len();
    }
}

#[test]
fn benchmark_decoder(b: testing::B) {
    let mut enc = simple8b::Encoder::new();
    let x: Vec<u64> = vec![10; 1024];
    for i in 0..x.len() {
        // x[i] = uint64(10)
        enc.write(x[i]);
    }
    let (y, _) = enc.bytes();

    b.ResetTimer();

    let mut dec = simple8b::Decoder::new(y);
    for i in 0..b.N {
        dec.set_bytes(y);
        let mut j = 0;
        while dec.next() {
            j += 1;
        }
        b.SetBytes((j * 8) as i64);
    }
}
