use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use chrono::DateTime;
use cloudreve_api::models::explorer::{file_type, FileResponse};
use uuid::Uuid;

use crate::{
    cfapi::placeholder::LocalFileInfo,
    inventory::{FileMetadata, InventoryDb, MetadataEntry},
};

pub struct CrPlaceholder {
    pub local_file_info: LocalFileInfo,
    local_path: PathBuf,
    drive_id: Uuid,
    file_meta: Option<FileMetadata>,
    mark_no_children: bool,
}

impl CrPlaceholder {
    pub fn new(local_path: impl Into<PathBuf>, _sync_root: PathBuf, drive_id: Uuid) -> Self {
        let local_path = local_path.into();
        Self {
            local_file_info: LocalFileInfo::from_path(&local_path)
                .unwrap_or_else(|_| LocalFileInfo::missing()),
            local_path,
            drive_id,
            file_meta: None,
            mark_no_children: false,
        }
    }

    pub fn with_invalidate_all_range(self, _enable: bool) -> Self {
        self
    }

    pub fn with_mark_no_children(mut self, enable: bool) -> Self {
        self.mark_no_children = enable;
        self
    }

    pub fn with_file_meta(mut self, file_meta: FileMetadata) -> Self {
        self.file_meta = Some(file_meta);
        self
    }

    pub fn delete_placeholder(&self, inventory: Arc<InventoryDb>) -> Result<()> {
        if self.local_file_info.exists {
            if self.local_path.is_dir() {
                std::fs::remove_dir_all(&self.local_path)
                    .context("failed to delete local directory")?;
            } else {
                std::fs::remove_file(&self.local_path).context("failed to delete local file")?;
            }
        }

        let path_str = self
            .local_path
            .to_str()
            .context("failed to convert path to string")?;
        inventory
            .batch_delete_by_path(vec![path_str])
            .context("failed to delete from inventory")?;
        Ok(())
    }

    pub fn commit(&mut self, inventory: Arc<InventoryDb>) -> Result<()> {
        let file_meta = self
            .file_meta
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("File metadata is not set"))?;

        if file_meta.is_folder {
            std::fs::create_dir_all(&self.local_path)
                .context("failed to create local directory")?;
        } else if let Some(parent) = self.local_path.parent() {
            std::fs::create_dir_all(parent).context("failed to create parent directory")?;
        }

        inventory
            .upsert(&MetadataEntry::from(file_meta))
            .context("failed to upsert inventory")?;
        self.local_file_info =
            LocalFileInfo::from_path(&self.local_path).unwrap_or_else(|_| LocalFileInfo::missing());
        Ok(())
    }

    pub fn with_remote_file(mut self, file_info: &FileResponse) -> Self {
        let created_at = DateTime::parse_from_rfc3339(&file_info.created_at)
            .ok()
            .map(|dt| dt.timestamp())
            .unwrap_or_default();

        let updated_at = DateTime::parse_from_rfc3339(&file_info.updated_at)
            .ok()
            .map(|dt| dt.timestamp())
            .unwrap_or_default();

        self.file_meta = Some(FileMetadata {
            id: 0,
            drive_id: self.drive_id,
            local_path: self.local_path.to_string_lossy().to_string(),
            is_folder: file_info.file_type == file_type::FOLDER,
            created_at,
            updated_at,
            size: file_info.size,
            etag: file_info
                .primary_entity
                .as_ref()
                .unwrap_or(&String::new())
                .clone(),
            permissions: file_info.permission.as_ref().unwrap_or(&String::new()).clone(),
            shared: file_info.shared.unwrap_or(false),
            metadata: file_info.metadata.clone().unwrap_or_default(),
            props: None,
            conflict_state: None,
        });
        self
    }

    pub fn update_sync_error_state(&mut self, _sync_error: bool) -> Result<()> {
        Ok(())
    }
}
