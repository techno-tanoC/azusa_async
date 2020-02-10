use std::path::*;
use tokio::sync::Mutex;
use tokio::task;

pub struct LockCopy(Mutex<()>);

impl LockCopy {
    pub fn new() -> Self {
        LockCopy(Mutex::new(()))
    }

    pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: &P,
        path: &Q,
        name: &str,
        ext: &str,
    ) {
        let _ = self.0.lock().await;
        let from = from.as_ref().to_path_buf();
        let path = path.as_ref().to_path_buf();
        let name = name.to_string().clone();
        let ext = ext.to_string().clone();

        task::spawn_blocking(move || {
            let fresh = Self::fresh_name(&path, &name, &ext);
            std::fs::copy(&from, &fresh).expect("failed to copy");
        })
        .await
        .unwrap();
    }

    fn fresh_name<P: AsRef<Path>>(path: &P, name: &str, ext: &str) -> PathBuf {
        let mut i = 0;
        loop {
            let candidate = path.as_ref().join(Self::build_name(name, i, ext));
            if candidate.exists() {
                i += 1;
            } else {
                return candidate.to_path_buf();
            }
        }
    }

    fn build_name(name: &str, count: u64, ext: &str) -> String {
        let count = if count >= 1 {
            count.to_string()
        } else {
            "".to_string()
        };

        let ext = if ext.is_empty() {
            "".to_string()
        } else {
            format!(".{}", ext)
        };

        name.to_string() + &count + &ext
    }
}
