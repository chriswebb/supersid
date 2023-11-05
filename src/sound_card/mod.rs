pub mod config;
pub mod alsa;

pub trait SoundCard<T: Sample> {
    fn new(config: config::SoundCardConfig) -> Self;
    fn config(&self) -> config::SoundCardConfig;
    fn create_player(&self, channels: usize) -> Box<dyn SoundCardPlayer<T>>;
    fn create_recorder(&self, channels: usize) -> Box<dyn SoundCardRecorder<T>>;
}

pub trait SoundCardPlayer<T: Sample> {
    fn wait_for_finish(&mut self) -> Result<(), std::io::Error>;
    fn play(&mut self, data: &[ChannelData<T>]) -> Result<(), std::io::Error>;
}

pub trait SoundCardRecorder<T: Sample> {
    fn record(&mut self, milliseconds: usize) -> Result<Vec<ChannelData<T>>, std::io::Error>;
}

pub trait Sample:
    ::alsa::pcm::IoFormat + welch_sde::Signal + num_traits::Float + std::iter::Sum + std::ops::SubAssign + std::ops::AddAssign + num_traits::cast::FromPrimitive + Default + Sized
{
}
impl Sample for f64 {}
impl Sample for f32 {}



#[derive(Debug, Clone)]
pub struct ChannelData<T: Sample> {
    pub channel_num: usize,
    pub channel_data: Vec<T>,
    pub record_duration: Option<std::time::Duration>
}

impl<T: Sample> ChannelData<T> {
    pub fn new(channel_num: usize, channel_data: Vec<T>) -> Self {
        Self {
            channel_num: channel_num,
            channel_data: channel_data,
            record_duration: None
        }
    }
}