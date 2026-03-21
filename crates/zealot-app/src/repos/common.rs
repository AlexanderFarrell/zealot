use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("error on database: {err:?}")]
    DatabaseError{err: String},

    #[error("error reaching database")]
    NotReachable,
}