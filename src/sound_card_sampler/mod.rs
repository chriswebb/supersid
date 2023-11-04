pub trait SpectrumSample:
    crate::sound_card::Sample + crate::spectral_density::Measurement
{

}

#[allow(non_snake_case)]
pub fn get_spectrum<T: SpectrumSample, U: crate::sound_card::SoundCard<T>>(sound_card: &U) -> Vec<crate::spectral_density::SpectralDensity<T>> {
    let sampling_rate_u = T::from(sound_card.config().sampling_rate.value()).unwrap();

    let mut recorder = sound_card.create_recorder(2);
    let N = crate::supersid::get_N(sampling_rate_u) / 16;
    
    let data: Vec<crate::sound_card::ChannelData<T>>;

    match recorder.record(1000) {
        Ok(res_data) => data = res_data,
        Err(error) => panic!("Unable to record: {}", error)
    };

    let mut spec_density = Vec::<crate::spectral_density::SpectralDensity::<T>>::new(); 
    let mut i = 0usize;
    while i < data.len() {
        spec_density.push(crate::spectral_density::SpectralDensity::<T>::new(&data[i].channel_data, sampling_rate_u, N));
        i += 1;
    }

    return spec_density;
}