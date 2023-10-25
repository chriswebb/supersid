pub mod config;
pub mod alsa;

pub trait SoundCard<'a, T: Sample, U: SoundCardPlayer<'a, T>, V: SoundCardRecorder<'a, T>> {
    fn new(config: &'a self::config::SoundCardConfig<'a>) -> Self;
    fn setup_player(&self, channels: u8) -> Result<U, std::io::Error>;
    fn setup_recorder(&self, channels: u8, period_size: usize) -> Result<V, std::io::Error>;
}

pub trait SoundCardRecorder<'a, T: Sample> {
    fn record(&self) -> Result<Vec<T>, std::io::Error>;
}

pub trait SoundCardPlayer<'a, T: Sample> {
    fn play(&self, data: Vec<T>) -> Result<(), std::io::Error>;
}

pub trait Sample:
    welch_sde::Signal + num_traits::Float + std::iter::Sum + std::ops::SubAssign + std::ops::AddAssign + num_traits::cast::FromPrimitive
{
}
impl Sample for f64 {}
impl Sample for f32 {}

