use indexmap::IndexMap;
use tokio::sync::Mutex;

use super::progress::Progress;

pub struct Table(Mutex<IndexMap<String, Progress>>);

impl Table {
    pub fn new() -> Self {
        Table(Mutex::new(IndexMap::new()))
    }

    pub fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    pub async fn add(&self, id: &str, pg: Progress) {
        self.0.lock().await.insert(id.to_string(), pg);
    }

    pub async fn delete(&self, id: &str) {
        self.0.lock().await.shift_remove(id);
    }

    pub async fn set_total(&self, id: &str, total: u64) {
        self.0.lock().await.get_mut(id).map(|pg| {
            pg.set_total(total);
        });
    }

    pub async fn progress(&self, id: &str, size: u64) {
        self.0.lock().await.get_mut(id).map(|pg| {
            pg.progress(size);
        });
    }

    pub async fn is_canceled(&self, id: &str) -> Option<bool> {
        self.0.lock().await.get(id).map(|pg| {
            pg.is_canceled()
        })
    }

    pub async fn cancel(&self, id: &str) {
        self.0.lock().await.get_mut(id).map(|pg| {
            pg.cancel();
        });
    }

    pub async fn to_vec(&self) -> Vec<(String, Progress)> {
        self.0.lock().await.iter().map(|(k, v)| {
            (k.clone(), v.clone())
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new() {
        let table = Table::new();
        assert_eq!(*table.0.lock().await, IndexMap::new());
    }

    #[test]
    fn test_generate_id() {
        let id = Table::generate_id();
        assert!(id.len() == 36);
    }

    #[tokio::test]
    async fn test_simulate() {
        let table = Table::new();
        let id = Table::generate_id();
        let pg = Progress::new("hello".to_string());

        table.add(&id, pg).await;
        table.set_total(&id, 1000).await;
        table.progress(&id, 100).await;
        table.progress(&id, 100).await;
        table.progress("non exists", 100).await;
        table.cancel(&id).await;

        let vec = table.to_vec().await;
        let mut ans = Progress::new("hello".to_string());
        ans.set_total(1000);
        ans.progress(200);
        ans.cancel();
        assert_eq!(vec, vec![(id.clone(), ans)]);

        table.delete(&id).await;
        assert_eq!(*table.0.lock().await, IndexMap::new());
    }
}
