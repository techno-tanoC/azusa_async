use std::path::Path;

use super::lock_copy::LockCopy;
use super::table::Table;

pub struct App {
    pub table: Table,
    pub lock_copy: LockCopy,
    pub client: reqwest::Client,
}

impl App {
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("ClientBuilder::build()");
        App {
            table: Table::new(),
            lock_copy: LockCopy::new(path),
            client,
        }
    }
}
