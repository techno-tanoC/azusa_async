use std::path::{Path, PathBuf};
use tokio::fs::File;

use super::lock_copy::LockCopy;
use super::table::Table;
use super::download::Download;

pub struct App {
    pub path: PathBuf,
    pub table: Table<File>,
    pub lock_copy: LockCopy,
    pub client: Download,
}

impl App {
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        App {
            path: path.as_ref().to_path_buf(),
            table: Table::new(),
            lock_copy: LockCopy::new(),
            client: Download::new(),
        }
    }
}
