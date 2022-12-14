use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jetstream::encoding::simple8b;

pub fn encode_benchmark(c: &mut Criterion) {
    const N: usize = 1024;
    let x: Vec<u64> = vec![15; N];
    let mut d: Vec<u64> = vec![0; N];

    c.bench_function("encode_all", |b| {
        b.iter(|| {
            simple8b::encode_all_ref(black_box(&mut d), black_box(&x)).unwrap();
        })
    });
}

// pub fn decode_benchmark(c: &mut Criterion) {
//     const N: usize = 1024;
//     let x: Vec<u64> = vec![10; N];
//     let mut y: Vec<u64> = vec![0; N];
//     simple8b::encode_all_ref(&mut y, &x).unwrap();
//     let mut z: Vec<u8> = vec![];
//     for i in 0..y.len() {
//         y[i].to_be_bytes().iter().for_each(|&bi| {
//             z.push(bi); // FIXME
//         })
//     }
//
//     c.bench_with_input(BenchmarkId::new("for_each", N), &z, |b, inp| {
//         b.iter(|| {
//             simple8b::for_each(black_box(inp), |d| {
//                 black_box(d);
//                 true
//             })
//             .unwrap();
//         })
//     });
// }

criterion_group!(benches, encode_benchmark);
criterion_main!(benches);
