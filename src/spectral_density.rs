use welch_sde::Build;
use std::iter::*;
use num_traits::{cast::FromPrimitive, float::Float};


// Transform Vec<Signal> to Vec<SpectralDensitySample<Frequency, SpectralDensityMeasure>>

#[derive(Debug)]
pub struct SpectralDensity<T: Measurement> {
    pub peak: Option<SpectralDensitySample<T, T>>,
    pub noise_floor: T,
    pub seems_off: bool,
    pub all_match: bool,
    pub data: std::collections::LinkedList<SpectralDensitySample<T, T>>
}

impl<T: Measurement> SpectralDensity<T> {
    pub fn new(data: &[T], audio_sampling_rate: T, num_ffts: usize) -> Self {
        let mut noise_total: Option<T> = None;
        let mut first_sample: Option<SpectralDensitySample<T, T>> = None;
        let mut peak_sample: Option<SpectralDensitySample<T, T>> = None;
        let mut seems_off: bool = true;
        let mut all_match: bool = true;
        let total_freqs: usize = data.len();
        let spectral_densities: std::collections::LinkedList<SpectralDensitySample<T, T>> = Self::get_welch_spectral_density(data, audio_sampling_rate, num_ffts).iter().map(|(freq, sd)| {         
            let sample: SpectralDensitySample<T, T> = SpectralDensitySample::new(*freq, *sd);
            if first_sample.is_none() {
                peak_sample = Some(sample);
                first_sample = Some(sample);
            }
            if sample.spectral_density >= (peak_sample.unwrap()).spectral_density { // >= used to ensure first_sample and peak won't match if they have the same sd
                peak_sample = Some(sample);
            }
            match noise_total {
                Some(noise) => noise_total = Some(noise + sample.spectral_density),
                None => noise_total = Some(sample.spectral_density),
            };
            return sample
        }).collect();
        
        let noise_floor = match noise_total {
            Some(noise) => noise / (T::from_usize(total_freqs).unwrap()),
            None => panic!("No samples to generate noise total")
        };

        if peak_sample.is_some() {
            if num_ffts > 1 {
                if (peak_sample.unwrap()).frequency == (first_sample.unwrap()).frequency || (peak_sample.unwrap()).spectral_density != (first_sample.unwrap()).spectral_density {
                    seems_off = false;
                    all_match = false;
                }
            }
            else {
                seems_off = false;
                all_match = true;
            }
        }

        Self {
            peak: peak_sample,
            noise_floor: noise_floor,
            data: spectral_densities,
            all_match: all_match,
            seems_off: seems_off
         }
    
    }
    /// Returns [Welch] [Builder] given the `signal` sampled at `fs`Hz
    pub fn get_welch_spectral_density(data: &[T], audio_sampling_rate: T, num_ffts: usize) -> Vec<(T, T)> {
        let welch_spectral_density: welch_sde::SpectralDensity<T> = welch_sde::SpectralDensity::<T>::builder(&data,audio_sampling_rate).n_segment(num_ffts).build();
        let periodogram: welch_sde::Periodogram<T> = welch_spectral_density.periodogram();
        let periodogram_map: Vec<(T, T)> = zip(periodogram.frequency(), &(*periodogram)).map(|(x, &y)| (x, y)).collect();
        return periodogram_map;
    }

    /// Returns the number of FFTs for the window of the welch spectral density
    pub fn get_fft_number(audio_sampling_rate: T) -> usize {
        // num_ffts = 1024 for 44100 and 48000,
        //            2048 for 96000,
        //            4096 for 192000
        // -> the frequency resolution is constant
        let audio_sampling_rate_usize: usize = audio_sampling_rate.to_usize().unwrap();
        if audio_sampling_rate_usize <= 48000 { 1024 } else { 1024 * audio_sampling_rate_usize / 48000 }
    }

    pub fn get_peak_freq(data: Vec<(T, T)>) -> T {
        let mut peak_freq: T = T::from_usize(0).unwrap();
        let mut first_sd: Option<T> = None;
        let mut max_sd: T = T::from_f64(f64::MIN).unwrap();
        for (freq, sd) in data {
            if first_sd.is_none() {
                max_sd = sd;
                first_sd = Some(sd);
            }
            if sd > max_sd {
                peak_freq = freq;
            }
        }
        return peak_freq;
    }
    
    pub fn get_noise_floor(data: Vec<(T, T)>) -> T {
        let mut noise_total: Option<T> = None;
        let total_freqs: usize = data.len();
        for (_, sd) in data {
            match noise_total {
                Some(noise) => noise_total = Some(noise + sd),
                None => noise_total = Some(sd),
            };
        }
    
        match noise_total {
            Some(noise) => noise / (T::from_usize(total_freqs).unwrap()),
            None => panic!("No samples to generate noise total")
        }
    }
    
    
}

#[derive(Debug, Clone, Copy)]
pub struct SpectralDensitySample<T: Measurement, U: Measurement>
{
    pub frequency: T,
    pub spectral_density: U,
}

impl<T: Measurement, U: Measurement> SpectralDensitySample<T, U> {
    pub fn new(freq: T, sd: U) -> Self {
        Self { frequency: freq, spectral_density: sd }
    }
}


pub trait Measurement:
    welch_sde::Signal + Float + std::iter::Sum + std::ops::SubAssign + std::ops::AddAssign + FromPrimitive
{
}
impl Measurement for f64 {}
impl Measurement for f32 {}

