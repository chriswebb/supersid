use num_traits::{{ToPrimitive, Num, int::PrimInt}};
use rand::prelude::*;
use rand_distr::StandardNormal;

pub fn generate_tone<T: crate::sound_card::Sample>(freq: T, sample_freq: T, sample_size: usize, amplification: T) -> Vec<T> {
    let noise_power = T::from(0.001).unwrap() * sample_freq / T::from(2.).unwrap();
    let sigma = noise_power.sqrt();
    (0..sample_size)
        .map(|i| T::from(i).unwrap() / sample_freq)
        .map(|t| {
            amplification * (T::from(2. * std::f64::consts::PI).unwrap() * freq * t).sin()
                //+ T::from(thread_rng().sample::<f64, StandardNormal>(StandardNormal)).unwrap() * sigma
        }).collect()
}


// pub struct SigFigInt<T: Num, U: PrimInt> {
//     pub value: T,
//     pub ivalue: U,
    
// }

// impl<T: Num, U: PrimInt> SigFigInt<T, U> {
//     pub fn create<T: Num, U: Num>(value: T, bits: usize) -> U {
    
//     }

// }

// impl<T: Num, U: PrimInt> std::fmt::Display for SigFigInt<T, U> {
//     pub fn create<T: Num, U: Num>(value: T, bits: usize) -> U {
    
//     }

// }

