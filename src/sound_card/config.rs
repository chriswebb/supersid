pub struct SoundCardConfig<'a> {
    pub device_id: &'a str,
    pub format: self::Format,
    pub sampling_rate: usize,
}

impl<'a> SoundCardConfig<'a> {
    pub fn new(device_id: &'a str, format: self::Format, sampling_rate: usize) -> Self {
        Self {
            device_id: device_id,
            format: format,
            sampling_rate: sampling_rate
        }
    }
    
}

pub enum Format {
    B16,
    B24,
    B32,
}

pub enum SamplingRates {

}