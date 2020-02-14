use reqwest::{header, Response};
use std::path::*;
use tokio::fs::File;
use tokio::prelude::*;

use super::app::App;
use super::progress::Progress;
use super::table::Table;

pub struct Download(reqwest::Client);

impl Download {
    pub fn new() -> Self {
        Download(reqwest::Client::new())
    }

    pub async fn start<P: AsRef<Path>>(&self, app: &App, url: &str, dest: &P, name: &str, ext: &str) {
        let (temp, path) = tempfile::NamedTempFile::new().unwrap().into_parts();
        let mut f: File = temp.into();
        let mut res = self.0.get(url).send().await.unwrap();

        debug!("{:?}", &res);

        if res.status().is_success() {
            let id = Table::generate_id();
            let pg = Progress::new(name.to_string());
            app.table.add(&id, pg).await;
            debug!("url: {:?}, id: {:?}", url, &id);

            if let Some(cl) = Self::content_length(&res) {
                app.table.set_total(&id, cl).await;
            }

            let flag = Self::read_chunks(app, &id, &mut res, &mut f).await;

            if flag {
                app.lock_copy.copy(&path, dest, name, ext).await;
                app.table.delete(&id).await;
            }
        }
    }

    async fn read_chunks(app: &App, id: &str, res: &mut reqwest::Response, f: &mut File) -> bool {
        while let Some(byte) = res.chunk().await.unwrap() {
            if app.table.is_canceled(id).await.unwrap() {
                debug!("canceled id: {:?}", id);
                return false;
            } else {
                app.table.progress(id, byte.as_ref().len() as u64).await;
                f.write_all(&byte).await.unwrap();
            }
        }
        return true;
    }

    fn content_length(res: &Response) -> Option<u64> {
        res.headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok()?.parse().ok())
    }
}
