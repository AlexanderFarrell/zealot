use std::error::Error;


#[derive(Debug, thiserror::Error)]
pub enum ServiceError<Domain: Error> {
    #[error("invalid data: {err:?}")]
    DomainError{err: Domain}
}