use crate::encoding::simple8b;

#[test]
fn test_encode_no_values() {
    let inp: Vec<u64> = vec![];
    let encoded = simple8b::encode_all(&inp).unwrap();

    let mut decoded = vec![0; inp.len()];
    let n = simple8b::decode_all(&mut decoded, &encoded).unwrap();

    if inp.len() != decoded[..n].len() {
        panic!("Len mismatch: got {}, exp {}", decoded.len(), inp.len());
    }
}

#[test]
fn test_too_big() {
    let values = 1;
    let mut inp: Vec<u64> = vec![0; values];
    for i in 0..values {
        inp[i] = 2 << 61 - 1;
    }
    simple8b::encode_all(&inp).expect_err("expected \"too big\" error");
}

#[test]
fn test_few_values() {
    test_encode(20, 2);
}

#[test]
fn test_encode_multiple_zeros() {
    test_encode(250, 0);
}

#[test]
fn test_encode_multiple_ones() {
    test_encode(250, 1);
}

#[test]
fn test_encode_multiple_large() {
    test_encode(250, 134);
}

#[test]
fn test_encode_240_ones() {
    test_encode(240, 1);
}

#[test]
fn test_encode_120_ones() {
    test_encode(120, 1);
}

#[test]
fn test_encode_60() {
    test_encode(60, 1);
}

#[test]
fn test_encode_30() {
    test_encode(30, 3);
}

#[test]
fn test_encode_20() {
    test_encode(20, 7);
}

#[test]
fn test_encode_15() {
    test_encode(15, 15);
}

#[test]
fn test_encode_12() {
    test_encode(12, 31);
}

#[test]
fn test_encode_10() {
    test_encode(10, 63);
}

#[test]
fn test_encode_8() {
    test_encode(8, 127);
}

#[test]
fn test_encode_7() {
    test_encode(7, 255);
}

#[test]
fn test_encode_6() {
    test_encode(6, 1023);
}

#[test]
fn test_encode_5() {
    test_encode(5, 4095);
}

#[test]
fn test_encode_4() {
    test_encode(4, 32767);
}

#[test]
fn test_encode_3() {
    test_encode(3, 1048575);
}

#[test]
fn test_encode_2() {
    test_encode(2, 1073741823);
}

#[test]
fn test_encode_1() {
    test_encode(1, 1152921504606846975);
}

fn test_encode(n: usize, val: u64) {
    let mut enc = simple8b::Encoder::new();
    let mut inp = vec![0; n];
    for i in 0..n {
        inp[i] = val;
        enc.write(inp[i]);
    }

    let encoded = enc.bytes().unwrap();

    let mut dec = simple8b::Decoder::new(encoded.clone());
    let mut i = 0;
    while dec.next() {
        assert!(
            i < inp.len(),
            "Decoded too many values: got {}, exp {}",
            i,
            inp.len()
        );

        assert_eq!(
            dec.read(),
            inp[i],
            "Decoded[{}] != {}, got {}",
            i,
            inp[i],
            dec.read()
        );
        i += 1;
    }

    {
        let (exp, got) = (n, i);
        assert_eq!(got, exp, "Decode len mismatch: exp {}, got {}", exp, got);
    }

    match simple8b::count_bytes(&encoded) {
        Err(err) => {
            panic!("Unexpected error in count: {}", err);
        }
        Ok(got) => {
            assert_eq!(got, n, "Count mismatch: got {}, exp {}", got, n);
        }
    }
}

#[test]
fn test_bytes() {
    let mut enc = simple8b::Encoder::new();
    for i in 0..30 {
        enc.write(i as u64);
    }
    let b = enc.bytes().unwrap();

    let mut dec = simple8b::Decoder::new(b);
    let mut x = (0 as u64);
    while dec.next() {
        if x != dec.read() {
            panic!("mismatch: got {}, exp {}", dec.read(), x);
        }
        x += 1;
    }
}

#[test]
fn test_encode__value_too_large() {
    let mut enc = simple8b::Encoder::new();

    let values: [u64; 2] = [1442369134000000000, 0];

    values.iter().for_each(|&v| {
        enc.write(v);
    });

    enc.bytes().expect_err("expected \"value too large\" error");
}

#[test]
fn test_decode__not_enough_bytes() {
    let mut dec = simple8b::Decoder::new(vec![]);
    assert!(
        !dec.next(),
        "expected next() to return false but it returned true"
    );
}

#[test]
fn test_count_bytes_between() {
    let mut enc = simple8b::Encoder::new();
    let mut inp: Vec<u64> = vec![0; 8];
    for i in 0..inp.len() {
        inp[i] = i as u64;
        enc.write(inp[i]);
    }

    let encoded = enc.bytes().unwrap();

    let mut dec = simple8b::Decoder::new(encoded.clone());
    let mut i = 0;
    while dec.next() {
        assert!(
            i < inp.len(),
            "decoded too many values: got {}, exp {}",
            i,
            inp.len()
        );

        assert_eq!(
            dec.read(),
            inp[i],
            "decoded[{}] != {}, got {}",
            i,
            inp[i],
            dec.read()
        );
        i += 1;
    }

    {
        let (exp, got) = (inp.len(), i);
        assert_eq!(got, exp, "decode len mismatch: exp {}, got {}", exp, got);
    }

    let got = simple8b::count_bytes_between(&encoded, 2, 6).unwrap();
    assert_eq!(got, 4, "count mismatch: got {}, exp {}", got, 4);
}

#[test]
fn test_count_bytes_between__skip_min() {
    let mut enc = simple8b::Encoder::new();
    let mut inp: Vec<u64> = vec![0; 8];
    for i in 0..inp.len() {
        inp[i] = (i as u64);
        enc.write(inp[i]);
    }
    inp.push(100000);
    enc.write(100000);

    let encoded = enc.bytes().unwrap();

    let mut dec = simple8b::Decoder::new(encoded.clone());
    let mut i = 0;
    while dec.next() {
        assert!(
            i < inp.len(),
            "decoded too many values: got {}, exp {}",
            i,
            inp.len()
        );

        assert_eq!(
            dec.read(),
            inp[i],
            "decoded[{}] != {}, got {}",
            i,
            inp[i],
            dec.read()
        );
        i += 1;
    }

    {
        let (exp, got) = (inp.len(), i);
        assert_eq!(got, exp, "decode len mismatch: exp {}, got {}", exp, got);
    }

    let got = simple8b::count_bytes_between(&encoded, 100000, 100001).unwrap();
    assert_eq!(got, 1, "count mismatch: got {}, exp {}", got, 1);
}
