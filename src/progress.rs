use bytes::Bytes;
use tempfile::{NamedTempFile, TempPath};
use tokio::fs::File;
use tokio::prelude::*;

use super::error::Result;
use super::item::Item;

#[derive(Debug)]
pub struct Progress {
    pub name: String,
    pub total: u64,
    pub size: u64,
    pub canceled: bool,
    file: File,
    path: TempPath,
}

impl Progress {
    pub fn new(name: String) -> Result<Self> {
        let (file, path) = NamedTempFile::new()?.into_parts();
        Ok(Progress {
            name,
            total: 0,
            size: 0,
            canceled: false,
            file: file.into(),
            path,
        })
    }

    pub async fn flush(&mut self) -> Result<()> {
        Ok(self.file.sync_all().await?)
    }

    pub fn path(&self) -> &TempPath {
        &self.path
    }

    pub fn make_item(&self, id: &str) -> Item {
        Item {
            id: id.to_string(),
            name: self.name.clone(),
            total: self.total.clone(),
            size: self.size.clone(),
            canceled: self.canceled.clone(),
        }
    }

    pub async fn progress(&mut self, bytes: Bytes) -> Result<()> {
        self.file.write_all(&bytes).await?;
        self.size += bytes.len() as u64;
        Ok(())
    }

    pub fn set_total(&mut self, total: u64) {
        self.total = total;
    }

    pub fn is_canceled(&self) -> bool {
        self.canceled
    }

    pub fn cancel(&mut self) {
        self.canceled = true;
    }
}
