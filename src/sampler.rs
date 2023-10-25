use welch_sde::Build;
use std::iter::*;
use num_traits::{cast::FromPrimitive, float::Float};

use libc;
use std::{mem, panic};
use std::os::unix::io::RawFd;
use alsa::{pcm, PollDescriptors, Direction, ValueOr};
use std::ffi::CString;
use alsa::pcm::*;
mod spectral_density;


// Transform Vec<Signal> to Vec<SpectralDensitySample<Frequency, SpectralDensityMeasure>>

#[derive(Debug)]
pub struct SpectralDensity<T: Measurement> {
    pub peak: Option<SpectralDensitySample<T, T>>,
    pub noise_floor: T,
    pub seems_off: bool,
    pub all_match: bool,
    data: std::collections::LinkedList<SpectralDensitySample<T, T>>
}

impl<T: Measurement> SpectralDensity<T> {
    pub fn new(data: Vec<T>, audio_sampling_rate: T, num_ffts: usize) -> Self {
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
    pub fn get_welch_spectral_density(data: Vec <T>, audio_sampling_rate: T, num_ffts: usize) -> Vec<(T, T)> {
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


impl<T: Measurement> Iterator for SpectralDensity<T> {
    type Item = SpectralDensitySample<T, T>;
    
    fn next(&mut self) -> Option<Self::Item> {
        None

    }
}

fn pcm_to_fd(p: &pcm::PCM) -> Result<RawFd, alsa::Error> {
    let mut fds: [libc::pollfd; 1] = unsafe { mem::zeroed() };
    let c = PollDescriptors::fill(p, &mut fds)?;
    if c != 1 {
        return Err(alsa::Error::unsupported("snd_pcm_poll_descriptors returned wrong number of fds"))
    }
    Ok(fds[0].fd)
}

fn record_from_plughw_standard() -> Result<(), alsa::Error> {
    let pcm = PCM::open(&*CString::new("plughw:CARD=Device,DEV=0").unwrap(), Direction::Capture, false).unwrap();
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(2).unwrap();
    hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();
    pcm.start().unwrap();
    let mut buf = [0i16; 1024];
    assert_eq!(pcm.io_i16().unwrap().readi(&mut buf).unwrap(), 1024/2);
    Ok(())
}


fn record_from_plughw_mmap() -> Result<(), alsa::Error> {
    
    use std::{thread, time};
    use alsa::direct::pcm::SyncPtrStatus;

    let pcm = PCM::open(&*CString::new("plughw:CARD=Device,DEV=0").unwrap(), Direction::Capture, false).unwrap();
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(1).unwrap();
    hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::MMapInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();

    let ss = unsafe { SyncPtrStatus::sync_ptr(pcm_to_fd(&pcm).unwrap(), false, None, None).unwrap() };
    assert_eq!(ss.state(), State::Prepared);

    let mut m = pcm.direct_mmap_capture::<i16>().unwrap();

    assert_eq!(m.status().state(), State::Prepared);
    assert_eq!(m.appl_ptr(), 0);
    assert_eq!(m.hw_ptr(), 0);


    println!("{:?}", m);

    let now = time::Instant::now();
    pcm.start().unwrap();
    while m.avail() < 256 { thread::sleep(time::Duration::from_millis(1)) };
    assert!(now.elapsed() >= time::Duration::from_millis(256 * 1000 / 44100));
    let (ptr1, md) = m.data_ptr();
    assert_eq!(ptr1.channels, 2);
    assert!(ptr1.frames >= 256);
    assert!(md.is_none());
    println!("Has {:?} frames at {:?} in {:?}", m.avail(), ptr1.ptr, now.elapsed());
    let samples: Vec<i16> = m.iter().collect();
    assert!(samples.len() >= ptr1.frames as usize * 2);
    println!("Collected {} samples", samples.len());
    let (ptr2, _md) = m.data_ptr();
    assert!(unsafe { ptr1.ptr.offset(256 * 2) } <= ptr2.ptr);
    Ok(())
}


fn main() {
    let standard_result = panic::catch_unwind(|| -> &'static str { 
        let ret = record_from_plughw_standard(); 
        if ret.is_err() {
            return "failed";
        }
        return "passed";
    });
    match standard_result {
        Ok(result_str) => println!("Recording using standard sample access {}.", result_str),
        Err(_error) => println!("Recording using standard sample access failed with panic."),
    };

    let direct_result = panic::catch_unwind(|| -> &'static str { 
        let ret = record_from_plughw_mmap(); 
        if ret.is_err() {
            return "failed";
        }
        return "passed";
    });
    
    match direct_result {
        Ok(result_str) => println!("Recording using direct sample access {}.", result_str),
        Err(_error) => println!("Recording using direct sample access failed with panic."),
    };
}
