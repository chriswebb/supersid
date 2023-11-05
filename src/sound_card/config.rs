
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SoundCardConfig {
    pub device_id: String,
    pub format: self::Format,
    pub sampling_rate: self::SamplingRate,
    pub period_size: usize
}

pub const BUFFER_LENGTH: usize = 16384;
pub const CHAN_BUFFER_LENGTH: usize = 32768;

impl SoundCardConfig {
    pub fn new(device_id: &str, format: self::Format, sampling_rate: self::SamplingRate, period_size: usize) -> Self {
        Self {
            device_id: device_id.to_string(),
            format: format,
            sampling_rate: sampling_rate,
            period_size: period_size
        }
    }

}


#[derive(Debug, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub enum Format {
    B16,
    B24,
    B32,
}

impl Format {
    
    pub fn get_bytes(&self) -> usize {
        match self {
            Format::B16 => 2,
            Format::B24 => 3,
            Format::B32 => 4
        }
    }
}

#[derive(Debug, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub enum SamplingRate {
    Hz44100,
    Hz48000,
    Hz96000,
    Hz192000
}

impl SamplingRate {
    
    pub fn sample_value<T: super::Sample>(&self) -> T {    
        T::from(self.value()).unwrap()
    }

    pub fn value(&self) -> usize {
        match self {
            Self::Hz44100 => 44100,
            Self::Hz48000 => 48000,
            Self::Hz96000 => 96000,
            Self::Hz192000 => 192000
        }
    }

    pub fn label(value: usize) -> Self {
        match value {
            44100 => Self::Hz44100,
            48000 => Self::Hz48000,
            96000 => Self::Hz96000,
            192000 => Self::Hz192000,
            _ => Self::Hz44100
        }
    }
}