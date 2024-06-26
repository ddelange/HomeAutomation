use crate::button::Press;
use crate::button_enum;
#[cfg(feature = "alloc")]
use crate::tomato::{Tomato, TomatoId, TomatoItem, TomatoLeaf};

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

button_enum! {
    /// All button are on the headboard of the bed. As seen from the foot of the
    /// bed it looks like this:
    ///
    /// LR----------------------------|
    /// |--------------|--------------|
    /// |-----------321|123-----------|
    /// |654-----------|-----------456|
    ///
    /// Legend:
    /// L: TopLeft, R: TopRight, 1: MiddleInner, 2: MiddleCenter, 3: MiddleOuter,
    /// 4: OuterInner, 5: OuterCenter, 6: OuterOuter
    Button {
        TopLeft,
        TopRight,
        MiddleInner,
        MiddleCenter,
        MiddleOuter,
        LowerInner,
        LowerCenter,
        LowerOuter,
    }
}

#[derive(
    strum::EnumDiscriminants, Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize,
)]
#[strum_discriminants(derive(Hash))]
pub enum Reading {
    Button(Button),
    Brightness(f32),
    Temperature(f32),
    Humidity(f32),
    GassResistance(f32), // in Ohm
    Pressure(f32),
    /// parts per million
    Co2(u16),
    /// weight on the left side of the bed
    WeightLeft(u32),
    /// weight on the right side of the bed
    WeightRight(u32),

    /// Mass Concentration PM1.0 \[μg/m³\]
    MassPm1_0(f32),
    /// Mass Concentration PM2.5 \[μg/m³\]
    MassPm2_5(f32),
    /// Mass Concentration PM4.0 \[μg/m³\]
    MassPm4_0(f32),
    /// Mass Concentration PM10 \[μg/m³\]
    MassPm10(f32),
    /// Number Concentration PM0.5 \[#/cm³\]
    MassPm0_5(f32),
    /// Number Concentration PM1.0 \[#/cm³\]
    NumberPm1_0(f32),
    /// Number Concentration PM2.5 \[#/cm³\]
    NumberPm2_5(f32),
    /// Number Concentration PM4.0 \[#/cm³\]
    NumberPm4_0(f32),
    /// Number Concentration PM10 \[#/cm³\]
    NumberPm10(f32),
    /// Typical Particle Size8 \[μm\]
    TypicalParticleSize(f32),
}

#[cfg(feature = "alloc")]
macro_rules! put_in_tree {
    ([$($reading:expr),+]) => {
        [$(crate::Reading::LargeBedroom(
            crate::large_bedroom::Reading::Bed($reading),
        )),+]
    };
}

#[cfg(feature = "alloc")]
fn leaf(val: &f32, device: Device, from_same_device: &'static [crate::Reading]) -> TomatoLeaf {
    TomatoLeaf {
        val: *val,
        device: device.as_str(),
        from_same_device,
    }
}

#[cfg(feature = "alloc")]
impl Tomato for Reading {
    fn inner<'a>(&'a self) -> TomatoItem<'a> {
        let (val, device) = match self {
            Reading::Brightness(val) => (*val, Device::Max44),
            Reading::Temperature(val) => (*val, Device::Sht31),
            Reading::Humidity(val) => (*val, Device::Sht31),
            Reading::GassResistance(val) => (*val, Device::Bme680),
            Reading::Pressure(val) => (*val, Device::Bme680),
            Reading::Co2(val) => (*val as f32, Device::Mhz14),
            Reading::WeightLeft(val) => (*val as f32, Device::Nau7802Left),
            Reading::WeightRight(val) => (*val as f32, Device::Nau7802Right),
            Reading::Button(val) => ((*val).into(), Device::Gpio),
            Reading::MassPm1_0(val) => (*val, Device::Sps30),
            Reading::MassPm2_5(val) => (*val, Device::Sps30),
            Reading::MassPm4_0(val) => (*val, Device::Sps30),
            Reading::MassPm10(val) => (*val, Device::Sps30),
            Reading::MassPm0_5(val) => (*val, Device::Sps30),
            Reading::NumberPm1_0(val) => (*val, Device::Sps30),
            Reading::NumberPm2_5(val) => (*val, Device::Sps30),
            Reading::NumberPm4_0(val) => (*val, Device::Sps30),
            Reading::NumberPm10(val) => (*val, Device::Sps30),
            Reading::TypicalParticleSize(val) => (*val, Device::Sps30),
        };

        // let related = todo!();

            // Reading::Brightness(val) => leaf(val, Device::Max44, &[]),
            // Reading::Temperature(val) => {
            //     leaf(val, Device::Sht31, &put_in_tree!([Reading::Humidity(0.0)]))
            // }
        TomatoItem::Leaf(TomatoLeaf {
            val,
            device: device.as_str(),
            from_same_device: todo!(),
        })
    }

    fn id(&self) -> TomatoId {
        ReadingDiscriminants::from(self) as TomatoId
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum Error {
    Running(SensorError),
    Setup(SensorError),
    SetupTimedOut(Device),
    Timeout(Device),
}

impl Error {
    pub fn affected_readings(&self) -> &'static [Reading] {
        match self {
            Self::Running(sensor_err) | Self::Setup(sensor_err) => sensor_err.device(),
            Self::SetupTimedOut(device) | Self::Timeout(device) => device.clone(),
        }
        .broken_readings()
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Running(e) => write!(f, "{} ran into error: {e}", e.device()),
            Error::Setup(e) => write!(f, "{} errored during setup: {e}", e.device()),
            Error::SetupTimedOut(d) => write!(f, "{d} timed out during setup"),
            Error::Timeout(d) => write!(f, "{d} timed out running"),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, Eq, PartialEq)]
pub enum SensorError {
    Sht31(heapless::String<200>),
    Bme680(heapless::String<200>),
    Max44(heapless::String<200>),
    Mhz14(heapless::String<200>),
    Sps30(heapless::String<200>),
    Nau7802Left(heapless::String<200>),
    Nau7802Right(heapless::String<200>),
}

impl core::fmt::Display for SensorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SensorError::Sht31(e) => write!(f, "{e}"),
            SensorError::Bme680(e) => write!(f, "{e}"),
            SensorError::Max44(e) => write!(f, "{e}"),
            SensorError::Mhz14(e) => write!(f, "{e}"),
            SensorError::Sps30(e) => write!(f, "{e}"),
            SensorError::Nau7802Left(e) => write!(f, "{e}"),
            SensorError::Nau7802Right(e) => write!(f, "{e}"),
        }
    }
}

impl MaxSize for SensorError {
    const POSTCARD_MAX_SIZE: usize = 201;
}

impl SensorError {
    pub fn device(&self) -> Device {
        match self {
            SensorError::Sht31(_) => Device::Sht31,
            SensorError::Bme680(_) => Device::Bme680,
            SensorError::Max44(_) => Device::Max44,
            SensorError::Mhz14(_) => Device::Mhz14,
            SensorError::Sps30(_) => Device::Sps30,
            SensorError::Nau7802Left(_) => Device::Nau7802Left,
            SensorError::Nau7802Right(_) => Device::Nau7802Right,
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum Device {
    Sht31,
    Bme680,
    Max44,
    Mhz14,
    Sps30,
    Nau7802Left,
    Nau7802Right,
    Gpio,
}

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Device {
    pub fn as_str(&self) -> &'static str {
        match self {
            Device::Sht31 => "Sht31",
            Device::Bme680 => "Bme680",
            Device::Max44 => "Max44",
            Device::Mhz14 => "Mhz14",
            Device::Sps30 => "Sps30",
            Device::Nau7802Left => "Nau7802Left",
            Device::Nau7802Right => "Nau7802Right",
            Device::Gpio => "Gpio",
        }
    }

    pub fn broken_readings(&self) -> &'static [Reading] {
        match self {
            Device::Sht31 => &[Reading::Temperature(0.0), Reading::Humidity(0.0)],
            Device::Bme680 => &[Reading::GassResistance(0.0), Reading::Pressure(0.0)],
            Device::Max44 => &[Reading::Brightness(0.0)],
            Device::Mhz14 => &[Reading::Co2(0)],
            Device::Sps30 => &[
                Reading::MassPm1_0(0.0),
                Reading::MassPm2_5(0.0),
                Reading::MassPm4_0(0.0),
                Reading::MassPm10(0.0),
                Reading::MassPm0_5(0.0),
                Reading::NumberPm1_0(0.0),
                Reading::NumberPm2_5(0.0),
                Reading::NumberPm4_0(0.0),
                Reading::NumberPm10(0.0),
                Reading::TypicalParticleSize(0.0),
            ],
            Device::Nau7802Left => &[Reading::WeightLeft(0)],
            Device::Nau7802Right => &[Reading::WeightRight(0)],
            Device::Gpio => &[
                Reading::Button(Button::TopLeft(Press(0))),
                Reading::Button(Button::TopRight(Press(0))),
                Reading::Button(Button::MiddleInner(Press(0))),
                Reading::Button(Button::MiddleCenter(Press(0))),
                Reading::Button(Button::MiddleOuter(Press(0))),
                Reading::Button(Button::LowerInner(Press(0))),
                Reading::Button(Button::LowerCenter(Press(0))),
                Reading::Button(Button::LowerOuter(Press(0))),
            ],
        }
    }
}
