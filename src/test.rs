use crate::decoder::Decoder;
use crate::emulator::Emulator;
use crate::encoder::Encoder;
use crate::jetstream::DatasetWithQuality;
use crate::testcase::{create_emulator, encode_and_decode, TESTS};
use std::io::stdout;
use std::io::Write;
use tabwriter::TabWriter;

fn create_input_data(
    ied: &mut Emulator,
    samples: usize,
    count_of_variables: usize,
    quality_change: bool,
) -> Vec<DatasetWithQuality> {
    let mut data = vec![DatasetWithQuality::new(count_of_variables); samples];
    // data.iter_mut().for_each(|d| {
    //     d.i32s = vec![0; count_of_variables];
    //     d.q = vec![0; count_of_variables];
    // });

    // generate data using IED emulator
    // the timestamp is a simple integer counter, starting from 0
    // for i := range data {
    data.iter_mut().enumerate().for_each(|(k, d)| {
        // compute emulated waveform data
        ied.step();

        // calculate timestamp
        d.t = k as u64;

        let i = ied.i.as_ref().unwrap();
        let v = ied.v.as_ref().unwrap();

        // set waveform data
        d.i32s[0] = (i.a * 1000.0) as i32;
        d.i32s[1] = (i.b * 1000.0) as i32;
        d.i32s[2] = (i.c * 1000.0) as i32;
        d.i32s[3] = ((i.a + i.b + i.c) * 1000.0) as i32;
        d.i32s[4] = (v.a * 100.0) as i32;
        d.i32s[5] = (v.b * 100.0) as i32;
        d.i32s[6] = (v.c * 100.0) as i32;
        d.i32s[7] = ((v.a + v.b + v.c) * 100.0) as i32;

        // set quality data
        d.q[0] = 0;
        d.q[1] = 0;
        d.q[2] = 0;
        d.q[3] = 0;
        d.q[4] = 0;
        d.q[5] = 0;
        d.q[6] = 0;
        d.q[7] = 0;

        if quality_change {
            if k == 2 {
                d.q[0] = 1
            } else if k == 3 {
                d.q[0] = 0x41
            }
        }
    });
    data
}

fn create_input_data_dual_ied(
    ied1: &mut Emulator,
    ied2: &mut Emulator,
    samples: usize,
    count_of_variables: usize,
    quality_change: bool,
) -> Vec<DatasetWithQuality> {
    let mut data = vec![DatasetWithQuality::new(count_of_variables); samples];
    // data.iter_mut().for_each(|d| {
    //     d.i32s = vec![0; count_of_variables];
    //     d.q = vec![0; count_of_variables];
    // });

    // generate data using IED emulator
    // the timestamp is a simple integer counter, starting from 0
    // for i := range data {
    data.iter_mut().enumerate().for_each(|(k, d)| {
        // compute emulated waveform data
        ied1.step();
        ied2.step();

        // calculate timestamp
        d.t = k as u64;

        let i1 = ied1.i.as_ref().unwrap();
        let v1 = ied1.v.as_ref().unwrap();
        let i2 = ied2.i.as_ref().unwrap();
        let v2 = ied2.v.as_ref().unwrap();

        // set waveform data
        d.i32s[0] = (v1.a * 100.0) as i32;
        d.i32s[1] = (v1.b * 100.0) as i32;
        d.i32s[2] = (v1.c * 100.0) as i32;
        d.i32s[3] = ((v1.a + v1.b + v1.c) * 100.0) as i32;
        d.i32s[4] = (v2.a * 100.0) as i32;
        d.i32s[5] = (v2.b * 100.0) as i32;
        d.i32s[6] = (v2.c * 100.0) as i32;
        d.i32s[7] = ((v2.a + v2.b + v2.c) * 100.0) as i32;

        d.i32s[8] = (i1.a * 1000.0) as i32;
        d.i32s[9] = (i1.b * 1000.0) as i32;
        d.i32s[10] = (i1.c * 1000.0) as i32;
        d.i32s[11] = ((i1.a + i1.b + i1.c) * 1000.0) as i32;
        d.i32s[12] = (i2.a * 1000.0) as i32;
        d.i32s[13] = (i2.b * 1000.0) as i32;
        d.i32s[14] = (i2.c * 1000.0) as i32;
        d.i32s[15] = ((i2.a + i2.b + i2.c) * 1000.0) as i32;

        // set quality data
        d.q[0] = 0;
        d.q[1] = 0;
        d.q[2] = 0;
        d.q[3] = 0;
        d.q[4] = 0;
        d.q[5] = 0;
        d.q[6] = 0;
        d.q[7] = 0;
        d.q[8] = 0;
        d.q[9] = 0;
        d.q[10] = 0;
        d.q[11] = 0;
        d.q[12] = 0;
        d.q[13] = 0;
        d.q[14] = 0;
        d.q[15] = 0;

        if quality_change {
            if k == 2 {
                d.q[0] = 1;
            } else if k == 3 {
                d.q[0] = 0x41;
            }
        }
    });
    data
}

#[test]
pub fn test_encode_decode() {
    // prepare table for presenting results

    let mut tab = TabWriter::new(stdout());

    write!(
        tab,
        "{}\n{}\n",
        [
            "samples", "sampling", "samples", "messages", "quality", "early", "spatial", "size",
            "size",
        ]
        .join("\t"),
        [
            "",
            "rate",
            "per message",
            "",
            "change",
            "stop",
            "refs",
            "(bytes)",
            "(%)",
        ]
        .join("\t")
    )
    .unwrap();

    // keys := make([]string, 0, len(tests))
    // for k := range tests {
    // 	keys = append(keys, k)
    // }
    // sort.Strings(keys)
    let mut keys = TESTS.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    keys.sort();

    // for _, name := range keys {
    keys.iter().for_each(|name| {
        // t.Parallel()
        let id = uuid::Uuid::new_v4();
        let test = TESTS.get(name).unwrap();

        // settings for IED emulator
        let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);

        // initialise data structure for input data
        let mut data: Vec<DatasetWithQuality> = if test.count_of_variables == 16 {
            let mut ied2: Emulator = create_emulator(test.sampling_rate, 0.0);
            create_input_data_dual_ied(
                &mut ied,
                &mut ied2,
                test.samples,
                test.count_of_variables,
                test.quality_change,
            )
        } else {
            create_input_data(
                &mut ied,
                test.samples,
                test.count_of_variables,
                test.quality_change,
            )
        };

        // create encoder and decoder
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

        if test.use_spatial_refs {
            stream.set_spatial_refs(
                test.count_of_variables,
                test.count_of_variables / 8,
                test.count_of_variables / 8,
                true,
            ); // TODO test include_neutral
            stream_decoder.set_spatial_refs(
                test.count_of_variables,
                test.count_of_variables / 8,
                test.count_of_variables / 8,
                true,
            ); // TODO test include_neutral
        }

        // encode the data
        // when each message is complete, decode
        let encode_stats = encode_and_decode(
            true,
            &mut data,
            &mut stream,
            &mut stream_decoder,
            test.count_of_variables,
            test.samples_per_message,
            test.early_encoding_stop,
        )
        .unwrap();

        let theory_bytes_per_message = if test.early_encoding_stop {
            test.count_of_variables * encode_stats.samples * 16
        } else {
            test.count_of_variables * test.samples_per_message * 16
        };
        let mean_bytes_per_message =
            (encode_stats.total_bytes as f64) / (encode_stats.messages as f64); // includes header overhead
        let percent = 100.0 * (mean_bytes_per_message as f64) / (theory_bytes_per_message as f64);
        // meanBytesWithoutHeader := float64(encode_stats.total_bytes-encode_stats.total_header_bytes) / float64(encode_stats.iterations)

        assert!(percent <= test.expected_size);

        write!(
            tab,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.1}\t{:.1}\n",
            encode_stats.samples,
            test.sampling_rate,
            test.samples_per_message,
            encode_stats.messages,
            test.quality_change,
            test.early_encoding_stop,
            test.use_spatial_refs,
            mean_bytes_per_message,
            percent
        )
        .unwrap();
    });

    // show table of results
    tab.flush().unwrap();
}

#[test]
fn test_wrong_id() {
    let id = uuid::Uuid::new_v4();
    let test = TESTS.get("a10-1").unwrap();

    // settings for IED emulator
    let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);
    let wrong_id: uuid::Uuid = uuid::Uuid::new_v4();

    // initialise data structure for input data
    let mut data: Vec<DatasetWithQuality> = create_input_data(
        &mut ied,
        test.samples,
        test.count_of_variables,
        test.quality_change,
    );

    // create encoder and decoder
    let mut stream = Encoder::new(
        id,
        test.count_of_variables,
        test.sampling_rate,
        test.samples_per_message,
    );
    let mut stream_decoder = Decoder::new(
        wrong_id,
        test.count_of_variables,
        test.sampling_rate,
        test.samples_per_message,
    );

    // encode the data
    // when each message is complete, decode
    let err = encode_and_decode(
        true,
        &mut data,
        &mut stream,
        &mut stream_decoder,
        test.count_of_variables,
        test.samples_per_message,
        test.early_encoding_stop,
    )
    .unwrap_err();
    assert_eq!(err, "IDs did not match");
}
