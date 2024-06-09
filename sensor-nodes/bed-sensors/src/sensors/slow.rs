use defmt::unwrap;
use embassy_futures::join;
use embassy_time::{with_timeout, Delay, Duration, Timer};

use mhzx::MHZ;
use protocol::large_bedroom::bed::{Device, Reading};

use bosch_bme680::{Bme680, MeasurementData};
use sht31::mode::{Sht31Measure, Sht31Reader, SingleShot};
use sht31::SHT31;
use sps30_async as sps30;
use sps30_async::Sps30;

use crate::channel::Queues;
use crate::error_cache::Error;
use crate::error_cache::SensorError;

use super::{I2cError, UartError};

const SPS30_UART_BUF_SIZE: usize = 100;
const SPS30_DRIVER_BUF_SIZE: usize = 2 * SPS30_UART_BUF_SIZE;

use super::concrete_types::ConcreteRx as Rx;
use super::concrete_types::ConcreteSharedI2c as I2c;
use super::concrete_types::ConcreteTx as Tx;

#[inline(always)]
pub async fn read(
    mut sht: SHT31<SingleShot, I2c<'_>>,
    mut bme: Bme680<I2c<'_>, Delay>,
    mut mhz: MHZ<Tx<'_>, Rx<'_>>,
    mut sps: Sps30<SPS30_DRIVER_BUF_SIZE, Tx<'_>, Rx<'_>, Delay>,
    publish: &'_ Queues,
) {
    // sht works in two steps
    //  - send measure command before sleep
    //  - then read
    if let Err(err) = sht.measure().await {
        let err = SensorError::Sht31(err);
        let err = Error::Running(err);
        publish.queue_error(err)
    }
    Timer::after_secs(1).await;

    loop {
        defmt::info!("this is where we break");
        let sht_read = with_timeout(Duration::from_millis(100), sht.read());
        let bme_measure = bme.measure(); // can not hang
        let mhz_measure = with_timeout(Duration::from_millis(100), mhz.read_co2());
        let sps_measure = with_timeout(Duration::from_millis(100), sps.read_measurement());
        let (bme_res, sht_res, mhz_res, sps_res) =
            join::join4(bme_measure, sht_read, mhz_measure, sps_measure).await;

        publish_bme_result(bme_res, publish);
        publish_sht_result(sht_res, publish);
        publish_mhz_result(mhz_res, publish);
        publish_sps_result(sps_res, publish);

        // sht works in two steps
        //  - send measure command before sleep
        //  - then read
        if let Err(err) = sht.measure().await {
            let err = SensorError::Sht31(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
        Timer::after_secs(1).await;
    }
}

fn publish_sps_result(
    sps_res: Result<
        Result<Option<sps30::Measurement>, sps30::Error<UartError, UartError>>,
        embassy_time::TimeoutError,
    >,
    publish: &Queues,
) {
    match sps_res {
        Ok(Ok(Some(sps30::Measurement {
            mass_pm1_0,
            mass_pm2_5,
            mass_pm4_0,
            mass_pm10,
            mass_pm0_5,
            number_pm1_0,
            number_pm2_5,
            number_pm4_0,
            number_pm10,
            typical_particle_size,
        }))) => {
            publish.send_p0(Reading::MassPm1_0(mass_pm1_0));
            publish.send_p0(Reading::MassPm2_5(mass_pm2_5));
            publish.send_p0(Reading::MassPm4_0(mass_pm4_0));
            publish.send_p0(Reading::MassPm10(mass_pm10));
            publish.send_p0(Reading::MassPm0_5(mass_pm0_5));
            publish.send_p0(Reading::NumberPm1_0(number_pm1_0));
            publish.send_p0(Reading::NumberPm2_5(number_pm2_5));
            publish.send_p0(Reading::NumberPm4_0(number_pm4_0));
            publish.send_p0(Reading::NumberPm10(number_pm10));
            publish.send_p0(Reading::TypicalParticleSize(typical_particle_size));
        }
        Ok(Ok(None)) => {
            defmt::todo!("no idea when we hit this");
        }
        Ok(Err(err)) => {
            let err = SensorError::Sps30(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
        Err(_timeout) => {
            let err = Error::Timeout(Device::Sps30);
            publish.queue_error(err)
        }
    }
}

fn publish_mhz_result(
    mhz_res: Result<
        Result<mhzx::Measurement, mhzx::Error<UartError, UartError>>,
        embassy_time::TimeoutError,
    >,
    publish: &Queues,
) {
    match mhz_res {
        Ok(Ok(mhzx::Measurement { co2, .. })) => {
            publish.send_p0(Reading::Co2(co2));
        }
        Ok(Err(err)) => {
            let err = SensorError::Mhz14(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
        Err(_timeout) => {
            let err = Error::Timeout(Device::Mhz14);
            publish.queue_error(err)
        }
    }
}

fn publish_sht_result(
    sht_res: Result<Result<sht31::prelude::Reading, sht31::SHTError>, embassy_time::TimeoutError>,
    publish: &Queues,
) {
    match sht_res {
        Ok(Ok(sht31::Reading {
            temperature,
            humidity,
        })) => {
            publish.send_p0(Reading::Temperature(temperature));
            publish.send_p0(Reading::Humidity(humidity));
        }
        Ok(Err(err)) => {
            let err = SensorError::Sht31(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
        Err(_timeout) => {
            let err = Error::Timeout(Device::Sht31);
            publish.queue_error(err)
        }
    }
}

fn publish_bme_result(
    bme_res: Result<MeasurementData, bosch_bme680::BmeError<I2cError>>,
    publish: &Queues,
) {
    match bme_res {
        Ok(MeasurementData {
            pressure,
            gas_resistance,
            ..
        }) => {
            let gas_resistance = unwrap!(gas_resistance); // sensor is on
            publish.send_p0(Reading::GassResistance(gas_resistance));
            publish.send_p0(Reading::Pressure(pressure));
        }
        Err(err) => {
            let err = SensorError::Bme680(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
    }
}