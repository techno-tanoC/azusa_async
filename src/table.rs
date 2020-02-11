use std::collections::HashMap;
use tokio::sync::Mutex;

use super::progress::Progress;

pub struct Table(Mutex<HashMap<String, Progress>>);

impl Table {
    pub fn new() -> Self {
        Table(Mutex::new(HashMap::new()))
    }

    pub fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    pub async fn add(&self, id: &str, pg: Progress) {
        self.0.lock().await.insert(id.to_string(), pg);
    }

    pub async fn delete(&self, id: &str) {
        self.0.lock().await.remove(id);
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
