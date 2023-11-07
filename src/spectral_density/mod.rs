use welch_sde::Build;
use std::iter::*;
use num_traits::{cast::FromPrimitive, float::Float};

pub mod plotter;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct SpectralDensity<T: Measurement> {
    pub peak: Option<SpectralDensitySample<T, T>>,
    pub noise_floor: T,
    pub audio_sampling_rate: T,
    pub freq_step: T,
    pub N: usize,
    pub seems_off: bool,
    pub all_match: bool,
    pub data: std::collections::LinkedList<SpectralDensitySample<T, T>>
}

impl<T: Measurement> SpectralDensity<T> {
    #[allow(non_snake_case)]
    pub fn new<U: crate::math::Sample>(data: &[U], audio_sampling_rate: T, N: usize) -> Self {
        let mut noise_total: Option<T> = None;
        let mut first_sample: Option<SpectralDensitySample<T, T>> = None;
        let mut second_sample: Option<SpectralDensitySample<T, T>> = None;
        let mut peak_sample: Option<SpectralDensitySample<T, T>> = None;
        let mut seems_off: bool = true;
        let mut all_match: bool = true;
        let total_freqs: usize = data.len();
        let spectral_densities: std::collections::LinkedList<SpectralDensitySample<T, T>> = Self::get_welch_spectral_density::<U>(data, audio_sampling_rate, N).iter().map(|(freq, sd)| {

            let sample = SpectralDensitySample::<T, T>::new(*freq, *sd);

            if first_sample.is_none() {
                peak_sample = Some(sample);
                first_sample = Some(sample);
            }
            else if second_sample.is_none() {
                second_sample = Some(sample);
            }

            if sample.spectral_density() >= (peak_sample.unwrap()).spectral_density() { // >= used to ensure first_sample and peak won't match if they have the same sd
                peak_sample = Some(sample);
            }

            match noise_total {
                Some(noise) => noise_total = Some(noise + sample.spectral_density()),
                None => noise_total = Some(sample.spectral_density()),
            };

            return sample

        }).collect();
        
        let freq_step = if spectral_densities.len() > 1 { second_sample.unwrap().frequency() } else { T::from_usize(0).unwrap() };

        let noise_floor = match noise_total {
            Some(noise) => noise / (T::from_usize(total_freqs).unwrap()),
            None => panic!("No samples to generate noise total")
        };

        if peak_sample.is_some() {
            if N < audio_sampling_rate.to_usize().unwrap() / 2 {
                let peak = peak_sample.unwrap();
                let first = first_sample.unwrap();
                if peak.frequency() == first.frequency() || peak.spectral_density() != peak.spectral_density() {
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
            audio_sampling_rate: audio_sampling_rate,
            freq_step: freq_step,
            N: N,
            data: spectral_densities,
            all_match: all_match,
            seems_off: seems_off
         }
    
    }
    

    /// Returns [Welch] [Builder] given the `signal` sampled at `fs`Hz
    #[allow(non_snake_case)]
    pub fn get_welch_spectral_density<U: crate::math::Sample>(data: &[U], audio_sampling_rate: T, N: usize) -> Vec<(T, T)> {
        let measure_data: Vec<T> = data.iter().map(|x| T::from_f64(x.to_f64().unwrap()).unwrap()).collect();
        let builder = welch_sde::SpectralDensity::<T>::builder(&measure_data,audio_sampling_rate).n_segment(N);
        let welch_spectral_density: welch_sde::SpectralDensity<T> = builder.build();
        let periodogram: welch_sde::Periodogram<T> = welch_spectral_density.periodogram();
        let periodogram_map: Vec<(T, T)> = zip(periodogram.frequency(), &(*periodogram)).map(|(x, &y)| (x, y)).collect();
        return periodogram_map;
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
pub struct SpectralDensitySample<T: Measurement, U: Measurement>(pub T, pub U);

impl<T: Measurement, U: Measurement> SpectralDensitySample<T, U> {
    pub fn frequency(&self) -> T {
        let Self(freq, sd) = self;
        return *freq;
    }

    pub fn spectral_density(&self) -> U {
        let Self(freq, sd) = self;
        return *sd;
    }

    pub fn spectral_density_db(&self) -> U {
        return U::from_usize(10).unwrap() * self.1.log10();
    }

    pub fn new(freq: T, sd: U) -> Self {
        Self(freq, sd)
    }
}

pub trait Measurement:
    welch_sde::Signal + ::num_traits::float::Float + std::iter::Sum + std::ops::SubAssign + std::ops::AddAssign + ::num_traits::cast::FromPrimitive + Default
{
}
impl Measurement for f64 {}
impl Measurement for f32 {}

