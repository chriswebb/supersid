use num_traits::ToPrimitive;

pub trait AlsaSoundCardLink {
    fn link(&mut self, other: &mut (dyn AlsaSoundCardLink)) -> Result<(), std::io::Error>;
    fn get_pcm<'a>(&'a mut self) -> &'a ::alsa::pcm::PCM;
}


#[derive(Clone)]
pub struct AlsaSoundCard<T: super::Sample> {
    pub config: super::config::SoundCardConfig,
    phantom: std::marker::PhantomData<T>
}


impl<T: super::Sample> AlsaSoundCard<T> {

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

        match hwp.set_buffer_size(16384) {
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
        Box::new(AlsaPlayer::<T>::new(self.clone(), channels))
    }

    fn create_recorder(&self, channels: usize) -> Box<dyn super::SoundCardRecorder<T>> {
        Box::new(AlsaRecorder::<T>::new(self.clone(), channels))
    }
    
}



pub struct AlsaPlayer<T: super::Sample> {
    pub sound_card: AlsaSoundCard<T>,
    pub channels: usize,
    alsa_pcm: ::alsa::pcm::PCM
}

impl<T: super::Sample> AlsaPlayer<T> {
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

impl<T: super::Sample> AlsaSoundCardLink for AlsaPlayer<T> {
    fn link(&mut self, other: &mut (dyn AlsaSoundCardLink)) -> Result<(), std::io::Error> {
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
        let num_frames = data.len() / self.channels;

        let pcm_io;

        match self.alsa_pcm.io_checked::<T>() {
            Ok(io) => pcm_io = io,
            Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
        };

        match pcm_io.writei(&interleaved_data) {
            Ok(frames_written) => assert_eq!(frames_written, num_frames),
            Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
        };

        Ok(())
    }
}


pub struct AlsaRecorder<T: super::Sample> {
    pub sound_card: AlsaSoundCard<T>,
    pub channels: usize,
    alsa_pcm: ::alsa::pcm::PCM
}


impl<T: super::Sample> AlsaRecorder<T> {
    fn new(sound_card: AlsaSoundCard<T>, channels: usize) -> Self {
        let data;
        match ::alsa::pcm::PCM::new(&(sound_card.config.device_id.as_str()), ::alsa::Direction::Capture, false) {
            Ok(pcm) => { data = AlsaRecorder::<T> {
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
    fn link(&mut self, other: &mut (dyn AlsaSoundCardLink)) -> Result<(), std::io::Error> {
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

        let pcm_io;

        match self.alsa_pcm.io_checked::<i32>() {
            Ok(io) => pcm_io = io,
            Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
        };

        let mut buffer= [0i32; 2048];
        let mut total_frames_read: usize = 0;
        let mut i: usize = 0;
        let mut data = Vec::<super::ChannelData<T>>::with_capacity(self.channels);

        while i < self.channels {
            data.push(super::ChannelData::<T>::new(i + 1, Vec::<T>::with_capacity(num_frames)));
            i += 1;
        }

        let record_start = std::time::Instant::now();
        while total_frames_read < num_frames {
            match pcm_io.readi(&mut buffer) {
                Ok(frames_read) => {
                    if frames_read > 0 {
                        i = 0;
                        while i < frames_read {
                            let mut j = 0;
                            while j < self.channels {
                                data[j].channel_data.push(T::from(buffer[i+j]).unwrap());
                                j += 1;
                            }
                            i += 1;
                        }

                        total_frames_read += frames_read;
                    }
                },
                Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
            };
            //println!("frames read {} of {} total.", total_frames_read, num_frames);
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