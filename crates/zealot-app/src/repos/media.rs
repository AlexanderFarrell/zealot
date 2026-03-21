use std::{path::{Path, PathBuf}};

use zealot_domain::{account::Account, media::{File, FileStat, MediaError}};

use crate::repos::common::RepoError;



pub trait MediaRepo {
    // TODO: We need to add more from the media_router.go
    fn get(&self, path: Path, account: &Account) -> Result<Vec<FileStat>, RepoError<MediaError>>;
    fn upload(&self, path: Path, bytes: &Vec<u8>, account: &Account) -> Result<Option<FileStat>, RepoError<MediaError>>;
    fn download(&self, path: Path, account: &Account) -> Result<Option<File>, RepoError<MediaError>>;
    fn rename(&self, path: Path, new_path: Path, account: &Account) -> Result<(), RepoError<MediaError>>;
    fn delete(&self, path: Path, account: &Account) -> Result<(), RepoError<MediaError>>;

    fn make_folder(&self, path: Path, account: &Account) -> Result<(), RepoError<MediaError>>;
    fn delete_folder(&self, path: Path, account: &Account) -> Result<(), RepoError<MediaError>>;
    fn user_path(&self, account: &Account) -> Result<PathBuf, RepoError<MediaError>>;
    fn exists(&self, path: Path, account: &Account) -> Result<bool, RepoError<MediaError>>;
}
