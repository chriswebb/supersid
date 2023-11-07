#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SoundCardConfig {
    pub device_id: String,
    pub format: self::Format,
    pub sampling_rate: self::SamplingRate,
    pub period_size: usize
}

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
    
    pub const SAMPLING_RATE_44100: usize = 44100;
    pub const SAMPLING_RATE_48000: usize = 48000;
    pub const SAMPLING_RATE_96000: usize = 96000;
    pub const SAMPLING_RATE_192000: usize = 192000;
    
    // pub const DOUBLE_SAMPLING_RATE_44100: usize = 44100 * 2;
    // pub const DOUBLE_SAMPLING_RATE_48000: usize = 48000 * 2;
    // pub const DOUBLE_SAMPLING_RATE_96000: usize = 96000 * 2;
    // pub const DOUBLE_SAMPLING_RATE_192000: usize = 192000 * 2;


    pub fn sample_value<T: crate::math::Sample>(&self) -> T {    
        T::from_usize(self.value()).unwrap()
    }

    pub fn value(&self) -> usize {
        match self {
            Self::Hz48000 => Self::SAMPLING_RATE_48000,
            Self::Hz96000 => Self::SAMPLING_RATE_96000,
            Self::Hz192000 => Self::SAMPLING_RATE_192000,
            Self::Hz44100 => Self::SAMPLING_RATE_44100
        }
    }

    pub fn label(value: usize) -> Self {
        match value {
            Self::SAMPLING_RATE_48000 => Self::Hz48000,
            Self::SAMPLING_RATE_96000 => Self::Hz96000,
            Self::SAMPLING_RATE_192000 => Self::Hz192000,
            _ => Self::Hz44100
        }
    }
}