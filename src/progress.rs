use tokio::prelude::*;
use tokio::io::AsyncSeek;

use super::error::Result;

#[derive(Debug)]
pub struct Progress<F> {
    pub name: String,
    pub total: u64,
    pub size: u64,
    pub canceled: bool,
    pub(crate) file: F
}

impl<F: AsyncRead + AsyncWrite + AsyncSeek + std::marker::Unpin> Progress<F> {
    pub fn new(name: String, file: F) -> Self {
        Progress {
            name,
            total: 0,
            size: 0,
            canceled: false,
            file,
        }
    }

    pub async fn write<B: AsRef<[u8]>>(&mut self, data: &B) -> Result<()> {
        let data = data.as_ref();
        self.size += data.len() as u64;
        Ok(self.file.write_all(&data).await?)
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
