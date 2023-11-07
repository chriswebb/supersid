use num_traits::{{ToPrimitive, Num, int::PrimInt}};
use rand::prelude::*;
use rand_distr::StandardNormal;
use core::convert::From;

pub trait Sample:
    Default + Sized + ::num_traits::cast::FromPrimitive + ::num_traits::cast::ToPrimitive + Copy
{
}

impl Sample for f64 {}
impl Sample for f32 {}
impl Sample for u8 {}
impl Sample for i8 {}
impl Sample for u16 {}
impl Sample for i16 {}
impl Sample for u32 {}
impl Sample for i32 {}
impl Sample for crate::math::u24 {}
impl Sample for crate::math::i24 {}



pub fn generate_tone<T: Sample + ::num_traits::ToPrimitive>(freq: f64, sample_freq: f64, sample_size: usize, amplification: f64) -> Vec<T> {
    (0..sample_size)
        .map(|i| i as f64 / sample_freq)
        .map(|t| {
            T::from_f64(amplification * (2. * std::f64::consts::PI * freq * t).sin()).unwrap()
        }).collect()
}

pub fn generate_tone_with_noise<T: Sample + ::num_traits::ToPrimitive>(freq: f64, sample_freq: f64, sample_size: usize, amplification: f64) -> Vec<T> {
    let noise_power = 0.001 * sample_freq / 2.;
    let sigma = noise_power.sqrt();
    (0..sample_size)
        .map(|i| i as f64 / sample_freq)
        .map(|t| {
            T::from_f64(
                amplification * (2. * std::f64::consts::PI * freq * t).sin()
                + thread_rng().sample::<f64, StandardNormal>(StandardNormal) * sigma
            ).unwrap()
        }).collect()
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct i24 {
    pub first_byte: u8,
    pub second_byte: u8,
    pub third_byte: u8
}

impl i24 {

    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), 3);
        Self { first_byte: data[0], second_byte: data[1], third_byte: data[2] }
    }

    pub fn new_bytes(first_byte: u8, second_byte: u8, third_byte: u8) -> Self {
        Self { first_byte: first_byte, second_byte: second_byte, third_byte: third_byte }
    }

    pub fn value_to_i32(&self) -> i32 {

        let mut val: i32 = 0;
        // Add the 8 ncessary bits for negative integers.
        let mut neg = cfg!(target_endian = "big") && (self.third_byte & 0x01) == 0x01;
        neg = neg || self.first_byte >= 0x80;
        if (neg) {
            val |= 0xFF as i32;
            val <<= 8;
        }
        val |= self.first_byte as i32;
        val <<= 8;
        val |= self.second_byte as i32;
        val <<= 8;
        val |= self.third_byte as i32;
        if cfg!(target_endian = "big") {
            val <<= 8;
        }
        return val;
    }

    pub fn value_from_i32(val: i32) -> Self {
        let mut new_val: i32;
        let mut new_u24 = Self {first_byte: 0xFF, second_byte: 0xFF, third_byte: 0xFF };
        // Chop the 8 unncessary bits.
        if cfg!(target_endian = "big") {
            new_val = val >> 8;
        }
        else {
            new_val = (val << 8) >> 8;
        }
        new_u24.third_byte &= new_val as u8;
        new_val >>= 8;
        new_u24.second_byte &= new_val as u8;
        new_val >>= 8;
        new_u24.first_byte &= new_val as u8;
        return new_u24;
    }
}


impl Default for i24 {
    fn default() -> Self { Self { first_byte: 0, second_byte: 0, third_byte: 0 } }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct u24 {
    pub first_byte: u8,
    pub second_byte: u8,
    pub third_byte: u8
}


impl u24 {
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), 3);
        Self { first_byte: data[0], second_byte: data[1], third_byte: data[2] }
    }

    pub fn new_bytes(first_byte: u8, second_byte: u8, third_byte: u8) -> Self {
        Self { first_byte: first_byte, second_byte: second_byte, third_byte: third_byte }
    }

    pub fn value_to_u32(&self) -> u32 {
        let mut val: u32 = 0;
        val |= self.first_byte as u32;
        val <<= 8;
        val |= self.second_byte as u32;
        val <<= 8;
        val |= self.third_byte as u32;
        if cfg!(target_endian = "big") {
            val <<= 8;
        }
        return val;
    }

    pub fn value_from_u32(val: u32) -> Self {
        let mut new_val: u32;
        let mut new_u24 = Self {first_byte: 0xFF, second_byte: 0xFF, third_byte: 0xFF };
        if cfg!(target_endian = "big") {
            new_val = val >> 8;
        }
        else {
            new_val = (val << 8) >> 8;
        }
        new_u24.third_byte &= new_val as u8;
        new_val >>= 8;
        new_u24.second_byte &= new_val as u8;
        new_val >>= 8;
        new_u24.first_byte &= new_val as u8;
        return new_u24;
    }
}


impl Default for u24 {
    fn default() -> Self { Self { first_byte: 0, second_byte: 0, third_byte: 0 } }
}


macro_rules! impl_i24_from_small {
    ($Small: ty) => {
        impl From<$Small> for i24 {
            #[inline(always)]
            fn from(small: $Small) -> Self {
                i24::value_from_i32(small as i32)
            }
        }
    };
}

macro_rules! impl_i24_from_large {
    ($Large: ty) => {
        impl From<i24> for $Large {
            #[inline(always)]
            fn from(small: i24) -> Self {
                small.value_to_i32() as Self
            }
        }
    };
}

macro_rules! impl_u24_from_small {
    ($Small: ty) => {
        impl From<$Small> for u24 {
            #[inline(always)]
            fn from(small: $Small) -> Self {
                u24::value_from_u32(small as u32)
            }
        }
    };
}

macro_rules! impl_u24_from_large {
    ($Large: ty) => {
        impl From<u24> for $Large {
            #[inline(always)]
            fn from(small: u24) -> Self {
                small.value_to_u32() as Self
            }
        }
    };
}



impl_i24_from_small! { i8 }
impl_i24_from_small! { i16 }
impl_i24_from_large! { i32 }
impl_i24_from_large! { i64 }
impl_i24_from_large! { i128 }
impl_i24_from_large! { isize }


impl_i24_from_small! { u8 }
impl_i24_from_small! { u16 }
impl_i24_from_large! { u32 }
impl_i24_from_large! { u64 }
impl_i24_from_large! { u128 }
impl_i24_from_large! { usize }



impl_i24_from_large! { f64 }
impl_i24_from_large! { f32 }


impl_u24_from_small! { i8 }
impl_u24_from_small! { i16 }
impl_u24_from_large! { i32 }
impl_u24_from_large! { i64 }
impl_u24_from_large! { i128 }
impl_u24_from_large! { isize }


impl_u24_from_small! { u8 }
impl_u24_from_small! { u16 }
impl_u24_from_large! { u32 }
impl_u24_from_large! { u64 }
impl_u24_from_large! { u128 }
impl_u24_from_large! { usize }



impl_u24_from_large! { f64 }
impl_u24_from_large! { f32 }



macro_rules! impl_from_primitive {
    ($T:ty, $to_ty:ident, $from_ty:ident) => {
        #[allow(deprecated)]
        impl ::num_traits::FromPrimitive for $T {
            #[inline]
            fn from_isize(n: isize) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_i8(n: i8) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_i16(n: i16) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_i32(n: i32) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_i64(n: i64) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_i128(n: i128) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }

            #[inline]
            fn from_usize(n: usize) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_u8(n: u8) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_u16(n: u16) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_u32(n: u32) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_u64(n: u64) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_u128(n: u128) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }

            #[inline]
            fn from_f32(n: f32) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
            #[inline]
            fn from_f64(n: f64) -> Option<$T> {
                Some(<$T>::$from_ty(n.$to_ty().unwrap()))
            }
        }
    };
}

impl_from_primitive!(i24, to_i32, value_from_i32);
impl_from_primitive!(u24, to_u32, value_from_u32);


macro_rules! impl_to_primitive {
    ($T:ty, $from_ty:ident, $to_ty:ident) => {
        #[allow(deprecated)]
        impl ::num_traits::ToPrimitive for $T {
            #[inline]
            fn to_isize(&self) -> Option<isize> {
                Some(self.$to_ty() as isize)
            }
            #[inline]
            fn to_i8(&self) -> Option<i8> {
                Some(self.$to_ty() as i8)
            }
            #[inline]
            fn to_i16(&self) -> Option<i16> {
                Some(self.$to_ty() as i16)
            }
            #[inline]
            fn to_i32(&self) -> Option<i32> {
                Some(self.$to_ty() as i32)
            }
            #[inline]
            fn to_i64(&self) -> Option<i64> {
                Some(self.$to_ty() as i64)
            }
            #[inline]
            fn to_i128(&self) -> Option<i128> {
                Some(self.$to_ty() as i128)
            }

            #[inline]
            fn to_usize(&self) -> Option<usize> {
                Some(self.$to_ty() as usize)
            }
            #[inline]
            fn to_u8(&self) -> Option<u8> {
                Some(self.$to_ty() as u8)
            }
            #[inline]
            fn to_u16(&self) -> Option<u16> {
                Some(self.$to_ty() as u16)
            }
            #[inline]
            fn to_u32(&self) -> Option<u32> {
                Some(self.$to_ty() as u32)
            }
            #[inline]
            fn to_u64(&self) -> Option<u64> {
                Some(self.$to_ty() as u64)
            }
            #[inline]
            fn to_u128(&self) -> Option<u128> {
                Some(self.$to_ty() as u128)
            }

            #[inline]
            fn to_f32(&self) -> Option<f32> {
                Some(self.$to_ty() as f32)
            }
            #[inline]
            fn to_f64(&self) -> Option<f64> {
                Some(self.$to_ty() as f64)
            }
        }
    };
}

impl_to_primitive!(i24, from_i32, value_to_i32);
impl_to_primitive!(u24, from_u32, value_to_u32);


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

