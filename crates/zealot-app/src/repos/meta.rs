use crate::repos::common::RepoError;


#[derive(Debug, thiserror::Error)]
pub enum MetaRepoError {

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetaDownloadTarget {
    Attributes,
    ItemTypes,
    Repeats,
    Rules,
}

pub trait MetaRepo {
    // TODO: Figure out this structure.
    async fn backup() -> Result<(), RepoError<MetaRepoError>>;

    async fn download(target: MetaDownloadTarget) -> Result<(), RepoError<MetaRepoError>>;

    // TODO: How will we upload everything?
    async fn upload() -> Result<(), RepoError<MetaRepoError>>;
}
