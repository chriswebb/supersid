use num_traits::ToPrimitive;

pub struct AlsaSoundCard<'a> {
    pub config: &'a super::config::SoundCardConfig<'a>,
    internal_device_id: &'a std::ffi::CString
}


impl<'a> AlsaSoundCard<'a> {
    pub fn get_format(format: super::config::Format) -> alsa::pcm::Format {
        match format {
            B16 => alsa::pcm::Format::s16(),
            B24 => alsa::pcm::Format::s24_3(),
            B32 => alsa::pcm::Format::s32()
        }
    }
    fn get_current_format(&self) -> alsa::pcm::Format {
        AlsaSoundCard::get_format(self.config.format)
    }
}

impl<'a, T: super::Sample> super::SoundCard<'a, T, AlsaPlayer<'a>, AlsaRecorder<'a>> for AlsaSoundCard<'a> {
    fn new(config: &'a super::config::SoundCardConfig<'a>) -> Self {
        Self {
            config: config,
            internal_device_id: &std::ffi::CString::new((*config).device_id).unwrap()
        }
    }
    fn setup_recorder(&self, channels: u8, period_size: usize) -> Result<AlsaRecorder<'a>, std::io::Error> {
        let pcm = alsa::pcm::PCM::open(self.internal_device_id, alsa::Direction::Capture, false).unwrap();
        let hwp = alsa::pcm::HwParams::any(&pcm).unwrap();
        hwp.set_channels(channels.to_u32().unwrap()).unwrap();
        hwp.set_rate(44100, alsa::ValueOr::Nearest).unwrap();
        hwp.set_format(self.get_current_format()).unwrap();
        hwp.set_access(alsa::pcm::Access::RWInterleaved).unwrap();
        pcm.hw_params(&hwp).unwrap();
        pcm.start().unwrap();
    }
    fn setup_player(&self, channels: u8, data: Vec<T>) -> Result<AlsaPlayer<'a>, std::io::Error> {

    }
}









pub struct AlsaPlayer<'a> {
    pub sound_card: &'a AlsaSoundCard<'a>,
    pub channels: u8
}

impl<'a> AlsaPlayer<'a> {
    fn new(sound_card: &'a AlsaSoundCard<'a>, channels: u8) -> Self {
        Self {
            sound_card: sound_card,
            channels: channels
        }
    }
}

impl<'a, T: super::Sample> super::SoundCardPlayer<'a, T> for AlsaPlayer<'a> {
    fn play(&self) -> Result<Vec<T>, std::io::Error> {

    }
}

pub struct AlsaRecorder<'a> {
    pub sound_card: &'a AlsaSoundCard<'a>,
    pub channels: u8
}

impl<'a> AlsaRecorder<'a> {
    fn new(sound_card: &'a AlsaSoundCard<'a>, channels: u8, period_size: usize) -> Self {
        Self {
            sound_card: sound_card,
            channels: channels
        }
    }
}

impl<'a, T: super::Sample> super::SoundCardRecorder<'a, T> for AlsaRecorder<'a> {
    fn record(&self) -> Result<Vec<T>, std::io::Error> {

    }
}