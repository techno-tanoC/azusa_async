use std::borrow::Cow;
use std::path::*;
use tokio::sync::Mutex;
use tokio::task;

pub struct LockCopy(Mutex<()>);

impl LockCopy {
    pub fn new() -> Self {
        LockCopy(Mutex::new(()))
    }

    pub async fn copy<P, Q>(&self, from: &P, path: &Q, name: &str, ext: &str)
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let _ = self.0.lock().await;
        let from = from.as_ref().to_path_buf();
        let path = path.as_ref().to_path_buf();
        let name = name.to_string();
        let ext = ext.to_string();

        task::spawn_blocking(move || {
            let fresh = Self::fresh_name(&path, &name, &ext);
            std::fs::copy(&from, &fresh).expect("failed to copy");
            Self::change_owner(&fresh);
        }).await.unwrap();
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

    fn change_owner<P: AsRef<Path>>(path: &P) {
        use std::os::unix::ffi::OsStrExt;
        use std::ffi::CString;

        let bytes = path.as_ref().as_os_str().as_bytes();
        let s = CString::new(bytes).unwrap();
        unsafe {
            libc::chown(s.as_ptr(), 1000, 1000);
        }
    }
}
