

#[derive(Debug, thiserror::Error)]
pub enum StringsError {
    #[error("cannot be empty")]
    IsEmpty,
}