use num_traits::ToPrimitive;
use crate::sound_card::SoundCard;
pub mod config;

/// Returns N for the window of the welch spectral density
#[allow(non_snake_case)]
pub fn get_N<T: crate::spectral_density::Measurement>(audio_sampling_rate: T) -> usize {
    // N = 1024 for 44100 and 48000,
    //     2048 for 96000,
    //     4096 for 192000
    // -> the frequency resolution is constant
    let audio_sampling_rate_usize: usize = audio_sampling_rate.to_usize().unwrap();
    if audio_sampling_rate_usize <= 48000 { 1024 } else { 1024 * audio_sampling_rate_usize / 48000 }
}

pub struct SuperSid<'a, T: crate::sound_card::Sample, U: crate::spectral_density::Measurement> {
    pub config: &'a config::SuperSidConfig,
    pub raw_data: Vec<crate::sound_card::ChannelData<T>>,
    pub spectrum: Vec<crate::spectral_density::SpectralDensity<U>>,
    pub station_data: Vec<super::spectral_density::SpectralDensitySample<U, U>>
}


impl<'a, T: crate::sound_card::Sample, U: crate::spectral_density::Measurement> SuperSid<'a, T, U> {
    pub fn measure(config: &'a config::SuperSidConfig) {
        
        let sampling_rate_f64 = config.sound_card.sampling_rate.sample_value::<f64>();

        let n: usize = 256;
        
        let sound_card = crate::sound_card::alsa::AlsaSoundCard::<f64>::new(config.sound_card.clone());
        let mut recorder = sound_card.create_recorder(2);
        let data: Vec<crate::sound_card::ChannelData<f64>>;

        let start = std::time::Instant::now();

        match recorder.record(1000) {
            Ok(res_data) => data = res_data,
            Err(error) => panic!("Unable to record: {}", error)
        };

        let record_finish_time = std::time::Instant::now();

        let mut spec_density = Vec::<crate::spectral_density::SpectralDensity::<f64>>::new(); 
        let mut i = 0usize;
        while i < data.len() {
            spec_density.push(crate::spectral_density::SpectralDensity::<f64>::new(&data[i].channel_data, sampling_rate_f64, n));
            i += 1;
        }

        i = 0;
        
        let sd_finish_time = std::time::Instant::now();

        while i < spec_density.len() {
            crate::spectral_density::plotter::plot_spectrum::<f64>(&spec_density[i], format!("spectral_density_channel_{}.png", i+1), None, Some("Frequency [Hz]"), Some("Spectral density [s^2/Hz]"));
            i += 1;
        }

        let plot_finish_time = std::time::Instant::now();

        let record_elapsed = record_finish_time.duration_since(start).as_nanos().to_f64().unwrap() / 1000f64 / 1000f64;
        let spectral_density_elapsed = sd_finish_time.duration_since(record_finish_time).as_nanos().to_f64().unwrap() / 1000f64 / 1000f64;
        let plot_elapsed = plot_finish_time.duration_since(sd_finish_time).as_nanos().to_f64().unwrap() / 1000f64 / 1000f64;

        println!("Recording duration: {} ms", record_elapsed);
        println!("Spectral Density creation duration: {} ms", spectral_density_elapsed);
        println!("Plot elapsed duration: {} ms", plot_elapsed);
    }
}

