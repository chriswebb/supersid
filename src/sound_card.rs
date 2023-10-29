pub mod config;
pub mod alsa;

pub trait SoundCard<T: Sample> {
    fn new(config: config::SoundCardConfig) -> Self;
    fn record(&self, channels: usize, milliseconds: usize) -> Result<Vec<ChannelData<T>>, std::io::Error>;
    fn play_and_wait(&self, channels: usize, data: &[ChannelData<T>]) -> Result<(), std::io::Error>;
    fn play(&self, channels: usize, data: &[ChannelData<T>]) -> Result<(), std::io::Error>;
}

pub trait Sample:
    ::alsa::pcm::IoFormat + welch_sde::Signal + num_traits::Float + std::iter::Sum + std::ops::SubAssign + std::ops::AddAssign + num_traits::cast::FromPrimitive
{
}
impl Sample for f64 {}
impl Sample for f32 {}



#[derive(Debug, Clone)]
pub struct ChannelData<T: Sample> {
    pub channel_num: usize,
    pub channel_data: Vec<T>
}

impl<T: Sample> ChannelData<T> {
    pub fn new(channel_num: usize, channel_data: Vec<T>) -> Self {
        Self {
            channel_num: channel_num,
            channel_data: channel_data
        }
    }
}