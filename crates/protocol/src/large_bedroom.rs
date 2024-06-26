use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::all_nodes;

pub mod bed;
pub mod desk;

#[derive(
    strum::EnumDiscriminants, Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize,
)]
#[strum_discriminants(derive(Hash))]
pub enum Reading {
    Bed(bed::Reading),
    Desk(desk::Reading),
}

#[cfg(feature = "alloc")]
all_nodes! {Reading; ReadingDiscriminants; Bed, Desk}

#[derive(
    strum::EnumDiscriminants,
    Clone,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    Eq,
    PartialEq,
)]
pub enum Error {
    Bed(bed::Error),
    Desk(desk::Error),
}

impl Error {
    pub fn broken_readings(&self) -> impl Iterator<Item = Reading> {
        match self {
            Error::Bed(error) => error
                .affected_readings()
                .into_iter()
                .cloned()
                .map(Reading::Bed),
            Error::Desk(_) => todo!(),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Bed(error) => write!(f, "{}", error),
            Error::Desk(error) => write!(f, "{}", error),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum Device {
    Bed(bed::Device),
    Desk(desk::Device),
}

#[derive(Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
pub enum Actuator {
    CleanSensor,
    CalibrateCo2,
}

impl Actuator {
    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    /// Buffer should be at least Self::ENCODED_SIZE long. The returned slice contains
    /// the serialized data. It can be shorter then the input buffer.
    pub fn encode_slice<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        postcard::to_slice_cobs(self, buf).expect("Encoding should not fail")
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, crate::DecodeError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(crate::DecodeError::CorruptEncoding)
    }

    pub fn version(&self) -> u8 {
        0
    }
}
