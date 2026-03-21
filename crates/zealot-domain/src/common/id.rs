#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(i64);

#[derive(Debug, thiserror::Error)]
pub enum IdError {
    #[error("id cannot be negative")]
    NegativeNumber,
}

impl TryFrom<i64> for Id {
    type Error = IdError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(IdError::NegativeNumber);
        }
        Ok(Self(value))
    }
}

impl From<Id> for i64 {
    fn from(value: Id) -> Self {
        value.0
    }
}
