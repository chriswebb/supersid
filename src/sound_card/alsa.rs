use num_traits::ToPrimitive;

pub trait AlsaSoundCardLink {
    fn link<'a, T: AlsaSoundCardLink>(&'a mut self, other: &'a mut T) -> Result<(), std::io::Error>;
    fn get_pcm<'a>(&'a mut self) -> &'a ::alsa::pcm::PCM;
}


#[derive(Clone)]
pub struct AlsaSoundCard<T: super::Sample> {
    pub config: super::config::SoundCardConfig,
    phantom: std::marker::PhantomData<T>
}


impl<T: super::Sample> AlsaSoundCard<T> {

    
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
        let channels_u32: u32 = channels.to_u32().unwrap();

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

        match hwp.set_period_size(self.config.period_size.to_i64().unwrap(), alsa::ValueOr::Nearest) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        match hwp.set_buffer_size(super::config::BUFFER_LENGTH.to_i64().unwrap()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };


        match hwp.set_rate(self.config.sampling_rate.value().to_u32().unwrap(), alsa::ValueOr::Nearest) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };


        match hwp.set_access(alsa::pcm::Access::RWInterleaved) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        match pcm.hw_params(&hwp) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        Ok(hwp)
    }

}

impl<T: super::Sample> super::SoundCard<T> for AlsaSoundCard<T> {
    fn new(config: super::config::SoundCardConfig) -> Self {
        Self {
            config: config,
            phantom: std::marker::PhantomData
        }
    }

    fn config(&self) -> super::config::SoundCardConfig {
        self.config.clone()
    }

    
    fn create_player(&self, channels: usize) -> Box<dyn super::SoundCardPlayer<T>> {
        Box::new(self.create_alsa_player(channels))
    }

    fn create_recorder(&self, channels: usize) -> Box<dyn super::SoundCardRecorder<T>> {
        Box::new(self.create_alsa_recorder(channels))
    }
    
}



pub struct AlsaPlayer<T: super::Sample> {
    pub sound_card: AlsaSoundCard<T>,
    pub channels: usize,
    alsa_pcm: ::alsa::pcm::PCM
}

impl<T: super::Sample> AlsaPlayer<T> {
    pub fn new(sound_card: AlsaSoundCard<T>, channels: usize) -> Self {
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

impl<T: super::Sample> AlsaSoundCardLink for AlsaPlayer<T> {
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


impl<T: super::Sample> super::SoundCardPlayer<T> for AlsaPlayer<T> {

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
    
        let mut interleaved_data: Vec<i32> = Vec::<i32>::with_capacity(min_length * self.channels);
        let mut j: usize;
        i = 0;
        while i < min_length {
            j = 0;
            while j < data.len() {
                interleaved_data.push(data[j].channel_data[i].to_i32().unwrap());
                j += 1;
            }
            i += 1;
        }
        let num_frames = interleaved_data.len() / self.channels;

        let pcm_io;

        match self.alsa_pcm.io_checked::<i32>() {
            Ok(io) => pcm_io = io,
            Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
        };

        let mut total_frames_written: usize = 0;
        let buffer_length = 16384;
        while (total_frames_written < interleaved_data.len()) {
            match pcm_io.writei(&interleaved_data[total_frames_written*self.channels..std::cmp::min(interleaved_data.len(), (total_frames_written+buffer_length)*self.channels)]) {
                Ok(frames_written) => total_frames_written += frames_written,
                Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
            };
        }

        Ok(())
    }
}


pub struct AlsaRecorder<T: super::Sample> {
    pub sound_card: AlsaSoundCard<T>,
    pub channels: usize,
    alsa_pcm: ::alsa::pcm::PCM,
    buffer_vec: Vec<[i32; super::config::CHAN_BUFFER_LENGTH]>
}


impl<T: super::Sample> AlsaRecorder<T> {
    pub fn new(sound_card: AlsaSoundCard<T>, channels: usize) -> Self {
        let data;
        match ::alsa::pcm::PCM::new(&(sound_card.config.device_id.as_str()), ::alsa::Direction::Capture, false) {
            Ok(pcm) => { data = AlsaRecorder::<T> {
                                sound_card: sound_card,
                                channels: channels,
                                alsa_pcm: pcm,
                                buffer_vec: Vec::<[i32; super::config::CHAN_BUFFER_LENGTH]>::new()
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
                                
                                match data.alsa_pcm.start() {
                                    Ok(_) => (),
                                    Err(error) => panic!("Could not start PCM capture device '{}': {}", data.sound_card.config.device_id, error)
                                };
                            }
                            return data;
            },
            Err(error) => panic!("Could not initialize PCM capture device '{}': {}", sound_card.config.device_id, error)
        }
    }
}

impl<T: super::Sample> AlsaSoundCardLink for AlsaRecorder<T> {
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

impl<T: super::Sample> super::SoundCardRecorder<T> for AlsaRecorder<T> {

    fn record(&mut self, milliseconds: usize) -> Result<Vec<super::ChannelData<T>>, std::io::Error> {

        let num_frames = self.sound_card.config.sampling_rate.value() * milliseconds / 1000;
        let buffer_multiple = num_frames.div_ceil(super::config::BUFFER_LENGTH);

        if (self.buffer_vec.len() != buffer_multiple) {
            if self.buffer_vec.len() > buffer_multiple {
                self.buffer_vec.truncate(buffer_multiple )
            }
            else if self.buffer_vec.len() < buffer_multiple {
                self.buffer_vec.resize_with(buffer_multiple, || -> [i32; super::config::CHAN_BUFFER_LENGTH] {
                    [0i32; super::config::CHAN_BUFFER_LENGTH]
                });
            }
        }

        let pcm_io;

        match self.alsa_pcm.io_checked::<i32>() {
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

        let record_start = std::time::Instant::now();
        let mut current_buffer_number: usize = 0;
    
        while total_frames_read < num_frames {
            match pcm_io.readi(&mut self.buffer_vec[current_buffer_number]) {
                Ok(frames_read) => total_frames_read += frames_read,
                Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
            };
            current_buffer_number += 1;
            //println!("frames read {} of {} total.", total_frames_read, num_frames);
        }

        {
            current_buffer_number = 0;
            let mut j: usize;
            while current_buffer_number < buffer_multiple {
                i = 0;
                while i < super::config::BUFFER_LENGTH {
                    j = 0;
                    while j < self.channels {
                        data[j].channel_data.push(T::from(self.buffer_vec[current_buffer_number][i+j]).unwrap());
                        j += 1;
                    }
                    i += 1;

                }
                current_buffer_number += 1;
            }
        }

        let record_duration = record_start.elapsed();

        i = 0;
        while i < self.channels {
            data[i].record_duration = Some(record_duration);
            if data[i].channel_data.len() > num_frames {
                data[i].channel_data.truncate(num_frames);
            }
            i += 1;
        }

        Ok(data)
    }
}