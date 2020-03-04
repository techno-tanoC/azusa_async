use std::borrow::Cow;
use std::path::*;
use tokio::sync::Mutex;
use tokio::fs;

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
        let _lock = self.0.lock().await;
        let fresh = Self::fresh_name(&path, &name, &ext);
        fs::copy(&from, &fresh).await.unwrap();
        Self::change_owner(&fresh).unwrap();
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

    fn change_owner<P: AsRef<Path>>(path: &P) -> Result<(), String> {
        use std::os::unix::ffi::OsStrExt;
        use std::ffi::CString;

        let bytes = path.as_ref().as_os_str().as_bytes();
        let s = CString::new(bytes).unwrap();
        let ret = unsafe {
            libc::chown(s.as_ptr(), 1000, 1000)
        };
        if ret == -1 {
            Err(format!("failed chown: {:?}", s))
        } else {
            Ok(())
        }
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
