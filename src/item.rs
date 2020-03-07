use serde::Serialize;

use super::progress::Progress;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub total: u64,
    pub size: u64,
    pub canceled: bool,
}

impl Item {
    pub fn from_progress<F>(id: String, pg: &Progress<F>) -> Item {
        Item {
            id,
            name: pg.name.clone(),
            total: pg.total,
            size: pg.size,
            canceled: pg.canceled,
        }
    }
}
