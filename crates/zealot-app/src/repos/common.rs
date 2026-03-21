use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepoError<T: Error> {
    #[error("error on database: {err:?}")]
    DatabaseError{err: String},

    #[error("domain logic error: {err:?}")]
    DomainError{err: T},

    #[error("resource not found")]
    NotFound,
}