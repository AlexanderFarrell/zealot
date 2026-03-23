#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("error on database: {err:?}")]
    DatabaseError { err: String },

    #[error("error reaching database")]
    NotReachable,

    #[error("not found")]
    NotFound,
}

impl From<sqlx::Error> for RepoError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => RepoError::NotFound,
            sqlx::Error::PoolTimedOut => RepoError::NotReachable,
            other => RepoError::DatabaseError {
                err: other.to_string(),
            },
        }
    }
}
