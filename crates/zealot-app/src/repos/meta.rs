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
     fn backup(&self) -> Result<(), RepoError<MetaRepoError>>;

     fn download(&self, target: MetaDownloadTarget) -> Result<(), RepoError<MetaRepoError>>;

    // TODO: How will we upload everything?
     fn upload(&self) -> Result<(), RepoError<MetaRepoError>>;
}
