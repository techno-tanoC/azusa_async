#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub name: String,
    pub total: u64,
    pub size: u64,
    pub canceled: bool,
}

impl Progress {
    pub fn new(name: String) -> Self {
        Progress {
            name,
            total: 0,
            size: 0,
            canceled: false,
        }
    }

    pub fn progress(&mut self, size: u64) {
        self.size += size;
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
