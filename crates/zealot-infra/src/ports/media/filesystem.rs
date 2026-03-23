use std::path::PathBuf;

use zealot_app::{config::ZealotConfig, ports::media::MediaPort};

#[derive(Debug)]
pub struct MediaFilesystemPort {
    path: std::path::PathBuf,
}

impl MediaFilesystemPort {
    pub fn new(config: &ZealotConfig) -> Self {
        Self {
            path: PathBuf::from(config.media_path.clone()),
        }
    }
}

impl MediaPort for MediaFilesystemPort {
    fn get(
        &self,
        path: &std::path::Path,
        account: &zealot_domain::account::Account,
    ) -> Result<
        Vec<zealot_domain::media::FileStat>,
        zealot_app::ports::common::PortError<zealot_app::ports::media::MediaError>,
    > {
        todo!()
    }

    fn upload(
        &self,
        path: &std::path::Path,
        bytes: &Vec<u8>,
        account: &zealot_domain::account::Account,
    ) -> Result<
        Option<zealot_domain::media::FileStat>,
        zealot_app::ports::common::PortError<zealot_app::ports::media::MediaError>,
    > {
        todo!()
    }

    fn download(
        &self,
        path: &std::path::Path,
        account: &zealot_domain::account::Account,
    ) -> Result<
        Option<zealot_domain::media::File>,
        zealot_app::ports::common::PortError<zealot_app::ports::media::MediaError>,
    > {
        todo!()
    }

    fn rename(
        &self,
        path: &std::path::Path,
        new_path: &std::path::Path,
        account: &zealot_domain::account::Account,
    ) -> Result<(), zealot_app::ports::common::PortError<zealot_app::ports::media::MediaError>>
    {
        todo!()
    }

    fn delete(
        &self,
        path: &std::path::Path,
        account: &zealot_domain::account::Account,
    ) -> Result<(), zealot_app::ports::common::PortError<zealot_app::ports::media::MediaError>>
    {
        todo!()
    }

    fn make_folder(
        &self,
        path: &std::path::Path,
        account: &zealot_domain::account::Account,
    ) -> Result<(), zealot_app::ports::common::PortError<zealot_app::ports::media::MediaError>>
    {
        todo!()
    }

    fn delete_folder(
        &self,
        path: &std::path::Path,
        account: &zealot_domain::account::Account,
    ) -> Result<(), zealot_app::ports::common::PortError<zealot_app::ports::media::MediaError>>
    {
        todo!()
    }

    fn user_path(
        &self,
        account: &zealot_domain::account::Account,
    ) -> Result<PathBuf, zealot_app::ports::common::PortError<zealot_app::ports::media::MediaError>>
    {
        todo!()
    }

    fn exists(
        &self,
        path: &std::path::Path,
        account: &zealot_domain::account::Account,
    ) -> Result<bool, zealot_app::ports::common::PortError<zealot_app::ports::media::MediaError>>
    {
        todo!()
    }
}
