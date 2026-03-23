use std::{fmt::Debug, path::{Path, PathBuf}};

use zealot_domain::{account::Account, media::{File, FileStat}};

use crate::repos::common::RepoError;



pub trait MediaRepo: Debug + Send + Sync {
    // TODO: We need to add more from the media_router.go
    fn get(&self, path: Path, account: &Account) -> Result<Vec<FileStat>, RepoError>;
    fn upload(&self, path: Path, bytes: &Vec<u8>, account: &Account) -> Result<Option<FileStat>, RepoError>;
    fn download(&self, path: Path, account: &Account) -> Result<Option<File>, RepoError>;
    fn rename(&self, path: Path, new_path: Path, account: &Account) -> Result<(), RepoError>;
    fn delete(&self, path: Path, account: &Account) -> Result<(), RepoError>;

    fn make_folder(&self, path: Path, account: &Account) -> Result<(), RepoError>;
    fn delete_folder(&self, path: Path, account: &Account) -> Result<(), RepoError>;
    fn user_path(&self, account: &Account) -> Result<PathBuf, RepoError>;
    fn exists(&self, path: Path, account: &Account) -> Result<bool, RepoError>;
}
