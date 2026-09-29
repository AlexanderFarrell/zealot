use std::fmt::Debug;
use std::path::{Path, PathBuf};

use zealot_domain::{
    account::Account,
    media::{File, FileStat},
};

use crate::ports::common::PortError;

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("io error: {err:?}")]
    IoError { err: std::io::Error },
}

pub trait MediaPort: Debug + Sync + Send {
    // TODO: We need to add more from the media_router.go
    fn get(&self, path: &Path, account: &Account) -> Result<Vec<FileStat>, PortError<MediaError>>;
    fn upload(
        &self,
        path: &Path,
        bytes: &Vec<u8>,
        account: &Account,
    ) -> Result<Option<FileStat>, PortError<MediaError>>;
    fn download(
        &self,
        path: &Path,
        account: &Account,
    ) -> Result<Option<File>, PortError<MediaError>>;
    fn rename(
        &self,
        path: &Path,
        new_path: &Path,
        account: &Account,
    ) -> Result<(), PortError<MediaError>>;
    fn delete(&self, path: &Path, account: &Account) -> Result<(), PortError<MediaError>>;

    fn make_folder(&self, path: &Path, account: &Account) -> Result<(), PortError<MediaError>>;
    fn delete_folder(&self, path: &Path, account: &Account) -> Result<(), PortError<MediaError>>;
    fn user_path(&self, account: &Account) -> Result<PathBuf, PortError<MediaError>>;
    fn exists(&self, path: &Path, account: &Account) -> Result<bool, PortError<MediaError>>;
}
