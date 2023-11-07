use sound_card::{{SoundCard, alsa::AlsaSoundCardLink, SoundCardPlayer, SoundCardRecorder}};
use num_traits::ToPrimitive;
use supersid::config::StationConfig;

mod spectral_density;
mod sound_card;
mod supersid;
mod math;
//mod sound_card_sampler;



fn main() {

    // TODO: Turn following into config loads
    let device_id = "hw:CARD=sndrpihifiberry,DEV=0";
    let format = sound_card::config::Format::B32;
    let sampling_rate = sound_card::config::SamplingRate::Hz192000;
    let sampling_rate_f64 = sampling_rate.sample_value::<f64>();
    let period_size: usize = sampling_rate.value() / 100;

    let n: usize = 128;
    
    let sound_card_config = sound_card::config::SoundCardConfig::new(device_id, format, sampling_rate, period_size);
    let sound_card_config_playback = sound_card_config.clone();
    let sound_card = sound_card::alsa::AlsaSoundCard::<i32>::new(sound_card_config);
    let mut recorder = sound_card.create_alsa_recorder(2);
    let data: Vec<sound_card::ChannelData<i32>>;

    let mut stations = Vec::<StationConfig>::with_capacity(6);
    stations.push(StationConfig::new("NAA", 'r', 24000));
    stations.push(StationConfig::new("NLK", 'b', 24800));
    stations.push(StationConfig::new("NML", 'g', 25200));
    stations.push(StationConfig::new("NPM", 'c', 21400));
    stations.push(StationConfig::new("NWC", 'y', 19800));
    stations.push(StationConfig::new("JJI", 'k', 22200));

    // let sound_card_playback = sound_card::alsa::AlsaSoundCard::<f64>::new(sound_card_config_playback);
    // let mut player = sound_card.create_alsa_player(2);
    //player.link::<crate::sound_card::alsa::AlsaRecorder<f64>>(&mut recorder);
   //let mut player = sound_card.create_alsa_player(2);
    //player.link::<crate::sound_card::alsa::AlsaRecorder<f64>>(&mut recorder);

    // let mut data_1: Vec<i32> = math::generate_tone::<f64>(50000f64, sampling_rate_f64, sampling_rate.value(), 50000.).iter()
    //     .map(|x| x.to_i32().unwrap()).collect();
    // let mut data_2: Vec<i32> = math::generate_tone::<f64>(50000f64, sampling_rate_f64, sampling_rate.value(), 50000.).iter()
    //     .map(|x| x.to_i32().unwrap()).collect();
    
    // let data_1 = math::generate_tone::<f64>(10000f64, sampling_rate_f64, sampling_rate.value(), 1.);
    // let data_2 = math::generate_tone::<f64>(20000f64, sampling_rate_f64, sampling_rate.value(), 1.);

    // let mut chan_data_vec = Vec::new();
    // chan_data_vec.push(sound_card::ChannelData::<i32>::new(1, data_1));
    // chan_data_vec.push(sound_card::ChannelData::<i32>::new(2, data_2));
    // let mut data_1: Vec<i32> = math::generate_tone::<f64>(50000f64, sampling_rate_f64, sampling_rate.value(), 1024.).iter()
    // .map(|x| x.to_i32().unwrap()).collect();
    // let mut data_2: Vec<i32> = math::generate_tone::<f64>(14000f64, sampling_rate_f64, sampling_rate.value(), 1024.).iter()
    // .map(|x| x.to_i32().unwrap()).collect();

    
    let mut data_1: Vec<i32> = crate::math::generate_tone::<i32>(50000f64, sampling_rate_f64, sampling_rate.value(), 50000.);
    let mut data_2: Vec<i32> = crate::math::generate_tone::<i32>(14000f64, sampling_rate_f64, sampling_rate.value(), 50000.);

    data_1.append(&mut data_1.clone()); // 2
    data_1.append(&mut data_1.clone()); // 4
    data_1.append(&mut data_1.clone()); // 8
    data_1.append(&mut data_1.clone()); // 16

    data_2.append(&mut data_2.clone()); // 2
    data_2.append(&mut data_2.clone()); // 4
    data_2.append(&mut data_2.clone()); // 8
    data_2.append(&mut data_2.clone()); // 16
    
    let sound_card_playback = sound_card::alsa::AlsaSoundCard::<i32>::new(sound_card_config_playback);
    let mut player = sound_card_playback.create_alsa_player(2);

    let mut chan_data_vec = Vec::new();
    chan_data_vec.push(sound_card::ChannelData::<i32>::new(1, data_1));
    chan_data_vec.push(sound_card::ChannelData::<i32>::new(2, data_2));

    std::thread::spawn(move || {
        //player.link::<crate::sound_card::alsa::AlsaRecorder<f64>>(&mut recorder);
       //let mut player = sound_card.create_alsa_player(2);
        //player.link::<crate::sound_card::alsa::AlsaRecorder<f64>>(&mut recorder);
    
        

        //let data_2 = math::generate_tone::<f64>(60000f64, sampling_rate_f64, sampling_rate.value(), 80000.);
    

        loop {
            if let Err(error) = player.play(&chan_data_vec) {
                println!("Error occurred while tring to play sample: {}", error);
                return;
            }
    
        }
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
    //recorder.record(1000);
    //std::thread::sleep(std::time::Duration::from_millis(500));
    let start = std::time::Instant::now();

    match recorder.record(2000) {
        Ok(res_data) => data = res_data,
        Err(error) => panic!("Unable to record: {}", error)
    };

    let record_finish_time = std::time::Instant::now();

    // let mut all = false;

    // let mut n_trial = n;
    // while (!all) {
    //     let sd = spectral_density::SpectralDensity::<f64>::new(&data[i].channel_data, sampling_rate_f64, n_trial);

    // }

    let mut spec_density = Vec::<spectral_density::SpectralDensity::<f64>>::new(); 
    let mut i = 0usize;
    while i < data.len() {
        spec_density.push(crate::spectral_density::SpectralDensity::<f64>::new::<i32>(&data[i].channel_data, sampling_rate_f64, n));
        i += 1;
    }

    let sd_finish_time = std::time::Instant::now();

    // i = 0;
    // for sd_data in spec_density[0].data.iter() {
    //     println!("Freq {} Hz measured power: {} dB/Hz", sd_data.frequency(), sd_data.spectral_density_db());
    //     i += 1;
    // }
    println!("------------------------------------------");
    println!("Spectrum Record Length: {}", spec_density[0].data.len());
    println!("------------------------------------------");

    i = 0;
    let mut j;
    for sd_data in spec_density[0].data.iter() {
        j = 0;
        while j < stations.len() {
            let station = &stations[j];
            if i == station.get_bin(spec_density[0].freq_step) {
                println!("Station {} ({} Hz): measured frequency {} Hz; measured power: {} dB/Hz", station.callsign, station.frequency, sd_data.frequency(), sd_data.spectral_density_db());
            }
            j += 1;
        }
        i += 1;
    }

    println!("------------------------------------------");
    i=0;
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
    println!("------------------------------------------");

    let spectrum_size = std::mem::size_of::<crate::spectral_density::SpectralDensity<f64>>() + spec_density[0].data.len() * std::mem::size_of::<crate::spectral_density::SpectralDensitySample<f64, f64>>();
    let raw_data_size = std::mem::size_of::<crate::sound_card::ChannelData<f64>>() + data[0].channel_data.len() * std::mem::size_of::<f64>();

    println!("Spectrum size in bytes: {} ({} MB per hour) ({} MB per day) ({} GB per year)", spectrum_size, spectrum_size * 3600 / 1024 / 1024, spectrum_size * 3600 * 24 / 1024 / 1024 , spectrum_size * 3600 * 24 * 365 / 1024 / 1024 / 1024);
    println!("Raw data size in bytes: {}  ({} MB per hour) ({} GB per day) ({} TB per year", raw_data_size, raw_data_size * 3600 / 1024 / 1024, raw_data_size * 3600 * 24 / 1024 / 1024 / 1024, raw_data_size * 3600 * 24 * 365 / 1024 / 1024 / 1024 / 1024);
}
