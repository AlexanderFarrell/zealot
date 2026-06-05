use tokio::sync::broadcast;
use zealot_app::ports::events::{EventPort, ZealotEvent, is_inside_rule_execution};

#[derive(Debug, Clone)]
pub struct BroadcastEventPort {
    tx: broadcast::Sender<ZealotEvent>,
}

impl BroadcastEventPort {
    pub fn new(tx: broadcast::Sender<ZealotEvent>) -> Self {
        Self { tx }
    }
}

impl EventPort for BroadcastEventPort {
    fn emit(&self, event: ZealotEvent) {
        if is_inside_rule_execution() {
            return;
        }
        let _ = self.tx.send(event);
    }
}
