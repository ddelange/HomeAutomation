use std::fs::create_dir_all;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use byteseries::{downsample, series, ByteSeries};
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::{Result, Section};
use protocol::reading_tree::{Item, ReadingInfo, Tree};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, trace};

use byteseries::file::OpenError as FileOpenError;
use series::data::OpenError as DataOpenError;
use series::Error::Open;

mod bitspec;
mod resampler;

use self::resampler::Resampler;

use super::Data;

#[derive(Debug)]
struct Meta {
    reading: protocol::Reading,
    field: bitspec::Field<f32>,
    set_at: Option<Instant>,
}

#[derive(Debug)]
pub(crate) struct Series {
    line: Vec<u8>,
    meta: Vec<Meta>,
    byteseries: ByteSeries,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Header {
    readings: Vec<protocol::Reading>,
    encoding: Vec<bitspec::Field<f32>>,
}

impl Series {
    fn open_or_create(reading: &protocol::Reading, dir: &Path) -> Result<Self> {
        let readings = reading.device().info().affects_readings;
        let specs = to_speclist(readings);
        let fields = bitspec::speclist_to_fields(specs);
        let meta = readings
            .iter()
            .zip(fields.iter())
            .map(|(reading, field)| Meta {
                reading: reading.clone(),
                field: field.clone(),
                set_at: None,
            })
            .collect();

        let payload_size = fields
            .iter()
            .map(|spec| spec.length as usize)
            .sum::<usize>()
            .div_ceil(8);
        let resampler = Resampler::from_fields(fields.clone(), payload_size);
        let resample_configs = vec![
            downsample::Config {
                max_gap: None,
                bucket_size: 10,
            },
            downsample::Config {
                max_gap: None,
                bucket_size: 100,
            },
            downsample::Config {
                max_gap: None,
                bucket_size: 1000,
            },
        ];

        let path = base_path(reading);
        let path = dir.join(path);
        let res = ByteSeries::open_existing_with_resampler::<Header, _>(
            &path,
            payload_size,
            resampler.clone(),
            resample_configs.clone(),
        );

        let expected_header = Header {
            readings: readings.to_vec(),
            encoding: fields.clone(),
        };

        let byteseries = try_create_new_if_open_failed(
            res,
            expected_header,
            &path,
            payload_size,
            resampler,
            resample_configs,
        )?;

        Ok(Self {
            line: vec![0; payload_size],
            meta,
            byteseries,
        })
    }

    fn append(&mut self, reading: &protocol::Reading) -> Result<()> {
        let index = reading
            .device()
            .info()
            .affects_readings
            .iter()
            .map(|r| r.leaf().branch_id)
            .position(|id_in_list| id_in_list == reading.leaf().branch_id)
            .expect(
                "reading.device.affected_readings() is a list that contains \
                reading.branch_id()",
            );

        let meta = &mut self.meta[index];
        meta.field.encode(reading.leaf().val, &mut self.line);
        meta.set_at = Some(Instant::now());

        if self
            .meta
            .iter()
            .map(|Meta { set_at, .. }| set_at)
            .all(|set| set.map(|s| s.elapsed().as_millis() < 500).unwrap_or(false))
        {
            let time = jiff::Timestamp::now();

            let device_info = reading.leaf().device.info();
            let scale_factor = millis_to_minimal_representation(device_info);
            let scaled_time = time.as_millisecond() as u64 / scale_factor;
            self.byteseries
                .push_line(scaled_time, &self.line)
                .wrap_err("Could not write to timeseries on disk")?;
            self.line.fill(0);
        }

        Ok(())
    }

    /// # Panics
    /// If any of the requested readings are not part of this series.
    #[instrument(skip(self))]
    pub fn read(
        &mut self,
        readings: &[protocol::Reading],
        start: jiff::Timestamp,
        end: jiff::Timestamp,
        n: usize,
    ) -> Result<(Vec<jiff::Timestamp>, Vec<Vec<f32>>), byteseries::series::Error> {
        let device_info = readings
            .first()
            .expect("There is at least one reading to read")
            .leaf()
            .device
            .info();
        let scale_factor = millis_to_minimal_representation(device_info);
        let start = start.as_millisecond() as u64 / scale_factor;
        let end = end.as_millisecond() as u64 / scale_factor;
        let range = start..=end;
        let fields = readings
            .iter()
            .map(|requested| {
                self.meta
                    .iter()
                    .find(|meta| requested.is_same_as(&meta.reading))
                    .inspect(|meta| trace!("meta used for decoding: {meta:?}"))
                    .map(|meta| meta.field.clone())
                    .unwrap_or_else(|| panic!(
                        "caller of read makes sure all readings are part of this \
                        series.\n\tseries: {:?},\n\trequested: {:?}",
                        self.meta, readings
                    ))
            })
            .collect();
        let mut resampler = Resampler::from_fields(fields, self.line.len());

        let mut timestamps = Vec::with_capacity(n * 2);
        let mut interleaved_data = Vec::with_capacity(n * 2);

        self.byteseries.read_n(
            n,
            range,
            &mut resampler,
            &mut timestamps,
            &mut interleaved_data,
        )?;

        let time = timestamps
            .into_iter()
            .map(|ts| {
                let millis = ts * scale_factor;
                jiff::Timestamp::from_millisecond(millis as i64)
                    .expect("timestamps are between MIN and MAX times of Timestamp type")
            })
            .collect();

        let len = interleaved_data
            .first()
            .expect("read returns an error if there is not data")
            .len();
        let mut data = vec![Vec::new(); len];
        for interleaved in interleaved_data {
            for (interleaved, data) in interleaved.into_iter().zip(data.iter_mut()) {
                data.push(interleaved);
            }
        }
        Ok((time, data))
    }
}

/// Multiplying the time for this sample by this factor
/// allows you to save the whole number and retain the required
/// temporal_resolution and min_sample_interval.
pub fn millis_to_minimal_representation(device_info: protocol::DeviceInfo) -> u64 {
    let needed_interval = device_info
        .temporal_resolution
        .min(device_info.min_sample_interval)
        .as_secs_f32();
    let mul_factor = 0.001 / needed_interval;
    let div_factor = 1. / mul_factor;
    div_factor.round() as u64
}

fn try_create_new_if_open_failed(
    res: Result<(ByteSeries, Header), series::Error>,
    expected_header: Header,
    path: &Path,
    payload_size: usize,
    resampler: Resampler,
    resample_configs: Vec<downsample::Config>,
) -> Result<ByteSeries, color_eyre::eyre::Error> {
    match res {
        Ok((byteseries, opened_file_header)) => {
            if opened_file_header == expected_header {
                info!("Opened existing byteseries from: {}", path.display());
                Ok(byteseries)
            } else {
                Err(eyre!("header in file does not match readings"))
                    .with_note(|| {
                        format!(
                            "header in the just existing (opened) byteseries: {opened_file_header:?}",
                        )
                    })
                    .with_note(|| {
                        format!(
                            "header for the data we want to write: {expected_header:?}",
                        )
                    })
            }
        }
        Err(Open(DataOpenError::File(FileOpenError::Io(e))))
            if e.kind() == io::ErrorKind::NotFound =>
        {
            if let Some(dirs) = path.parent() {
                create_dir_all(dirs)
                    .wrap_err("Could not create dirs structure for reading")
                    .with_note(|| format!("dirs: {}", dirs.display()))?;
                std::fs::read_dir(dirs)
                    .unwrap()
                    .map(Result::unwrap)
                    .for_each(|p| println!("{p:?}"));
            }
            // compile_error!("create directory structure");
            info!("creating new byteseries");
            ByteSeries::new_with_resamplers(
                &path,
                payload_size,
                expected_header,
                resampler,
                resample_configs,
            )
            .wrap_err("Could not create new byteseries")
            .with_note(|| format!("path: {}", path.display()))
        }
        Err(e) => Err(e).wrap_err("Could not open existing byteseries")?,
    }
}

#[instrument(level = "debug", skip(data))]
pub(crate) async fn store(data: &Data, reading: &protocol::Reading, data_dir: &Path) -> Result<()> {
    let mut data = data.0.lock().await;

    let key = reading.device();
    if let Some(series) = data.get_mut(&key) {
        series
            .append(reading)
            .wrap_err("failed to append to existing timeseries")?;
    } else {
        let mut series = Series::open_or_create(reading, data_dir)
            .wrap_err("Could not open new series")
            .with_note(|| format!("reading was: {reading:?}"))?;
        series
            .append(reading)
            .wrap_err("failed to newly created timeseries")?;
        let existing = data.insert(key, series);
        assert!(existing.is_none(), "should not race we still hold the lock");
    }

    Ok(())
}

fn to_speclist(readings: &[protocol::Reading]) -> Vec<bitspec::LengthWithOps> {
    readings
        .iter()
        .map(|r| bitspec::RangeWithRes {
            range: r.range(),
            resolution: r.resolution(),
        })
        .map(bitspec::LengthWithOps::from)
        .collect()
}

/// relative path without extension
fn base_path(reading: &protocol::Reading) -> PathBuf {
    let mut parts = Vec::new();
    let mut current = reading as &dyn Tree;
    loop {
        match current.inner() {
            Item::Leaf(ReadingInfo { device, .. }) => {
                parts.push(device.info().name.to_lowercase());
                break;
            }
            Item::Node(inner) => {
                parts.push(current.name().to_lowercase());
                current = inner;
                continue;
            }
        }
    }
    parts.into_iter().collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use protocol::large_bedroom::{bed, desk};
    use protocol::{large_bedroom, Reading};

    #[test]
    fn millis_to_minimal_representation_factor_makes_sense() {
        let info = protocol::DeviceInfo {
            name: "test",
            affects_readings: &[],
            min_sample_interval: std::time::Duration::from_secs(5),
            max_sample_interval: std::time::Duration::from_secs(5),
            temporal_resolution: std::time::Duration::from_secs(1),
        };
        let factor = millis_to_minimal_representation(info);
        assert_eq!(5000 / factor, 5);

        let info = protocol::DeviceInfo {
            name: "test",
            affects_readings: &[],
            min_sample_interval: std::time::Duration::from_secs(5),
            max_sample_interval: std::time::Duration::from_secs(5),
            temporal_resolution: std::time::Duration::from_millis(1),
        };
        let factor = millis_to_minimal_representation(info);
        assert_eq!(5005 / factor, 5005)
    }

    #[test]
    fn readings_from_same_device_have_same_path() {
        let reading_a =
            Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Temperature(0.0)));
        let reading_b =
            Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Humidity(0.0)));

        assert_eq!(base_path(&reading_a), base_path(&reading_b));
    }

    #[test]
    fn reading_path_different_between_locations() {
        let reading_a =
            Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Humidity(0.0)));
        let reading_b =
            Reading::LargeBedroom(large_bedroom::Reading::Desk(desk::Reading::Humidity(0.0)));

        assert_ne!(base_path(&reading_a), base_path(&reading_b));
    }

    #[test]
    fn reading_path_is_expected() {
        let reading =
            Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Humidity(0.0)));
        assert_eq!(base_path(&reading), PathBuf::from("largebedroom/bed/sht31"));
    }
}
