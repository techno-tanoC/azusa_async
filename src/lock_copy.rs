use std::borrow::Cow;
use std::path::*;
use std::io::SeekFrom;
use tokio::sync::Mutex;
use tokio::fs::File;
use tokio::prelude::*;
use tokio::io::{*, AsyncSeek};

use super::error::Result;

pub struct LockCopy {
    mutex: Mutex<()>,
    path: PathBuf,
}

impl LockCopy {
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        LockCopy {
            mutex: Mutex::new(()),
            path: path.as_ref().to_path_buf()
        }
    }

    pub async fn copy<F>(&self, from: &mut F, name: &str, ext: &str) -> Result<()>
        where
            F: AsyncRead + AsyncSeek + Unpin
    {
        let _lock = self.mutex.lock().await;
        let fresh = Self::fresh_name(&self.path, &name, &ext);
        let mut to = BufWriter::new(File::create(&fresh).await?);
        Self::rewind_and_copy(from, &mut to).await?;
        Ok(())
    }

    async fn rewind_and_copy<F, G>(from: &mut F, to: &mut G) -> Result<()>
        where
            F: AsyncRead + AsyncSeek + Unpin,
            G: AsyncWrite + Unpin
    {
        from.seek(SeekFrom::Start(0)).await?;
        io::copy(from, to).await?;
        Ok(())
    }

    fn fresh_name<P: AsRef<Path>>(path: &P, name: &str, ext: &str) -> PathBuf {
        let mut i = 0;
        loop {
            let name = Self::build_name(name, i, ext);
            let candidate = path.as_ref().join(name);
            if candidate.exists() {
                i += 1;
            } else {
                return candidate.to_path_buf();
            }
        }
    }

    fn build_name(name: &str, count: u64, ext: &str) -> String {
        let count: Cow<'_, _> = if count >= 1 {
            format!("({})", count).into()
        } else {
            "".into()
        };

        let ext: Cow<'_, _> = if ext.is_empty() {
            "".into()
        } else {
            format!(".{}", ext).into()
        };

        name.to_string() + &count + &ext
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_name() {
        let fresh = LockCopy::fresh_name(&".", "dummy", "toml");
        assert_eq!(fresh.to_string_lossy(), "./dummy.toml".to_string());

        let fresh = LockCopy::fresh_name(&".", "Cargo", "toml");
        assert_eq!(fresh.to_string_lossy(), "./Cargo(1).toml".to_string());
    }

    #[test]
    fn test_build_name() {
        let name = LockCopy::build_name("hello", 0, "jpg");
        assert_eq!(name, "hello.jpg");

        let name = LockCopy::build_name("hello", 1, "jpg");
        assert_eq!(name, "hello(1).jpg");

        let name = LockCopy::build_name("hello", 0, "");
        assert_eq!(name, "hello");

        let name = LockCopy::build_name("hello", 1, "");
        assert_eq!(name, "hello(1)");
    }
}
