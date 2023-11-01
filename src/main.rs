use sound_card::SoundCard;
use num_traits::ToPrimitive;
use spectral_density::Measurement;

mod spectral_density;
mod sound_card;
mod sound_card_sampler;


pub struct StationConfig<T: Measurement>{
    pub callsign: String,
    pub color: char,
    pub frequency: usize,
    phantom: std::marker::PhantomData<T>
}

impl<T: Measurement> StationConfig<T>{
    pub fn new(callsign: &str, color: char, frequency: usize) -> Self {
        Self {
            callsign: callsign.to_string(),
            color: color,
            frequency: frequency,
            phantom: std::marker::PhantomData
        }
    }

    pub fn get_filter(&self, sampling_rate: sound_card::config::SamplingRate, nfft: usize) -> spectral_density::FrequencyFilter<T> {
        spectral_density::FrequencyFilter { frequency: T::from(self.frequency).unwrap(), bin: self.frequency * nfft / sampling_rate.value()}
    }
}

fn main() {


    
    // TODO: Turn following into config loads
    let device_id = "hw:CARD=sndrpihifiberry,DEV=0";
    let format = sound_card::config::Format::B32;
    let sampling_rate = sound_card::config::SamplingRate::Hz192000;
    let sampling_rate_f64 = sampling_rate.value().to_f64().unwrap();
    let period_size: usize = 1024;

    let num_ffts = spectral_density::SpectralDensity::get_fft_number(sampling_rate_f64) / 16;

    let sound_card_config = sound_card::config::SoundCardConfig::new(device_id, format, sampling_rate, period_size);
    let sound_card = sound_card::alsa::AlsaSoundCard::<f64>::new(sound_card_config);
    let mut recorder = sound_card.create_recorder(2);
    let data: Vec<sound_card::ChannelData<f64>>;

    let mut stations = Vec::<StationConfig<f64>>::with_capacity(6);
    stations.push(StationConfig::<f64>::new("NAA", 'r', 24000));
    stations.push(StationConfig::<f64>::new("NLK", 'b', 24800));
    stations.push(StationConfig::<f64>::new("NML", 'g', 25200));
    stations.push(StationConfig::<f64>::new("NPM", 'c', 21400));
    stations.push(StationConfig::<f64>::new("NWC", 'y', 19800));
    stations.push(StationConfig::<f64>::new("JJI", 'k', 22200));

    let station_filters: Vec<spectral_density::FrequencyFilter<f64>> = 
        stations.iter().map(|x| x.get_filter(sampling_rate, num_ffts)).collect();


    let start = std::time::Instant::now();

    match recorder.record(1000) {
        Ok(res_data) => data = res_data,
        Err(error) => panic!("Unable to record: {}", error)
    };

    let record_finish_time = std::time::Instant::now();

    let mut spec_density = Vec::<spectral_density::SpectralDensity::<f64>>::new(); 
    let mut i = 0usize;
    while i < data.len() {
        spec_density.push(spectral_density::SpectralDensity::<f64>::new(&data[i].channel_data, sampling_rate_f64, num_ffts));
        i += 1;
    }

    i = 0;
    
    let sd_finish_time = std::time::Instant::now();

    let record_elapsed = record_finish_time.duration_since(start).as_nanos().to_f64().unwrap() / 1000f64 / 1000f64;
    let spectral_density_elapsed = sd_finish_time.duration_since(record_finish_time).as_nanos().to_f64().unwrap() / 1000f64 / 1000f64;

    println!("Recording duration: {} ms", record_elapsed);
    println!("Spectral Density creation duration: {} ms", spectral_density_elapsed);

}
