use std::path::{Path, PathBuf};
use crate::instance::{Instance, InstanceStatus};
use log::trace;
use thiserror::Error;
use crate::{instance, try_path_fmt};
use crate::instance::consts::MINECRAFT_PACKAGE_RELATIVE_PATH;
use crate::instance::write::InstanceWriteError::{MinecraftPackageAlreadyExist, MinecraftPackageIsAFile};

#[derive(Error, Debug)]
pub enum InstanceWriteError{

    #[error("error happened while trying to write configuration file for instance : {0}\n")]
    InstanceConfigurationWritingError(#[from] instance::configuration::ConfigurationFileParsingError),

    #[error("error happened while trying to write configuration file for instance \n")]
    PathNotSpecified(),

    #[error("minecraft package already exist and is a file \n")]
    MinecraftPackageIsAFile(),

    #[error("minecraft package already exist \n")]
    MinecraftPackageAlreadyExist(),

    #[error("io error raised while writing instance : {0}\n")]
    IoError(#[from] std::io::Error),

}

type InstanceWriteResult = Result<Instance, InstanceWriteError>;

pub async fn write_at<T: AsRef<Path> + Sync>(instance: Instance, path: &T) -> InstanceWriteResult {
    let base_path = path.as_ref();
    trace!("writing instance at {}", try_path_fmt!(base_path));

    let minecraft_package_folder = &PathBuf::from(base_path).join(instance::consts::MINECRAFT_PACKAGE_RELATIVE_PATH);

    async fn make_minecraft_package_folder_at(folder_path: &Path) -> std::io::Result<()> {
        trace!("making minecraft package folder at {}", try_path_fmt!(folder_path));
        tokio::fs::create_dir(folder_path).await
    }

    let minecraft_package_folder_exist = minecraft_package_folder.exists();
    let minecraft_package_folder_is_dir = minecraft_package_folder.is_dir();

    match (&instance.persistence_status, minecraft_package_folder_exist, minecraft_package_folder_is_dir) {
        (InstanceStatus::Persisted(instance_path), false, false) if !instance_path.starts_with(minecraft_package_folder) => {
            let old_instance_minecraft_package_dir = instance_path.join(MINECRAFT_PACKAGE_RELATIVE_PATH);
            trace!("coping old instance minecraft package dir from {} to {}", try_path_fmt!(old_instance_minecraft_package_dir), try_path_fmt!(minecraft_package_folder));
            tokio::fs::copy(old_instance_minecraft_package_dir, minecraft_package_folder).await?;
        },
        (_, false, false) => make_minecraft_package_folder_at(minecraft_package_folder).await?,
        (_, true, false) => Err(MinecraftPackageIsAFile())?,
        (_, true, true) => Err(MinecraftPackageAlreadyExist())?,
        _ => ()
    }

    let instance_configuration_file_path = PathBuf::from(base_path).join(instance::consts::CONFIGURATION_FILE_RELATIVE_PATH);
    instance::configuration::write_in_file(&instance, &instance_configuration_file_path).await?;

    Ok(instance.mutate_persistence_status(InstanceStatus::Persisted(base_path.to_path_buf())))
}
