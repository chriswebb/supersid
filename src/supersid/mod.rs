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