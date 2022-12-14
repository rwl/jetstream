use jetstream::*;
use rasciigraph::{plot, Config};
use std::env;
use std::time::Instant;

fn main() {
    // define settings
    let uuid = uuid::Uuid::new_v4();
    let variable_per_sample = 8; // number of "variables", such as voltages or currents. 8 is equivalent to IEC 61850-9-2 LE
    let system_frequency = 50.03; // Hz
    let sampling_rate = 4800; // Hz
    let samples_per_message = 480; // each message contains 100 ms of data

    let args: Vec<String> = env::args().collect();
    let quiet = args.len() > 1 && args[1] == "--quiet";

    // initialise an encoder
    let mut enc = Encoder::new(
        uuid,
        variable_per_sample,
        sampling_rate,
        samples_per_message,
    );

    // use the Synaptec "emulator" library to generate three-phase voltage and current test signals
    let mut emu = emulator::Emulator::new(sampling_rate, system_frequency);
    emu.i = Some(emulator::ThreePhaseEmulation {
        pos_seq_mag: 500.0,
        ..Default::default()
    });
    emu.v = Some(emulator::ThreePhaseEmulation {
        pos_seq_mag: 400000.0 / f64::sqrt(3.0) * f64::sqrt(2.0),
        ..Default::default()
    });

    // use emulator to generate test data
    let message_count = 10_000;
    let samples_to_encode = message_count * 480; // equates to 1 full message
    let mut data = create_input_data(&mut emu, samples_to_encode, variable_per_sample);

    let t0 = Instant::now();
    // loop through data samples and encode into Slipstream format
    // for d in 0..data.len() {
    data.iter_mut().for_each(|d| {
        let (buf, length) = enc.encode(d).unwrap();

        // check if message encoding has finished (or an error occurred)
        if length > 0 {
            // buf should now contain an encoded message, and can be send over the network or stored

            // print encoding performance results
            if !quiet {
                let theory_bytes = variable_per_sample * samples_per_message * 16;
                println!("Original data size: {} bytes", theory_bytes);
                println!(
                    "Encoded Slipstream message size: {} bytes ({:1.2} of original)",
                    buf.len(),
                    100.0 * (buf.len() as f64) / (theory_bytes as f64)
                );
            }

            // initialise a decoder
            let mut dec = Decoder::new(
                uuid,
                variable_per_sample,
                sampling_rate,
                samples_per_message,
            );

            // decode the message
            dec.decode_to_buffer(&buf, length).unwrap();

            // iterate through the decoded samples
            if !quiet {
                let mut decoded_data = vec![0.0; samples_to_encode];
                for i in 0..dec.out.len() {
                    // extract the phase A current values (at index '0') and convert to Amps
                    decoded_data[i] = (dec.out[i].i32s[0] as f64) / 1000.0;

                    // extract individual values
                    // for j in 0..dec.i32_count {
                    //     println!(
                    //         "timestamp: {} value: {} quality: {}",
                    //         dec.out[i].t, dec.out[i].i32s[j], dec.out[i].q[j]
                    //     );
                    // }
                }

                // print plot of decoded data in terminal
                println!("Decoded phase A current data:");
                let graph = plot(
                    decoded_data,
                    Config::default().with_height(10).with_width(80),
                );
                println!("{}", graph);
            }
        }
    });

    println!("Elapsed time = {:?}", t0.elapsed());
}

fn create_input_data(
    ied: &mut emulator::Emulator,
    samples: usize,
    count_of_variables: usize,
) -> Vec<DatasetWithQuality> {
    // intialise data structure
    // data := make([]slipstream.DatasetWithQuality, samples)
    // for i := range data {
    // 	data[i].int32s = make([]int32, count_of_variables)
    // 	data[i].Q = make([]uint32, count_of_variables)
    // }
    let mut data = vec![DatasetWithQuality::new(count_of_variables); samples];
    // data.iter_mut().for_each(|d| {
    //     d.i32s = vec![0; count_of_variables];
    //     d.q = vec![0; count_of_variables];
    // });

    // generate data using IED emulator
    // the timestamp is a simple integer counter, starting from 0
    data.iter_mut().enumerate().for_each(|(k, d)| {
        // compute emulated waveform data
        ied.step();

        // extract emulated data and store in Slipstream input structure:

        // emulate timestamp
        d.t = k as u64;

        let i = ied.i.as_mut().unwrap();
        let v = ied.v.as_mut().unwrap();

        // set waveform data for current and voltage
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
    });

    data
}
