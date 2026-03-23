use std::fmt::Debug;

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

pub trait MetaRepo: Debug + Send + Sync {
    // TODO: Figure out this structure.
     fn backup(&self) -> Result<(), RepoError>;

     fn download(&self, target: MetaDownloadTarget) -> Result<(), RepoError>;

    // TODO: How will we upload everything?
     fn upload(&self) -> Result<(), RepoError>;
}
