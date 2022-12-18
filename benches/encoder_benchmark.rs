use criterion::{criterion_group, criterion_main, Criterion};
use jetstream::emulator::Emulator;
use jetstream::testcase::{create_emulator, create_input_data, encode_and_decode, TESTS};
use jetstream::{DatasetWithQuality, Decoder, Encoder};
use uuid::Uuid;

pub fn encode_decode_benchmark(c: &mut Criterion) {
    let mut keys = TESTS.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    keys.sort();

    keys.iter().for_each(|name| {
        let test = TESTS.get(name).unwrap();

        // settings for IED emulator
        let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);

        // initialise data structure for input data
        let mut data: Vec<DatasetWithQuality> = create_input_data(
            &mut ied,
            test.samples,
            test.count_of_variables,
            test.quality_change,
        );

        // create encoder and decoder
        let id = Uuid::new_v4();
        let mut stream = Encoder::new(
            id,
            test.count_of_variables,
            test.sampling_rate,
            test.samples_per_message,
        );
        let mut stream_decoder = Decoder::new(
            id,
            test.count_of_variables,
            test.sampling_rate,
            test.samples_per_message,
        );

        c.bench_function(name, |b| {
            b.iter(|| {
                // encode the data
                // when each message is complete, decode
                encode_and_decode(
                    false,
                    &mut data,
                    &mut stream,
                    &mut stream_decoder,
                    test.count_of_variables,
                    test.samples_per_message,
                    test.early_encoding_stop,
                )
                .unwrap();
            });
        });
    });
}

pub fn encode_benchmark(c: &mut Criterion) {
    let mut keys = TESTS.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    keys.sort();

    keys.iter().for_each(|name| {
        let test = TESTS.get(name).unwrap();

        // settings for IED emulator
        let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);

        // initialise data structure for input data
        let mut data: Vec<DatasetWithQuality> = create_input_data(
            &mut ied,
            test.samples,
            test.count_of_variables,
            test.quality_change,
        );

        // create encoder and decoder
        let id = Uuid::new_v4();
        let mut enc = Encoder::new(
            id,
            test.count_of_variables,
            test.sampling_rate,
            test.samples_per_message,
        );
        let _dec = Decoder::new(
            id,
            test.count_of_variables,
            test.sampling_rate,
            test.samples_per_message,
        );

        c.bench_function(name, |b| {
            b.iter(|| {
                data.iter_mut().for_each(|d| {
                    let (_buf, len) = enc.encode(d).unwrap();

                    if len > 0 {
                        // b.StopTimer();
                        // dec.DecodeToBuffer(buf, len);
                        // b.StartTimer();
                    }
                });
            });
        });
    });
}

pub fn decode_benchmark(c: &mut Criterion) {
    let mut keys = TESTS.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    keys.sort();

    keys.iter().for_each(|name| {
        let test = TESTS.get(name).unwrap();

        // settings for IED emulator
        let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);

        // initialise data structure for input data
        let mut data: Vec<DatasetWithQuality> = create_input_data(
            &mut ied,
            test.samples,
            test.count_of_variables,
            test.quality_change,
        );

        // create encoder and decoder
        let id = Uuid::new_v4();
        let mut enc = Encoder::new(
            id,
            test.count_of_variables,
            test.sampling_rate,
            test.samples_per_message,
        );
        let mut dec = Decoder::new(
            id,
            test.count_of_variables,
            test.sampling_rate,
            test.samples_per_message,
        );

        c.bench_function(name, |b| {
            b.iter(|| {
                // b.StopTimer();

                // for d := range data {
                data.iter_mut().for_each(|d| {
                    let (buf, len) = enc.encode(d).unwrap();

                    if len > 0 {
                        // b.StartTimer();
                        dec.decode_to_buffer(&buf, len).unwrap();
                        // b.StopTimer();
                    }
                });
            });
        });
    });
}

criterion_group!(
    benches,
    encode_decode_benchmark,
    encode_benchmark,
    decode_benchmark
);
criterion_main!(benches);
