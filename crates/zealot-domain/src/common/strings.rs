#[derive(Debug, thiserror::Error)]
pub enum StringsError {
    #[error("cannot be empty")]
    IsEmpty,
}

pub struct StringsHelper;

impl StringsHelper {
    pub fn is_within(min: usize, max: usize, s: &str) -> bool {
        if s.len() < min {
            return false;
        }

        if s.len() > max {
            return false;
        }

        return true;
    }
}