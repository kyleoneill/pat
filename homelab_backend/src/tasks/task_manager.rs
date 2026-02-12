use crate::db::PatDatabase;
use tokio::sync::watch::{Receiver, Sender};

pub struct TaskManager {
    #[allow(dead_code)]
    db_handle: PatDatabase,
    log_creation_send_channel: Sender<&'static str>,
    log_creation_receive_channel: Receiver<&'static str>, // This should return a result with some data?
}

impl TaskManager {
    pub fn new(
        db_handle: PatDatabase,
        log_creation_send_channel: Sender<&'static str>,
        log_creation_receive_channel: Receiver<&'static str>,
    ) -> Self {
        Self {
            db_handle,
            log_creation_send_channel,
            log_creation_receive_channel,
        }
    }
}

impl TaskManager {
    pub async fn run_logs_task(&mut self) {
        // Manually run the recurring task to generate logs and wait for it to finish
        // TODO: I should do something with the results here
        let _ = self.log_creation_send_channel.send("run task");
        let _ = self.log_creation_receive_channel.changed().await;
    }
}
