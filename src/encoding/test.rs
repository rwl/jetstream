use crate::encoding::simple8b;

fn Test_Encode_NoValues(t: testing::T) {
    let inp: Vec<u64> = vec![];
    let (encoded, _) = simple8b.EncodeAll(inp);

    let decoded = vec![0; inp.len()];
    let (n, _) = simple8b.DecodeAll(decoded, encoded);

    if inp.len() != decoded[..n].len() {
        t.Fatalf("Len mismatch: got {}, exp {}", decoded.len(), inp.len());
    }
}

fn Test_TooBig(t: testing::T) {
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

fn Test_FewValues(t: testing::T) {
    testEncode(t, 20, 2);
}

fn Test_Encode_Multiple_Zeros(t: testing::T) {
    testEncode(t, 250, 0);
}

fn Test_Encode_Multiple_Ones(t: testing::T) {
    testEncode(t, 250, 1);
}

fn Test_Encode_Multiple_Large(t: testing::T) {
    testEncode(t, 250, 134);
}

fn Test_Encode_240Ones(t: testing::T) {
    testEncode(t, 240, 1);
}

fn Test_Encode_120Ones(t: testing::T) {
    testEncode(t, 120, 1);
}

fn Test_Encode_60(t: testing::T) {
    testEncode(t, 60, 1);
}

fn Test_Encode_30(t: testing::T) {
    testEncode(t, 30, 3);
}

fn Test_Encode_20(t: testing::T) {
    testEncode(t, 20, 7);
}

fn Test_Encode_15(t: testing::T) {
    testEncode(t, 15, 15);
}

fn Test_Encode_12(t: testing::T) {
    testEncode(t, 12, 31);
}

fn Test_Encode_10(t: testing::T) {
    testEncode(t, 10, 63);
}

fn Test_Encode_8(t: testing::T) {
    testEncode(t, 8, 127);
}

fn Test_Encode_7(t: testing::T) {
    testEncode(t, 7, 255);
}

fn Test_Encode_6(t: testing::T) {
    testEncode(t, 6, 1023);
}

fn Test_Encode_5(t: testing::T) {
    testEncode(t, 5, 4095);
}

fn Test_Encode_4(t: testing::T) {
    testEncode(t, 4, 32767);
}

fn Test_Encode_3(t: testing::T) {
    testEncode(t, 3, 1048575);
}

fn Test_Encode_2(t: testing::T) {
    testEncode(t, 2, 1073741823);
}

fn Test_Encode_1(t: testing::T) {
    testEncode(t, 1, 1152921504606846975);
}

fn testEncode(t: testing::T, n: usize, val: u64) {
    let mut enc = simple8b::Encoder::new();
    let mut inp = vec![0; n];
    for i in 0..n {
        inp[i] = val;
        enc.Write(inp[i]);
    }

    let (encoded, err) = enc.Bytes();
    if err != nil {
        t.Fatalf("Unexpected error: %v", err);
    }

    let mut dec = simple8b::Decoder::new(encoded);
    let mut i = 0;
    while dec.Next() {
        if i >= inp.len() {
            t.Fatalf("Decoded too many values: got {}, exp {}", i, inp.len());
        }

        if dec.Read() != inp[i] {
            t.Fatalf("Decoded[{}] != {}, got {}", i, inp[i], dec.Read());
        }
        i += 1;
    }

    {
        let (exp, got) = (n, i);
        if got != exp {
            t.Fatalf("Decode len mismatch: exp {}, got {}", exp, got);
        }
    }

    let (got, err) = simple8b::CountBytes(encoded);
    if err != nil {
        t.Fatalf("Unexpected error in Count: {}", err);
    }
    if got != n {
        t.Fatalf("Count mismatch: got {}, exp {}", got, n);
    }
}

fn Test_Bytes(t: testing::T) {
    let mut enc = simple8b::Encoder::new();
    for i in 0..30 {
        enc.Write(i as u64);
    }
    let (b, _) = enc.Bytes();

    let mut dec = simple8b::Decoder::new(b);
    let mut x = (0 as u64);
    while dec.Next() {
        if x != dec.Read() {
            t.Fatalf("mismatch: got {}, exp {}", dec.Read(), x);
        }
        x += 1;
    }
}

fn Test_Encode_ValueTooLarge(t: testing::T) {
    let mut enc = simple8b::Encoder::new();

    let values: [u64; 2] = [1442369134000000000, 0];

    values.iter().for_each(|&v| {
        enc.Write(v);
    });

    let (_, err) = enc.Bytes();
    if err == nil {
        t.Fatalf("Expected error, got nil")
    }
}

fn Test_Decode_NotEnoughBytes(t: testing::T) {
    let mut dec = simple8b::Decoder::new(vec![]);
    if dec.Next() {
        t.Fatalf("Expected Next to return false but it returned true")
    }
}

fn TestCountBytesBetween(t: testing::T) {
    let mut enc = simple8b::Encoder::new();
    let mut inp: Vec<u64> = vec![0; 8];
    for i in 0..inp.len() {
        inp[i] = i as u64;
        enc.Write(inp[i]);
    }

    let (encoded, err) = enc.Bytes();
    if err != nil {
        t.Fatalf("Unexpected error: {}", err);
    }

    let mut dec = simple8b::Decoder::new(encoded);
    let mut i = 0;
    while dec.Next() {
        if i >= inp.len() {
            t.Fatalf("Decoded too many values: got {}, exp {}", i, inp.len());
        }

        if dec.Read() != inp[i] {
            t.Fatalf("Decoded[{}] != {}, got {}", i, inp[i], dec.Read());
        }
        i += 1;
    }

    {
        let (exp, got) = (inp.len(), i);
        if got != exp {
            t.Fatalf("Decode len mismatch: exp {}, got {}", exp, got);
        }
    }

    let (got, err) = simple8b::CountBytesBetween(encoded, 2, 6);
    if err != nil {
        t.Fatalf("Unexpected error in Count: {}", err);
    }
    if got != 4 {
        t.Fatalf("Count mismatch: got {}, exp {}", got, 4);
    }
}

fn TestCountBytesBetween_SkipMin(t: testing::T) {
    let mut enc = simple8b::Encoder::new();
    let mut inp: Vec<u64> = vec![0; 8];
    for i in 0..inp.len() {
        inp[i] = (i as u64);
        enc.Write(inp[i]);
    }
    inp.push(100000);
    enc.Write(100000);

    let (encoded, err) = enc.Bytes();
    if err != nil {
        t.Fatalf("Unexpected error: {}", err);
    }

    let mut dec = simple8b::Decoder::new(encoded);
    let mut i = 0;
    while dec.Next() {
        if i >= inp.len() {
            t.Fatalf("Decoded too many values: got {}, exp {}", i, inp.len());
        }

        if dec.Read() != inp[i] {
            t.Fatalf("Decoded[{}] != {}, got {}", i, inp[i], dec.Read());
        }
        i += 1;
    }

    {
        let (exp, got) = (inp.len(), i);
        if got != exp {
            t.Fatalf("Decode len mismatch: exp {}, got {}", exp, got);
        }
    }

    let (got, err) = simple8b::CountBytesBetween(encoded, 100000, 100001);
    if err != nil {
        t.Fatalf("Unexpected error in Count: {}", err);
    }
    if got != 1 {
        t.Fatalf("Count mismatch: got {}, exp {}", got, 1);
    }
}

fn BenchmarkEncode(b: testing::B) {
    let mut total = 0;
    let x: Vec<u64> = vec![15; 1024];

    b.ResetTimer();
    for i in 0..b.N {
        simple8b::EncodeAll(x);
        b.SetBytes((x.len() * 8) as i64);
        total += x.len();
    }
}

fn BenchmarkEncoder(b: testing::B) {
    let x: Vec<u64> = vec![15; 1024];
    // for i := 0; i < len(x); i++ {
    // 	x[i] = uint64(15)
    // }

    let mut enc = simple8b::Encoder::new();
    b.ResetTimer();
    for i in 0..b.N {
        enc.SetValues(x);
        enc.Bytes();
        b.SetBytes((x.len() as i64) * 8);
    }
}

fn BenchmarkDecode(b: testing::B) {
    let mut total = 0;

    let x: Vec<u64> = vec![10; 1024];
    // for i := 0; i < len(x); i++ {
    // 	x[i] = uint64(10)
    // }
    let (y, _) = simple8b::EncodeAll(x);

    let decoded = vec![0; x.len()];

    b.ResetTimer();

    for i in 0..b.N {
        let (_, _) = simple8b::DecodeAll(decoded, y);
        b.SetBytes((decoded.len() * 8) as i64);
        total += decoded.len();
    }
}

fn BenchmarkDecoder(b: testing::B) {
    let mut enc = simple8b::Encoder::new();
    let x: Vec<u64> = vec![10; 1024];
    for i in 0..x.len() {
        // x[i] = uint64(10)
        enc.Write(x[i]);
    }
    let (y, _) = enc.Bytes();

    b.ResetTimer();

    let mut dec = simple8b::Decoder::new(y);
    for i in 0..b.N {
        dec.SetBytes(y);
        let mut j = 0;
        while dec.Next() {
            j += 1;
        }
        b.SetBytes((j * 8) as i64);
    }
}
