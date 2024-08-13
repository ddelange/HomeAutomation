use hdrhistogram::Histogram;
use itertools::Itertools;
use jiff::Unit;
use log_store::api::ErrorEvent;
use protocol::reading_tree::ReadingInfo;
use protocol::reading_tree::{Item, Tree};
use protocol::Reading;
use protocol::{Device, Error};

use ratatui::{text::Line, widgets::Bar};
use std::collections::HashMap;
use std::sync::TryLockError;
use tui_tree_widget::TreeItem;

mod fetch;
use fetch::{histogram, history};

mod logs;

#[derive(Debug)]
pub struct SensorInfo {
    pub info: ReadingInfo,
    /// This value is not up to date, only use for requesting
    /// data use the last element of recent_history for printing
    pub reading: Reading,
    timing: Histogram<u64>,
    pub percentiles_from_store: histogram::Stored,
    recent_history: Vec<(jiff::Timestamp, f32)>,
    pub history_from_store: history::Stored,
    condition: Result<(), Box<Error>>,
    logs: logs::Logs,
    pub logs_from_store: fetch::logs::Stored,
}

pub struct ErrorDensity {
    pub t5_min: f32,
    pub t15_min: f32,
    pub t30_min: f32,
    pub t45_min: f32,
    pub t60_min: f32,
}

impl ErrorDensity {
    fn from_log(log: &logs::Logs) -> Self {
        let buckets = [5, 15, 30, 45, 60].map(|min| jiff::Span::new().minutes(min));
        let counts = log.density(buckets);

        Self {
            t5_min: counts[0],
            t15_min: counts[1],
            t30_min: counts[2],
            t45_min: counts[3],
            t60_min: counts[4],
        }
    }
}

pub struct Details {
    pub last_reading: Option<(jiff::Timestamp, String)>,
    pub condition: Result<(), Box<Error>>,
    pub description: String,
    pub errors_since: ErrorDensity,
}

impl SensorInfo {
    fn last_at(&self) -> Option<jiff::Timestamp> {
        self.condition.clone().ok();
        self.recent_history.last().map(|(ts, _)| ts).copied()
    }

    pub fn details(&self) -> Details {
        let last_reading = self.recent_history.last().copied().map(|(ts, val)| {
            let val = format!("{0:.1$} {2}", val, self.info.precision(), self.info.unit);
            (ts, val)
        });
        Details {
            last_reading,
            condition: self.condition.clone(),
            description: self.info.description.to_owned(),
            errors_since: ErrorDensity::from_log(&self.logs),
        }
    }

    pub fn histogram(&self) -> Vec<Bar> {
        let percentiles = if self.percentiles_from_store.very_outdated() {
            self.fallback_local_hist()
        } else {
            self.percentiles_from_store.data.lock().unwrap().clone()
        };
        histogram_bars(&percentiles)
    }

    pub fn fallback_local_hist(&self) -> Vec<log_store::api::Percentile> {
        self.timing
            .iter_quantiles(1)
            .map(|it| log_store::api::Percentile {
                percentile: it.percentile(),
                bucket_ends: it.value_iterated_to(),
                count_in_bucket: it.count_at_value(),
            })
            .dedup_by(|a, b| {
                a.bucket_ends == b.bucket_ends
                    && a.percentile.total_cmp(&b.percentile).is_eq()
                    && a.count_in_bucket == b.count_in_bucket
            })
            .collect_vec()
    }

    pub fn chart<'a>(&mut self, plot_buf: &'a mut Vec<(f64, f64)>) -> Option<ChartParts<'a>> {
        let guard = match self.history_from_store.data.try_lock() {
            Ok(list) => Some(list),
            Err(TryLockError::WouldBlock) => None,
            Err(other) => panic!(
                "fetching sensor history from data-store \
                panicked: {other}"
            ),
        };
        let empty = Vec::new();
        let old_history = guard.as_deref().unwrap_or(&empty);

        let reference = old_history
            .first()
            .map(|(t, _)| t)
            .or_else(|| self.recent_history.first().map(|(t, _)| t))?;

        let first_recent = self
            .recent_history
            .first()
            .map(|(t, _)| t)
            .cloned()
            .unwrap_or(jiff::Timestamp::default());
        plot_buf.clear();

        for xy in old_history
            .iter()
            .take_while(|(t, _)| *t < first_recent)
            .chain(self.recent_history.iter())
            .map(|(x, y)| {
                (
                    (*x - *reference)
                        .total(jiff::Unit::Second)
                        .expect("unit is not a calander unit"),
                    *y as f64,
                )
            })
        {
            plot_buf.push(xy);
        }

        Some(ChartParts {
            reading: self.info.clone(),
            data: plot_buf,
        })
    }

    pub fn logs(&self) -> Vec<ErrorEvent> {
        let mut logs = self.logs_from_store.list();
        let last = logs
            .last()
            .map(|ErrorEvent { start, .. }| *start)
            .unwrap_or(jiff::Timestamp::from_second(0).unwrap());
        let without_duplicates = self
            .logs
            .list()
            .iter()
            .skip_while(|ErrorEvent { start, .. }| start < &last)
            .cloned();
        logs.extend(without_duplicates);
        logs
    }
}

/// Guaranteed to be unique for a leaf,
/// the path to the leaf (through branch-id's) is
/// encoded with the last byte byte being the leaf's id
pub type TreeKey = [u8; 6];
pub struct Readings {
    // in the ground there are multiple trees
    pub ground: Vec<TreeItem<'static, TreeKey>>,
    pub data: HashMap<TreeKey, SensorInfo>,
}

fn add_leaf(text: String, tree: &mut TreeItem<'static, TreeKey>, key: TreeKey) {
    let new_item = TreeItem::new_leaf(key, text.clone());
    // todo is exists its fine handle that
    let _ignore_existing = tree.add_child(new_item); // errors when identifier already exists

    let new_child = tree
        .children()
        .iter()
        .position(|item| *item.identifier() == key)
        .expect("just added it");
    let existing = tree.child_mut(new_child).expect("just added it");
    existing.update_text(text);
}

fn add_root<'a>(
    tomato: &dyn Tree,
    ground: &'a mut Vec<TreeItem<'static, TreeKey>>,
) -> &'a mut TreeItem<'static, TreeKey> {
    let key = [tomato.branch_id(); 6];
    let exists = ground.iter().any(|item| *item.identifier() == key);
    if !exists {
        let new_root = TreeItem::new(key, tomato.name(), vec![]).unwrap();
        ground.push(new_root);
    }

    ground
        .iter_mut()
        .find(|item| *item.identifier() == key)
        .expect("checked and added if missing")
}

fn add_node<'a>(
    tomato: &dyn Tree,
    tree: &'a mut TreeItem<'static, TreeKey>,
) -> &'a mut TreeItem<'static, TreeKey> {
    let key = [tomato.branch_id(); 6];
    let new_item = TreeItem::new(key, tomato.name(), Vec::new()).unwrap();
    // add just in case it was not there yet
    let _ignore_existing = tree.add_child(new_item);
    let new_child = tree
        .children()
        .iter()
        .position(|item| *item.identifier() == key)
        .expect("just added it");
    tree.child_mut(new_child).expect("just added it")
}

fn extract_leaf_info(reading: &Reading) -> (TreeKey, String, f32) {
    let mut key = [0u8; 6];
    key[0] = reading.branch_id();

    let mut reading = reading as &dyn Tree;
    for byte in &mut key[1..] {
        reading = match reading.inner() {
            Item::Node(inner) => {
                *byte = inner.branch_id();
                inner
            }
            Item::Leaf(ReadingInfo { val, .. }) => {
                let name = reading.name();
                return (key, name, val);
            }
        };
    }
    unreachable!("reading should not be deeper then key size")
}

enum IsPlaceholder {
    Yes,
    No,
}

impl Readings {
    pub fn add(&mut self, reading: Reading) {
        self.update_tree(&reading, IsPlaceholder::No);
        self.record_data(reading);
    }

    pub(crate) fn populate_from_reading_list(&mut self, list: Vec<Reading>) {
        for reading in list {
            self.update_tree(&reading, IsPlaceholder::Yes);
            self.record_missing_data(reading);
        }
    }

    pub(crate) fn populate_from_device_list(&mut self, list: Vec<Device>) {
        for reading in list.iter().flat_map(|d| d.info().affects_readings) {
            self.update_tree(&reading, IsPlaceholder::Yes);
            self.record_missing_data(reading.clone());
        }
    }

    pub fn add_error(&mut self, error: Box<Error>) {
        self.update_tree_err(&error);
        self.record_error(error);
    }

    fn record_error(&mut self, error: Box<Error>) {
        for broken in error.device().info().affects_readings {
            let (key, _, _) = extract_leaf_info(broken);

            if let Some(info) = self.data.get_mut(&key) {
                info.condition = Err(error.clone());
                info.logs.add(&error);
            } else {
                let errors = logs::Logs::new_from(&error);
                self.data.insert(
                    key,
                    SensorInfo {
                        info: broken.leaf(),
                        reading: broken.clone(),
                        timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
                        percentiles_from_store: histogram::Stored::new(),

                        recent_history: Vec::new(),
                        history_from_store: history::Stored::new(),
                        logs_from_store: fetch::logs::Stored::new(),

                        condition: Err(error.clone()),
                        logs: errors,
                    },
                );
            }
        }
    }

    fn record_data(&mut self, reading: Reading) {
        let (key, _, val) = extract_leaf_info(&reading);
        let time = jiff::Timestamp::now();

        if let Some(info) = self.data.get_mut(&key) {
            if let Some(last_reading) = info.last_at() {
                info.timing += (time - last_reading)
                    .total(Unit::Millisecond)
                    .expect("no calander units involved") as u64
            }
            info.recent_history.push((time, val));
            info.condition = Ok(());
        } else {
            let history = vec![(time, val)];
            self.data.insert(
                key,
                SensorInfo {
                    info: reading.leaf(),
                    reading,
                    timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
                    percentiles_from_store: histogram::Stored::new(),

                    recent_history: history,
                    history_from_store: history::Stored::new(),
                    logs_from_store: fetch::logs::Stored::new(),

                    condition: Ok(()),
                    logs: logs::Logs::new_empty(),
                },
            );
        }
    }

    fn record_missing_data(&mut self, reading: Reading) {
        let (key, _, _) = extract_leaf_info(&reading);
        if self.data.contains_key(&key) {
            return;
        }

        self.data.insert(
            key,
            SensorInfo {
                info: reading.leaf(),
                reading,
                timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
                percentiles_from_store: histogram::Stored::new(),

                recent_history: Vec::new(),
                history_from_store: history::Stored::new(),
                logs_from_store: fetch::logs::Stored::new(),

                condition: Ok(()),
                logs: logs::Logs::new_empty(),
            },
        );
    }

    fn update_tree(&mut self, reading: &Reading, placeholder: IsPlaceholder) {
        let (key, _, _) = extract_leaf_info(reading);

        let mut tree = add_root(reading as &dyn Tree, &mut self.ground);
        let mut tomato = match reading.inner() {
            Item::Leaf(_) => unreachable!("no values at level 0"),
            Item::Node(inner) => inner,
        };
        loop {
            match tomato.inner() {
                Item::Leaf(info) => {
                    let text = if let IsPlaceholder::Yes = placeholder {
                        tomato.name()
                    } else {
                        format!(
                            "{0}: {1:.2$} {3}",
                            tomato.name(),
                            info.val,
                            info.precision(),
                            info.unit
                        )
                    };
                    add_leaf(text, tree, key);
                    return;
                }
                Item::Node(inner) => {
                    tree = add_node(tomato, tree);
                    tomato = inner;
                }
            };
        }
    }

    fn update_tree_err(&mut self, error: &Error) {
        for broken in error.device().info().affects_readings {
            let (key, _, _) = extract_leaf_info(broken);

            let mut tree = add_root(broken as &dyn Tree, &mut self.ground);
            let mut tomato = match broken.inner() {
                Item::Leaf(_) => unreachable!("no values at level 0"),
                Item::Node(inner) => inner,
            };
            loop {
                match tomato.inner() {
                    Item::Leaf(_) => {
                        let text = format!("{}: {}", tomato.name(), error);
                        add_leaf(text, tree, key);
                        break;
                    }
                    Item::Node(inner) => {
                        tree = add_node(tomato, tree);
                        tomato = inner;
                    }
                };
            }
        }
    }
}

pub struct ChartParts<'a> {
    pub reading: ReadingInfo,
    pub data: &'a [(f64, f64)],
}

fn histogram_bars(percentiles: &[log_store::api::Percentile]) -> Vec<Bar<'static>> {
    percentiles
        .into_iter()
        .map(
            |log_store::api::Percentile {
                 bucket_ends,
                 percentile,
                 count_in_bucket,
             }| {
                Bar::default()
                    .value(*count_in_bucket)
                    .text_value(format!("p{percentile}: {}", count_in_bucket))
                    .label(Line::from(format!("..{}", bucket_ends)))
            },
        )
        .collect()
}
