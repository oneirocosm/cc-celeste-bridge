use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::time::delay_queue::DelayQueue;

#[derive(Default)]
pub struct ToTcp {
    pub queue: Arc<Mutex<DelayQueue<String>>>,
}
