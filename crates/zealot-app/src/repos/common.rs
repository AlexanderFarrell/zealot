#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("error on database: {err:?}")]
    DatabaseError { err: String },

    #[error("error reaching database")]
    NotReachable,

    #[error("not found")]
    NotFound,

    /// A unique constraint rejected the write (for example a duplicate key).
    #[error("already exists")]
    Conflict,
}

impl From<sqlx::Error> for RepoError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => RepoError::NotFound,
            sqlx::Error::PoolTimedOut => RepoError::NotReachable,
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                RepoError::Conflict
            }
            other => RepoError::DatabaseError {
                err: other.to_string(),
            },
        }
    }
}
