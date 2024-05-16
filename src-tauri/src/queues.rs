use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::time::delay_queue::DelayQueue;

#[derive(Default)]
pub struct ToTcpA {
    pub queue: Arc<Mutex<DelayQueue<String>>>,
}

pub struct ToTcp {
    pub tx: Arc<Mutex<Sender<String>>>,
    pub rx: Arc<Mutex<Receiver<String>>>,
}

impl ToTcp {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}
