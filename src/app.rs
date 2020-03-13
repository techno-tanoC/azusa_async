use std::path::Path;

use super::lock_copy::LockCopy;
use super::table::Table;
use super::download::Download;

pub struct App {
    pub table: Table,
    pub lock_copy: LockCopy,
    pub client: Download,
}

impl App {
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        App {
            table: Table::new(),
            lock_copy: LockCopy::new(path),
            client: Download::new(),
        }
    }
}
