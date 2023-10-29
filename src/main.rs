use sound_card::SoundCard;
use num_traits::ToPrimitive;

mod spectral_density;
mod sound_card;

fn main() {
    
    let device_id = "hw:CARD=sndrpihifiberry,DEV=0";
    let format = sound_card::config::Format::B32;
    let sampling_rate = sound_card::config::SamplingRates::Hz192000;
    let period_size: usize = 1024;

    let sound_card_config = sound_card::config::SoundCardConfig::new(device_id, format, sampling_rate, period_size);
    let sound_card = sound_card::alsa::AlsaSoundCard::<f64>::new(sound_card_config);
    let data: Vec<sound_card::ChannelData<f64>>;

    match sound_card.record(1, 1000) {
        Ok(res_data) => data = res_data,
        Err(error) => panic!("Unable to record: {}", error)
    };

    let sampling_rate_f64 = sampling_rate.value().to_f64().unwrap();
    let mut spec_density = Vec::<spectral_density::SpectralDensity::<f64>>::new(); 
    let mut i = 0usize;
    while i < data.len() {
        spec_density.push(spectral_density::SpectralDensity::<f64>::new(&data[i].channel_data, sampling_rate_f64, spectral_density::SpectralDensity::get_fft_number(sampling_rate_f64)));
        i += 1;
    }

    
}
