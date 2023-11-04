use crate::spectral_density::{Measurement, self};

#[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct SuperSidConfig {
    pub monitor_id: String,
    pub site: SuperSidSite,
    pub sound_card: crate::sound_card::config::SoundCardConfig,
    pub stations: Vec<StationConfig>,

    


}

impl SuperSidConfig {

    
}


#[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct SuperSidSite {
    pub site_name: String,
    pub site_contact_email: String,
    pub site_latitude: f64,
    pub site_longitude: f64,

}


impl SuperSidSite {
    pub fn new(site_name: String, site_contact_email: String, site_latitude: f64, site_longitude: f64) -> Self {
        Self { site_name: site_name,
            site_contact_email: site_contact_email,
            site_latitude: site_latitude,
            site_longitude: site_longitude
        }
    }
}

#[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct StationConfig{
    pub callsign: String,
    pub color: char,
    pub frequency: usize
}

impl StationConfig{
    pub fn new(callsign: &str, color: char, frequency: usize) -> Self {
        Self {
            callsign: callsign.to_string(),
            color: color,
            frequency: frequency
        }
    }
    
    #[allow(non_snake_case)]
    pub fn get_bin<T: crate::spectral_density::Measurement>(&self, freq_per_step: T) -> usize {
        let freq_t = T::from(self.frequency).unwrap();
        (freq_t / freq_per_step).to_usize().unwrap() + if freq_t % freq_per_step > freq_per_step / T::from(2).unwrap() { 1 } else { 0 }
    }
}