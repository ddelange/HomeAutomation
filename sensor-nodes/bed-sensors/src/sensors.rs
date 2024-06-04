use defmt::info;
use embassy_embedded_hal::shared_bus;
use embassy_futures::join;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::Uart;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{with_timeout, Delay, Duration};
use max44009::{Max44009, SlaveAddr};
use mhzx::MHZ;
use protocol::large_bedroom::bed::{Device, Error, SensorError};
use crate::error_cache::make_error_string;
use sps30_async::Sps30;

use crate::channel::Channel;

pub mod fast;
pub mod slow;

// Todo make failed init not critical. Keep trying init in background
// while we are measuring

pub async fn init_then_measure(
    publish: &Channel,
    i2c: Mutex<NoopRawMutex, I2c<'static, Async>>,
    usart_mhz: Uart<'static, Async>,
    usart_sps: Uart<'static, Async>,
) -> Result<(), protocol::large_bedroom::bed::Error> {

    info!("initializing sensors");
    let bme_config = bosch_bme680::Configuration::default();
    let bme = with_timeout(
        Duration::from_secs(2),
        bosch_bme680::Bme680::new(
            shared_bus::asynch::i2c::I2cDevice::new(&i2c),
            bosch_bme680::DeviceAddress::Secondary,
            Delay,
            &bme_config,
            20,
        ),
    )
    .await
    .map_err(|_| Error::SetupTimedOut(Device::Bme680))?
    .map_err(|err| make_error_string(err))
    .map_err(SensorError::Bme680)
    .map_err(Error::Setup)?;

    defmt::info!("hi");
    let mut max44009 = Max44009::new(
        shared_bus::asynch::i2c::I2cDevice::new(&i2c),
        SlaveAddr::default(),
    );
    with_timeout(
        Duration::from_millis(250),
        max44009.set_measurement_mode(max44009::MeasurementMode::Continuous),
    )
    .await
    .map_err(|_| Error::SetupTimedOut(Device::Max44))?
    .map_err(|err| make_error_string(err))
    .map_err(SensorError::Max44)
    .map_err(Error::Setup)?;

    let sht = sht31::SHT31::new(shared_bus::asynch::i2c::I2cDevice::new(&i2c), Delay)
        .with_mode(sht31::mode::SingleShot)
        .with_unit(sht31::TemperatureUnit::Celsius)
        .with_accuracy(sht31::Accuracy::High);

    let (tx, rx) = usart_mhz.split();
    let mut usart_buf = [0u8; 9 * 10]; // 9 byte messages
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let mhz = MHZ::from_tx_rx(tx, rx);

    let (tx, rx) = usart_sps.split();
    let mut usart_buf = [0u8; 100];
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let sps30 = with_timeout(Duration::from_millis(100), Sps30::from_tx_rx(tx, rx, Delay))
        .await
        .map_err(|_| Error::SetupTimedOut(Device::Sps30))?
        .map_err(|err| make_error_string(err))
        .map_err(SensorError::Sps30)
        .map_err(Error::Setup)?;

    let sensors_fast = fast::read(max44009, /*buttons,*/ &publish);
    let sensors_slow = slow::read(sht, bme, mhz, sps30, &publish);
    join::join(sensors_fast, sensors_slow).await;

    defmt::unreachable!();
}
