use reqwest::{header, Response};
use std::path::*;
use tokio::fs::File;
use tokio::prelude::*;

use super::app::App;
use super::progress::Progress;
use super::table::Table;
use super::error::Result;

pub struct Download(reqwest::Client);

impl Download {
    pub fn new() -> Self {
        Download(reqwest::Client::new())
    }

    pub async fn start<P: AsRef<Path>>(&self, app: &App, url: &str, dest: &P, name: &str, ext: &str) -> Result<()> {
        let mut temp: File = tempfile::tempfile()?.into();
        let mut res = self.0.get(url).send().await?;

        debug!("{:?}", &res);

        if res.status().is_success() {
            let id = Table::generate_id();
            let pg = Progress::new(name.to_string());
            app.table.add(&id, pg).await;
            debug!("name: {:?}, url: {:?}, id: {:?}", &name, url, &id);

            if let Some(cl) = Self::content_length(&res) {
                app.table.set_total(&id, cl).await;
            }

            let flag = Self::read_chunks(app, &id, &mut res, &mut temp).await?;

            if flag {
                app.lock_copy.copy(&mut temp, dest, name, ext).await?;
            }

            app.table.delete(&id).await;
        }
        Ok(())
    }

    async fn read_chunks<W: AsyncWrite + Unpin>(app: &App, id: &str, res: &mut reqwest::Response, f: &mut W) -> Result<bool> {
        while let Some(byte) = res.chunk().await? {
            match app.table.is_canceled(id).await {
                Some(true) => {
                    debug!("canceled id: {:?}", id);
                    return Ok(false);
                },
                Some(false) => {
                    app.table.progress(id, byte.as_ref().len() as u64).await;
                    f.write_all(&byte).await?;
                },
                None => {
                    error!("read_chunks id not found in table: id {:?}", id);
                    return Ok(false);
                }
            }
        }
        return Ok(true);
    }

    fn content_length(res: &Response) -> Option<u64> {
        res.headers()
            .get(header::CONTENT_LENGTH)?
            .to_str().ok()?.parse().ok()
    }
}
