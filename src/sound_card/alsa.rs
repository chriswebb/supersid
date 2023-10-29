use num_traits::ToPrimitive;

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
            super::config::Format::B16 => alsa::pcm::Format::s16(),
            super::config::Format::B24 => alsa::pcm::Format::s24_3(),
            super::config::Format::B32 => alsa::pcm::Format::s32()
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

        match hwp.set_rate(self.config.sampling_rate.value().to_u32().unwrap(), alsa::ValueOr::Nearest) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        match hwp.set_period_size(self.config.period_size.to_i64().unwrap(), alsa::ValueOr::Nearest) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        match hwp.set_format(self.get_current_format()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        match hwp.set_access(alsa::pcm::Access::RWInterleaved) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        Ok(hwp)
    }

    fn internal_play(&self, pcm: &::alsa::pcm::PCM, channels: usize, data: &[super::ChannelData<T>]) -> Result<(), ::alsa::Error> {

        // Interleave Audio
        let mut i: usize = 0;
        if data.len() < 1 {
            return Ok(());
        }

        if channels != data.len() {
            panic!("Data (len: {}) does not contain enough channels ({}).", data.len(), channels);
        }

        let mut min_length: usize = usize::MAX;
        
        while i < data.len() {
            min_length = std::cmp::min(min_length, data[i].channel_data.len());
            i += 1;
        }
    
        let mut interleaved_data: Vec<T> = Vec::<T>::with_capacity(min_length * channels);
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
        let num_frames = data.len() / channels;

        let pcm_io;

        match pcm.io_checked::<T>() {
            Ok(io) => pcm_io = io,
            Err(error) => return Err(error)
        };

        match pcm_io.writei(&interleaved_data) {
            Ok(frames_written) => assert_eq!(frames_written, num_frames),
            Err(error) => return Err(error)
        }

        Ok(())
    }

}

impl<T: super::Sample> super::SoundCard<T> for AlsaSoundCard<T> {
    fn new(config: super::config::SoundCardConfig) -> Self {
        Self {
            config: config,
            phantom: std::marker::PhantomData
        }
    }


    fn play(&self, channels: usize, data: &[super::ChannelData<T>]) -> Result<(), std::io::Error> {
        
        let pcm: ::alsa::pcm::PCM;
        let hwp: ::alsa::pcm::HwParams;

        let result = ::alsa::pcm::PCM::new(&(self.config.device_id.as_str()), alsa::Direction::Playback, true);
        match result {
            Ok(res) => pcm = res,
            Err(error) => return Err(Self::get_std_error(error))
        };

        match self.setup_hardware(&pcm, channels) {
            Ok(hw_params) => hwp = hw_params,
            Err(error) => return Err(Self::get_std_error(error))
        };
        match pcm.hw_params(&hwp) {
            Ok(_) => (),
            Err(error) => return Err(Self::get_std_error(error))
        };

        match pcm.start() {
            Ok(_) => (),
            Err(error) => return Err(Self::get_std_error(error))
        };

        match self.internal_play(&pcm, channels, data) {
            Ok(_) => Ok(()),
            Err(error) => Err(Self::get_std_error(error))
        }
    }

    
    fn play_and_wait(&self, channels: usize, data: &[super::ChannelData<T>]) -> Result<(), std::io::Error> {
        
        let pcm: ::alsa::pcm::PCM;
        let hwp: ::alsa::pcm::HwParams;

        let result = ::alsa::pcm::PCM::new(&(self.config.device_id.as_str()), alsa::Direction::Playback, true);
        match result {
            Ok(res) => pcm = res,
            Err(error) => return Err(Self::get_std_error(error))
        };

        match self.setup_hardware(&pcm, channels) {
            Ok(hw_params) => hwp = hw_params,
            Err(error) => return Err(Self::get_std_error(error))
        };
        match pcm.hw_params(&hwp) {
            Ok(_) => (),
            Err(error) => return Err(Self::get_std_error(error))
        };

        match pcm.start() {
            Ok(_) => (),
            Err(error) => return Err(Self::get_std_error(error))
        };

        match self.internal_play(&pcm, channels, data) {
            Ok(_) => (),
            Err(error) => return Err(Self::get_std_error(error))
        };


        match pcm.drain() {
            Ok(()) => Ok(()),
            Err(error) => Err(AlsaSoundCard::<T>::get_std_error(error))
        }
    }


    fn record(&self, channels: usize, milliseconds: usize) -> Result<Vec<super::ChannelData<T>>, std::io::Error> {
        
        let pcm: ::alsa::pcm::PCM;
        let hwp: ::alsa::pcm::HwParams;

        let result = ::alsa::pcm::PCM::new(&(self.config.device_id.as_str()), alsa::Direction::Capture, false);
        match result {
            Ok(res_pcm) => pcm = res_pcm,
            Err(error) => return Err(Self::get_std_error(error))
        };

        match self.setup_hardware(&pcm, channels) {
            Ok(hw_params) => hwp = hw_params,
            Err(error) => return Err(Self::get_std_error(error))
        };

        match pcm.hw_params(&hwp) {
            Ok(_) => (),
            Err(error) => return Err(Self::get_std_error(error))
        };

        match pcm.start() {
            Ok(_) => (),
            Err(error) => return Err(Self::get_std_error(error))
        };

        let num_frames = self.config.sampling_rate.value() * milliseconds / 1000;

        let pcm_io;

        match pcm.io_checked::<T>() {
            Ok(io) => pcm_io = io,
            Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
        };

        let mut buffer:[T; 1024] = [T::from_usize(0).unwrap(); 1024];
        let mut total_frames_read: usize = 0;
        let mut i: usize = 0;
        let mut data = Vec::<super::ChannelData<T>>::with_capacity(channels);
        while i < channels {
            data.push(super::ChannelData::<T>::new(i + 1, Vec::<T>::with_capacity(num_frames)));
            i += 1;
        }
        while total_frames_read < num_frames {
            match pcm_io.readi(&mut buffer) {
                Ok(frames_read) => {
                    if frames_read > 0 {
                        i = 0;
                        while i < frames_read {
                            let mut j = 0;
                            while i < channels {
                                data[j].channel_data.push(buffer[i+j]);
                                j += 1;
                            }
                            i += 1;
                        }

                        total_frames_read += frames_read;
                    }
                },
                Err(error) => return Err(AlsaSoundCard::<T>::get_std_error(error))
            };
        }

        i = 0;
        while i < channels {

            if data[i].channel_data.len() > num_frames {
                data[i].channel_data.truncate(num_frames);
            }
        }

        Ok(data)

    }

    
}


