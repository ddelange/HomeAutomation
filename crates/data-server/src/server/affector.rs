use std::sync::{Arc, Mutex};

use color_eyre::Result;
use protocol::Affector;
use slotmap::{DefaultKey, SlotMap};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::{Receiver, Sender};

use tracing::{debug, instrument, warn};

#[derive(Debug)]
pub(crate) struct Registration {
    tx: tokio::sync::mpsc::Sender<protocol::Affector>,
    controls: Vec<protocol::Affector>,
}

impl Registration {
    fn update(&mut self, new: Affector) {
        if let Some(curr) = self.controls.iter_mut().find(|a| a.is_same_as(&new)) {
            *curr = new;
        } else {
            debug!("new affector: {new:?} registered");
            self.controls.push(new);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Registar(Arc<Mutex<SlotMap<DefaultKey, Registration>>>);

impl Registar {
    pub(crate) fn register(&self, tx: Sender<Affector>) -> DefaultKey {
        let mut this = self.0.lock().expect("nothing should panic");
        this.insert(Registration {
            tx,
            controls: Vec::new(),
        })
    }

    pub(crate) fn update_affectors(&self, key: DefaultKey, affector: Affector) {
        let mut this = self.0.lock().expect("nothing should panic");
        let registration = this
            .get_mut(key)
            .expect("items are removed when track_and_control_affectors only");
        registration.update(affector)
    }

    pub(crate) fn remove(&self, key: DefaultKey) {
        let mut this = self.0.lock().expect("nothing should panic");
        this.remove(key).expect("things are only removed once");
    }

    pub(crate) fn activate(&self, order: Affector) -> Result<(), Offline> {
        let mut this = self.0.lock().expect("nothing should panic");
        for possible_controller in this
            .iter_mut()
            .map(|(_, reg)| reg)
            .filter(|reg| reg.controls.contains(&order))
        {
            if possible_controller.tx.try_send(order).is_ok() {
                possible_controller.update(order);
                return Ok(());
            }
        }

        Err(Offline)
    }

    pub(crate) fn list(&self) -> Vec<Affector> {
        let this = self.0.lock().expect("nothing should panic");
        this.iter()
            .flat_map(|(_, reg)| reg.controls.iter())
            .cloned()
            .collect()
    }
}

pub struct Offline;

#[instrument(skip_all)]
pub(super) async fn control_affectors(mut writer: OwnedWriteHalf, mut rx: Receiver<Affector>) {
    debug!("controlling newly connected node's affectors");

    while let Some(new_order) = rx.recv().await {
        let buf = new_order.encode();
        if let Err(e) = writer.write_all(&buf).await {
            warn!("Could not send affector order: {e}");
            break;
        }
    }
}
