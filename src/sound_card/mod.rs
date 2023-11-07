pub mod config;
pub mod alsa;

pub trait SoundCard<T: crate::math::Sample> {
    fn new(config: config::SoundCardConfig) -> Self;
    fn config(&self) -> config::SoundCardConfig;
}

pub trait SoundCardPlayer<T: crate::math::Sample> {
    fn wait_for_finish(&mut self) -> Result<(), std::io::Error>;
    fn play(&mut self, data: &[ChannelData<T>]) -> Result<(), std::io::Error>;
}

pub trait SoundCardRecorder<T: crate::math::Sample> {
    fn record(&mut self, milliseconds: usize) -> Result<Vec<ChannelData<T>>, std::io::Error>;
}



#[derive(Debug, Clone)]
pub struct ChannelData<T: crate::math::Sample> {
    pub channel_num: usize,
    pub channel_data: Vec<T>,
    pub record_duration: Option<std::time::Duration>
}

impl<T: crate::math::Sample> ChannelData<T> {
    pub fn new(channel_num: usize, channel_data: Vec<T>) -> Self {
        Self {
            channel_num: channel_num,
            channel_data: channel_data,
            record_duration: None
        }
    }
}