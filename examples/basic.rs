use jetstream::emulator::{Emulator, ThreePhaseEmulation};
use jetstream::{DatasetWithQuality, Decoder, Encoder};
use rasciigraph::{plot, Config};

fn main() {
    // define settings
    let uuid = uuid::Uuid::new_v4();
    let variablePerSample = 8; // number of "variables", such as voltages or currents. 8 is equivalent to IEC 61850-9-2 LE
    let systemFrequency = 50.03; // Hz
    let samplingRate = 4800; // Hz
    let samplesPerMessage = 480; // each message contains 100 ms of data

    // initialise an encoder
    let mut enc = Encoder::new(uuid, variablePerSample, samplingRate, samplesPerMessage);

    // use the Synaptec "emulator" library to generate three-phase voltage and current test signals
    let mut emu = Emulator::new(samplingRate, systemFrequency);
    emu.I = Some(ThreePhaseEmulation {
        PosSeqMag: 500.0,
        ..Default::default()
    });
    emu.V = Some(ThreePhaseEmulation {
        PosSeqMag: 400000.0 / math.Sqrt(3) * math.Sqrt(2),
        ..Default::default()
    });

    // use emulator to generate test data
    let samplesToEncode = 480; // equates to 1 full message
    let mut data = createInputData(&mut emu, samplesToEncode, variablePerSample);

    // loop through data samples and encode into Slipstream format
    // for d in 0..data.len() {
    data.iter_mut().for_each(|d| {
        let (buf, length, err) = enc.Encode(d);

        // check if message encoding has finished (or an error occurred)
        if err == nil && length > 0 {
            // buf should now contain an encoded message, and can be send over the network or stored

            // print encoding performance results
            let theoryBytes = variablePerSample * samplesPerMessage * 16;
            println!("Original data size: {} bytes", theoryBytes);
            println!(
                "Encoded Slipstream message size: {} bytes ({:1.2} of original)",
                buf.len(),
                100.0 * (buf.len() as f64) / (theoryBytes as f64)
            );

            // initialise a decoder
            let mut dec = Decoder::new(uuid, variablePerSample, samplingRate, samplesPerMessage);

            // decode the message
            let errDecode = dec.DecodeToBuffer(buf, length);

            // iterate through the decoded samples
            if errDecode == nil {
                let mut decodedData = vec![0.0; samplesToEncode];
                for i in 0..dec.Out.len() {
                    // extract the phase A current values (at index '0') and convert to Amps
                    decodedData[i] = (dec.Out[i].Int32s[0] as f64) / 1000.0;

                    // extract individual values
                    for j in 0..dec.Int32Count {
                        println!(
                            "timestamp: {} value: {} quality: {}",
                            dec.Out[i].T, dec.Out[i].Int32s[j], dec.Out[i].Q[j]
                        );
                    }
                }

                // print plot of decoded data in terminal
                println!("Decoded phase A current data:");
                let graph = plot(
                    decodedData,
                    Config::default().with_height(10).with_width(80),
                );
                println!(graph);
            }
        }
    });
}

fn createInputData(
    ied: &mut Emulator,
    samples: usize,
    countOfVariables: usize,
) -> Vec<DatasetWithQuality> {
    // intialise data structure
    // data := make([]slipstream.DatasetWithQuality, samples)
    // for i := range data {
    // 	data[i].Int32s = make([]int32, countOfVariables)
    // 	data[i].Q = make([]uint32, countOfVariables)
    // }
    let mut data: Vec<DatasetWithQuality> = vec![DatasetWithQuality; samples];
    data.iter_mut().for_each(|d| {
        d.Int32s = vec![0; countOfVariables];
        d.Q = vec![0; countOfVariables];
    });

    // generate data using IED emulator
    // the timestamp is a simple integer counter, starting from 0
    // for i := range data {
    data.iter_mut().for_each(|d| {
        // compute emulated waveform data
        ied.Step();

        // extract emulated data and store in Slipstream input structure:

        // emulate timestamp
        d.T = i as u64;

        // set waveform data for current and voltage
        d.Int32s[0] = int32(ied.I.A * 1000.0) as i32;
        d.Int32s[1] = int32(ied.I.B * 1000.0) as i32;
        d.Int32s[2] = int32(ied.I.C * 1000.0) as i32;
        d.Int32s[3] = int32((ied.I.A + ied.I.B + ied.I.C) * 1000.0) as i32;
        d.Int32s[4] = int32(ied.V.A * 100.0) as i32;
        d.Int32s[5] = int32(ied.V.B * 100.0) as i32;
        d.Int32s[6] = int32(ied.V.C * 100.0) as i32;
        d.Int32s[7] = int32((ied.V.A + ied.V.B + ied.V.C) * 100.0) as i32;

        // set quality data
        d.Q[0] = 0;
        d.Q[1] = 0;
        d.Q[2] = 0;
        d.Q[3] = 0;
        d.Q[4] = 0;
        d.Q[5] = 0;
        d.Q[6] = 0;
        d.Q[7] = 0;
    });

    data
}
