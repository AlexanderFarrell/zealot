use std::sync::Arc;

use crate::ports::{
    events::EventPort,
    media::MediaPort,
    password::PasswordPort,
    rule_runner::RuleRunnerPort,
};

pub mod clock;
pub mod common;
pub mod desktop;
pub mod encryption;
pub mod events;
pub mod mail_sender;
pub mod media;
pub mod password;
pub mod rule_runner;

#[derive(Debug, Clone)]
pub struct ZealotPorts {
    pub media:       Arc<dyn MediaPort>,
    pub password:    Arc<dyn PasswordPort>,
    pub events:      Arc<dyn EventPort>,
    pub rule_runner: Arc<dyn RuleRunnerPort>,
}
