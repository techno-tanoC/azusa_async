use super::lock_copy::*;
use super::table::*;

pub struct App {
    pub table: Table,
    pub lock_copy: LockCopy,
}

impl App {
    pub fn new() -> Self {
        App {
            table: Table::new(),
            lock_copy: LockCopy::new(),
        }
    }
}

