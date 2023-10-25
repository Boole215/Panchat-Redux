use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, StreamConfig, SampleFormat, Stream};
use ringbuf::{Producer, Consumer, HeapRb, SharedRb};
use std::mem::{MaybeUninit};
use std::sync::{Arc};
use std::error::Error;

struct AudioIO{
    input_stream: Stream,
    output_stream: Stream,
}

impl AudioIO {
    pub fn new(latency: f32) -> Result<Self, Box<dyn Error>>{
	let host = cpal::default_host();
	let input_device = host.default_input_device()
            .expect("failed to find input device");
	let output_device = host.default_output_device()
            .expect("failed to find output device");

	println!("Using input device: \"{}\"", input_device.name()?);
	println!("Using output device: \"{}\"", output_device.name()?);

	let input_config : cpal::SupportedStreamConfig = input_device.default_input_config()?;
	println!("{:?}", input_config);
	let input_stream_config : cpal::StreamConfig = input_config.into();
	let output_config : cpal::SupportedStreamConfig = output_device.default_output_config()?;
	println!("{:?}", output_config);
	let output_stream_config : cpal::StreamConfig = output_config.into();

	let latency: f32 = 11.0;
	let latency_frames = (latency / 1_000.0) * input_stream_config.sample_rate.0 as f32;
	let latency_samples = latency_frames as usize * input_stream_config.channels as usize;


	let ring = HeapRb::<f32>::new(latency_samples*2);
	let (mut producer, mut consumer) = ring.split();

	// Fill the samples with 0.0 equal to the length of the delay
	for _ in 0..latency_samples {
	    // The ring buffer has twice as much space as necessary to add latency here,
	    // so this should never fail
	    producer.push(0.0).unwrap();
	}

	let input_stream = input_device.build_input_stream(&input_stream_config,
							   move |data, _: &_| Self::input_data_fn(&mut producer, data),
							   err_fn, None).expect("Failed to create input stream");

	println!("Built input stream successfully.");

	let output_stream = output_device.build_output_stream(&output_stream_config,
							      move |data, _: &_| Self::output_data_fn(&mut consumer, data),
							      err_fn, None).expect("Failed to create output stream");
	println!("Successfully built streams");


	Ok(Self{
	    input_stream,
	    output_stream,
	})
    }

    fn input_data_fn(producer: &mut Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
		     data: &[f32]){
	let mut output_fell_behind = false;
	for &sample in data {
	    if producer.push(sample).is_err() {
		output_fell_behind = true;
	    }
	}
	if output_fell_behind {
	    eprintln!("Output stream fell behind: try increasing latency");
	}
    }

    fn output_data_fn(consumer: &mut Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
		      data: &mut [f32]){
	let mut input_fell_behind = false;
	let mut cur_sample = None;
	let mut write_counter: u8 = 2;
	for sample in data {
	    if write_counter == 2 {
		cur_sample = consumer.pop();
		write_counter = 0;
	    }
	    *sample = match cur_sample {
		Some(s) => {
		    write_counter += 1;
		    s
		}
		None => {
		    input_fell_behind = true;
		    f32::EQUILIBRIUM
		}
	    };
	}
	if input_fell_behind {
	    eprintln!("Input stream fell behind: try increasing latency");
	}
    }

}


fn main() -> Result<(), Box<dyn Error>> {
    let audio_io = AudioIO::new(11.0)?;
    audio_io.input_stream.play();
    audio_io.output_stream.play();


    println!("Playing for 3 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(audio_io);
    println!("Done!");
    Ok(())
}



fn err_fn(err: cpal::StreamError){
    eprintln!("an error occurred on stream: {}", err);
}
