use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum PortError<T: Error> {
    #[error("{err:?}")]
    OtherError { err: T },
}
