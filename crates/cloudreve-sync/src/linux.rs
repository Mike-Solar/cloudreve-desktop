use std::path::Path;

use anyhow::{Context, Result};

use crate::platform::{PlatformCapabilities, SyncModeCapability};

#[derive(Debug, Clone)]
pub enum LinuxSyncBackendKind {
    FullSync,
    KdePlaceholders,
}

#[derive(Debug, Clone)]
pub struct LinuxSyncBackend {
    pub kind: LinuxSyncBackendKind,
    pub capabilities: PlatformCapabilities,
}

impl LinuxSyncBackend {
    pub fn current() -> Self {
        let capabilities = PlatformCapabilities::current();
        Self::from_capabilities(capabilities)
    }

    pub fn prepare_current() -> Result<Self> {
        let mut capabilities = PlatformCapabilities::current();
        if capabilities.desktop_environment == crate::platform::DesktopEnvironment::Kde {
            let kde_backend = crate::kde::KdePlaceholderBackend::ensure_installed()?;
            capabilities.kde_placeholder_backend = Some(kde_backend);
            capabilities = PlatformCapabilities::current();
        }

        Ok(Self::from_capabilities(capabilities))
    }

    fn from_capabilities(capabilities: PlatformCapabilities) -> Self {
        let kind = match capabilities.sync_mode {
            SyncModeCapability::KdePlaceholders => LinuxSyncBackendKind::KdePlaceholders,
            _ => LinuxSyncBackendKind::FullSync,
        };

        Self { kind, capabilities }
    }

    pub fn log_selection(&self, drive_id: &str) {
        tracing::info!(
            target: "linux::sync_backend",
            id = %drive_id,
            os = %self.capabilities.os,
            desktop = ?self.capabilities.desktop_environment,
            sync_mode = ?self.capabilities.sync_mode,
            reason = %self.capabilities.reason,
            "Resolved Linux sync backend"
        );
    }

    pub fn prepare_sync_root(&self, sync_path: &Path) -> Result<()> {
        match self.kind {
            LinuxSyncBackendKind::FullSync => {
                std::fs::create_dir_all(sync_path).context("failed to create sync directory")?;
            }
            LinuxSyncBackendKind::KdePlaceholders => {
                std::fs::create_dir_all(sync_path)
                    .context("failed to create KDE placeholder sync directory")?;
                tracing::info!(
                    target: "linux::sync_backend",
                    path = %sync_path.display(),
                    "KDE placeholder backend selected"
                );
            }
        }

        Ok(())
    }

    pub fn should_run_initial_full_sync(&self) -> bool {
        match self.kind {
            LinuxSyncBackendKind::FullSync => true,
            LinuxSyncBackendKind::KdePlaceholders => true,
        }
    }
}
