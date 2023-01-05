use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Weak;
use tokio::sync::{watch::Sender, RwLock};
use tracing::trace;

use crate::{instance::Instance, mc::meta::McMeta};

use super::{InstallProgress, ModLoaderVersion, Modloader, ModloaderVersion};

pub enum InstallStages {
    DownloadingAssets,
    DownloadingLibraries,
    ExtractingNatives,
}

#[derive(Debug)]
pub struct VanillaModLoader {
    mc_version: ModLoaderVersion,
    instance_ref: Weak<RwLock<Instance>>,
}

#[async_trait]
impl Modloader for VanillaModLoader {
    type Stages = InstallStages;

    fn new(mc_version: ModLoaderVersion, instance_ref: Weak<RwLock<Instance>>) -> Self {
        VanillaModLoader {
            mc_version,
            instance_ref,
        }
    }
    async fn install(&self, progress_send: Sender<InstallProgress<InstallStages>>) -> Result<()> {
        let mc_version = &self.mc_version;
        // TODO: REMOVE HARDCODED
        let base_dir = std::env::current_dir().unwrap().join("MC_TEST");

        let meta = McMeta::download_meta().await?;

        let version_meta = meta
            .versions
            .iter()
            .find(|version| &version.id == mc_version)
            .ok_or_else(|| anyhow::anyhow!("Cannot find version"))?
            .get_version_meta(&base_dir)
            .await?;

        let mut downloads = vec![];

        let asset_index = version_meta
            .get_asset_index_meta(&base_dir)
            .await
            .context("Failed to get asset index meta")?;

        let assets = asset_index
            .get_asset_downloads(&base_dir)
            .await
            .context("Failed to download assets")?;
        downloads.extend(assets);

        let libs = version_meta
            .get_allowed_libraries(&base_dir)
            .await
            .context("Failed to get libraries")?;
        downloads.extend(libs);

        let client = version_meta
            .get_jar_client(&base_dir)
            .await
            .context("Failed to get client download")?;
        downloads.push(client);

        let total_size = downloads
            .iter()
            .map(|download| download.size.unwrap_or(0))
            .sum::<u64>()
            / (1024 * 1024);

        let (progress, mut progress_handle) = tokio::sync::watch::channel(carbon_net::Progress {
            current_count: 0,
            current_size: 0,
        });
        let length = &downloads.len();

        let handle = tokio::spawn(async move {
            carbon_net::download_multiple(downloads, progress).await?;
            Ok::<(), anyhow::Error>(())
        });

        let instance_ref = self.instance_ref.upgrade().unwrap();
        let instance = instance_ref.read().await;

        while progress_handle.changed().await.is_ok() {
            trace!(
                "Progress: {} / {} - {} / {} MB",
                progress_handle.borrow().current_count,
                length - 1,
                progress_handle.borrow().current_size,
                total_size
            );
        }

        handle.await??;

        version_meta
            .extract_natives(&base_dir, &instance.name)
            .await?;

        Ok(())
    }
    fn remove(&self) -> Result<()> {
        Ok(())
    }
    fn verify(&self) -> Result<()> {
        Ok(())
    }
    fn get_version(&self) -> ModloaderVersion {
        self.mc_version.clone()
    }
}