use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    name: String,
    total: u64,
    size: u64,
    canceled: bool
}

impl Progress {
    pub fn new(name: String) -> Self {
        Progress {
            name,
            total: 0,
            size: 0,
            canceled: false
        }
    }

    pub fn progress(&mut self, size: u64) {
        self.size += size;
    }

    pub fn set_total(&mut self, total: u64) {
        self.total = total;
    }

    pub fn cancel(&mut self) {
        self.canceled = true;
    }
}
