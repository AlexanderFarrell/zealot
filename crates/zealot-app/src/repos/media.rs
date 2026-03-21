use std::{path::{Path, PathBuf}};

use zealot_domain::{account::Account, media::{File, FileStat, MediaError}};

use crate::repos::common::RepoError;



pub trait MediaRepo {
    // TODO: We need to add more from the media_router.go
    fn get(path: Path, account: &Account) -> Result<Vec<FileStat>, RepoError<MediaError>>;
    fn upload(path: Path, bytes: &Vec<u8>, account: &Account) -> Result<Option<FileStat>, RepoError<MediaError>>;
    fn download(path: Path, account: &Account) -> Result<Option<File>, RepoError<MediaError>>;
    fn rename(path: Path, new_path: Path, account: &Account) -> Result<(), RepoError<MediaError>>;
    fn delete(path: Path, account: &Account) -> Result<(), RepoError<MediaError>>;

    fn make_folder(path: Path, account: &Account) -> Result<(), RepoError<MediaError>>;
    fn delete_folder(path: Path, account: &Account) -> Result<(), RepoError<MediaError>>;
    fn user_path(account: &Account) -> Result<PathBuf, RepoError<MediaError>>;
    fn exists(path: Path, account: &Account) -> Result<bool, RepoError<MediaError>>;
}