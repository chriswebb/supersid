use num_traits::ToPrimitive;

pub trait AlsaSoundCardLink {
    fn link<'a, T: AlsaSoundCardLink>(&'a mut self, other: &'a mut T) -> Result<(), std::io::Error>;
    fn get_pcm<'a>(&'a mut self) -> &'a ::alsa::pcm::PCM;
}

impl alsa::pcm::IoFormat for crate::math::u24 {
    #[cfg(target_endian = "little")]
    const FORMAT: alsa::pcm::Format = alsa::pcm::Format::U243LE;
    #[cfg(target_endian = "big")]
    const FORMAT: alsa::pcm::Format = alsa::pcm::Format::U243BE;
}

impl alsa::pcm::IoFormat for crate::math::i24 {
    #[cfg(target_endian = "little")]
    const FORMAT: alsa::pcm::Format = alsa::pcm::Format::S243LE;
    #[cfg(target_endian = "big")]
    const FORMAT: alsa::pcm::Format = alsa::pcm::Format::S243BE;
}

#[derive(Clone)]
pub struct AlsaSoundCard<T: crate::math::Sample + ::alsa::pcm::IoFormat > {
    pub config: super::config::SoundCardConfig,
    phantom: std::marker::PhantomData<T>
}




impl<T: crate::math::Sample + ::alsa::pcm::IoFormat> AlsaSoundCard<T> {

    
    pub fn create_alsa_player(&self, channels: usize) -> AlsaPlayer<T> {
        AlsaPlayer::<T>::new(self.clone(), channels)
    }

    pub fn create_alsa_recorder(&self, channels: usize) -> AlsaRecorder<T> {
        AlsaRecorder::<T>::new(self.clone(), channels)
    }

    pub fn get_std_error(error: ::alsa::Error) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, error)
    }

    pub fn get_format(format: super::config::Format) -> alsa::pcm::Format {
        match format {
            super::config::Format::B16 => alsa::pcm::Format::S16LE,
            super::config::Format::B24 => alsa::pcm::Format::S243LE,
            super::config::Format::B32 => alsa::pcm::Format::S32LE
        }
    }

    fn get_current_format(&self) -> alsa::pcm::Format {
        Self::get_format(self.config.format)
    }

    fn setup_hardware<'a>(&'a self, pcm: &'a ::alsa::pcm::PCM, channels: usize) -> Result<::alsa::pcm::HwParams, ::alsa::Error> {
        
        let hwp: ::alsa::pcm::HwParams;
        let channels_u32 = channels as u32;

        match alsa::pcm::HwParams::any(pcm) {
            Ok(hw_params) => hwp = hw_params,
            Err(error) => return Err(error)
        };

        match hwp.set_channels(channels_u32) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };
        
        match hwp.set_format(self.get_current_format()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        match hwp.set_rate(self.config.sampling_rate.value() as u32, alsa::ValueOr::Nearest) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        match hwp.set_access(alsa::pcm::Access::RWInterleaved) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        Ok(hwp)
    }

}

impl<T: crate::math::Sample + ::alsa::pcm::IoFormat> super::SoundCard<T> for AlsaSoundCard<T> {
    fn new(config: super::config::SoundCardConfig) -> Self {
        Self {
            config: config,
            phantom: std::marker::PhantomData
        }
    }

    fn config(&self) -> super::config::SoundCardConfig {
        self.config.clone()
    }
}



pub struct AlsaPlayer<T: crate::math::Sample + ::alsa::pcm::IoFormat> {
    pub sound_card: AlsaSoundCard<T>,
    pub channels: usize,
    alsa_pcm: ::alsa::pcm::PCM
}

impl<T: crate::math::Sample + ::alsa::pcm::IoFormat> AlsaPlayer<T> {
    fn new(sound_card: AlsaSoundCard<T>, channels: usize) -> Self {
        let data;
        match ::alsa::pcm::PCM::new(&(sound_card.config.device_id.as_str()), ::alsa::Direction::Playback, false) {
            Ok(pcm) => { data = AlsaPlayer::<T> {
                                sound_card: sound_card,
                                channels: channels,
                                alsa_pcm: pcm
                            };
                            {
                                let hwp;
                                match data.sound_card.setup_hardware(&data.alsa_pcm, data.channels) {
                                    Ok(hw_params) => hwp = hw_params,
                                    Err(error) => panic!("Could not setup hardware for PCM capture device '{}': {}", data.sound_card.config.device_id, error)
                                };
                        
                                match data.alsa_pcm.hw_params(&hwp) {
                                    Ok(_) => (),
                                    Err(error) => panic!("Could not set hardware for PCM capture device '{}': {}", data.sound_card.config.device_id, error)
                                };

                                let swp = &data.alsa_pcm.sw_params_current().unwrap();
                                swp.set_start_threshold(hwp.get_buffer_size().unwrap()).unwrap();
                                

                            }
                            return data;
            },
            Err(error) => panic!("Could not initialize PCM capture device '{}': {}", sound_card.config.device_id, error)
        }
    }
}

impl<T: crate::math::Sample + ::alsa::pcm::IoFormat> AlsaSoundCardLink for AlsaPlayer<T> {
    fn link<'a, U: AlsaSoundCardLink>(&'a mut self, other: &'a mut U) -> Result<(), std::io::Error> {
        match self.alsa_pcm.link(&other.get_pcm()) {
            Ok(_) => Ok(()),
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Error occurred while linking to other pcm."))
        }
    }

    fn get_pcm<'a>(&'a mut self) -> &'a alsa::pcm::PCM {
        return &self.alsa_pcm;
    }
}


impl<T: crate::math::Sample + ::alsa::pcm::IoFormat> super::SoundCardPlayer<T> for AlsaPlayer<T> {

    fn wait_for_finish(&mut self) -> Result<(), std::io::Error> {
        match self.alsa_pcm.drain() {
            Ok(()) => Ok(()),
            Err(error) => Err(AlsaSoundCard::<T>::get_std_error(error))
        }
    }

    fn play(&mut self, data: &[super::ChannelData<T>]) -> Result<(), std::io::Error> {

        let mut i: usize = 0;
        if data.len() < 1 {
            return Ok(());
        }

        if self.channels != data.len() {
            panic!("Data (len: {}) does not contain enough channels ({}).", data.len(), self.channels);
        }

        let mut min_length: usize = usize::MAX;
        
        while i < data.len() {
            min_length = std::cmp::min(min_length, data[i].channel_data.len());
            i += 1;
        }
    
        let mut interleaved_data: Vec<T> = Vec::<T>::with_capacity(min_length * self.channels);
        let mut j: usize;
        i = 0;
        while i < min_length {
            j = 0;
            while j < data.len() {
                interleaved_data.push(data[j].channel_data[i]);
                j += 1;
            }
            i += 1;
        }
        let num_frames = interleaved_data.len() / self.channels;

        let pcm_io;

        match self.alsa_pcm.io_checked::<T>() {
            Ok(io) => pcm_io = io,
            Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
        };


        let mut total_frames_written: usize = 0;
        let buffer_length = super::config::SamplingRate::SAMPLING_RATE_192000 / 50;
        while total_frames_written < interleaved_data.len() {
            match pcm_io.writei(&interleaved_data[total_frames_written*self.channels..std::cmp::min(interleaved_data.len(), (total_frames_written+buffer_length)*self.channels)]) {
                Ok(frames_written) => total_frames_written += frames_written,
                Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
            };
        }

        Ok(())
    }
}


pub struct AlsaRecorder<T: crate::math::Sample + ::alsa::pcm::IoFormat> {
    pub sound_card: AlsaSoundCard<T>,
    pub channels: usize,
    alsa_pcm: ::alsa::pcm::PCM,
    buffer: [T; super::config::SamplingRate::SAMPLING_RATE_192000 / 50]
}


impl<T: crate::math::Sample + ::alsa::pcm::IoFormat> AlsaRecorder<T> {
    fn new(sound_card: AlsaSoundCard<T>, channels: usize) -> Self {
        let data;
        match ::alsa::pcm::PCM::new(&(sound_card.config.device_id.as_str()), ::alsa::Direction::Capture, false) {
            Ok(pcm) => { data = AlsaRecorder::<T> {
                                sound_card: sound_card,
                                channels: channels,
                                alsa_pcm: pcm,
                                buffer: [T::default(); super::config::SamplingRate::SAMPLING_RATE_192000 / 50]
                            };
                            {
                                let sampling_rate_value = data.sound_card.config.sampling_rate.value();
                                let hwp;
                                match data.sound_card.setup_hardware(&data.alsa_pcm, data.channels) {
                                    Ok(hw_params) => hwp = hw_params,
                                    Err(error) => panic!("Could not setup hardware for PCM capture device '{}': {}", data.sound_card.config.device_id, error)
                                };

                                match hwp.set_period_size(data.sound_card.config.period_size as i64, alsa::ValueOr::Nearest) {
                                    Ok(_) => (),
                                    Err(error) => panic!("Could not set period size to '{}' for PCM capture device '{}': {}", data.sound_card.config.period_size as i64, data.sound_card.config.device_id, error)
                                };
                        
                                let buffer_size = data.sound_card.config.period_size as i64 * 8;
                                match hwp.set_buffer_size(buffer_size) {
                                    Ok(_) => (),
                                    Err(error) => panic!("Could not set buffer size to '{}' for PCM capture device '{}': {}", buffer_size, data.sound_card.config.device_id, error)
                                };
                        
                                match data.alsa_pcm.hw_params(&hwp) {
                                    Ok(_) => (),
                                    Err(error) => panic!("Could not set hardware for PCM capture device '{}': {}", data.sound_card.config.device_id, error)
                                };
                                
                                match data.alsa_pcm.start() {
                                    Ok(_) => (),
                                    Err(error) => panic!("Could not start PCM capture device '{}': {}", data.sound_card.config.device_id, error)
                                };
                                
                                match data.alsa_pcm.drop() {
                                    Ok(_) => (),
                                    Err(error) => panic!("Could not stop PCM capture device '{}' after starting: {}", data.sound_card.config.device_id, error)
                                };
                            }
                            return data;
            },
            Err(error) => panic!("Could not initialize PCM capture device '{}': {}", sound_card.config.device_id, error)
        }
    }
}

impl<T: crate::math::Sample + ::alsa::pcm::IoFormat> AlsaSoundCardLink for AlsaRecorder<T> {
    fn link<'a, U: AlsaSoundCardLink>(&'a mut self, other: &'a mut U) -> Result<(), std::io::Error> {
        match self.alsa_pcm.link(&other.get_pcm()) {
            Ok(_) => Ok(()),
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Error occurred while linking to other pcm."))
        }
    }

    fn get_pcm<'a>(&'a mut self) -> &'a alsa::pcm::PCM {
        return &self.alsa_pcm;
    }
}

impl<T: crate::math::Sample + ::alsa::pcm::IoFormat> super::SoundCardRecorder<T> for AlsaRecorder<T> {

    fn record(&mut self, milliseconds: usize) -> Result<Vec<super::ChannelData<T>>, std::io::Error> {

        let num_frames = self.sound_card.config.sampling_rate.value() * milliseconds / 1000;

        let pcm_io;

        match self.alsa_pcm.io_checked::<T>() {
            Ok(io) => pcm_io = io,
            Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
        };

        let mut total_frames_read: usize = 0;
        let mut data = Vec::<super::ChannelData<T>>::with_capacity(self.channels);
        let mut i: usize = 0;

        while i < self.channels {
            data.push(super::ChannelData::<T>::new(i + 1, Vec::<T>::with_capacity(num_frames)));
            i += 1;
        }
    
        match self.alsa_pcm.prepare() {
            Ok(_) => (),
            Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, format!("Could not prepare PCM capture device '{}': {}", self.sound_card.config.device_id, error)))
        };

        let mut j: usize;
        let buffer_len = self.buffer.len();
        while total_frames_read < num_frames {
            match pcm_io.readi(&mut self.buffer) {
                Ok(frames_read) => {
                    total_frames_read += frames_read;
                    i = 0;
                    while i < buffer_len / 2 {
                        j = 0;
                        while j < self.channels {
                            data[j].channel_data.push(self.buffer[(2*i)+j]);
                            j += 1;
                        }
                        i += 1;
                    }
                }
                Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
            };
        }

        match self.alsa_pcm.drop() {
            Ok(_) => (),
            Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, format!("Could not drop PCM capture device '{}'  after capture: {}", self.sound_card.config.device_id, error)))
        };

        Ok(data)
    }
}