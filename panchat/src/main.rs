use cpal::{Data, Sample, SampleFormat, FromSample};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");

    let mut supported_configs_range = device.supported_output_configs()
	.expect("error while querying configs");
    let supported_config = supported_configs_range.next()
	.expect("no supported config?!")
	.with_max_sample_rate();

    let sample_format = supported_config.sample_format();
    let config = supported_config.into();


    let err_fn = |err| eprintln!("an error occured on the output audio stream: {}", err);

    let stream = match sample_format {
	SampleFormat::F32 => device.build_output_stream(&config, write_silence::<f32>, err_fn, None),
	SampleFormat::I16 => device.build_output_stream(&config, write_silence::<i16>, err_fn, None),
	SampleFormat::U16 => device.build_output_stream(&config, write_silence::<u16>, err_fn, None),
	sample_format => panic!("Unsupported sample format '{sample_format}'")
    }.unwrap();

    stream.play().unwrap();

    println!("Hello, world!");
}

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
	*sample = Sample::EQUILIBRIUM;
    }
}
