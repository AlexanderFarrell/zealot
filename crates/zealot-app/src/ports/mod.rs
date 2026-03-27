use std::sync::Arc;

use crate::ports::{media::MediaPort, password::PasswordPort};

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
    pub media: Arc<dyn MediaPort>,
    pub password: Arc<dyn PasswordPort>,
}
