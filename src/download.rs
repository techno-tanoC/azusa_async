use reqwest::{header, Response};
use std::path::*;

use super::app::App;
use super::progress::Progress;
use super::table::Table;
use super::error::{Error, Result};

pub struct Download(reqwest::Client);

impl Download {
    pub fn new() -> Self {
        Download(reqwest::Client::new())
    }

    pub async fn start<P: AsRef<Path>>(&self, app: &App, url: &str, dest: &P, name: &str, ext: &str) -> Result<()> {
        let mut res = self.0.get(url).send().await?;

        if res.status().is_success() {
            let id = Table::generate_id();
            let pg = Progress::new(name.to_string())?;
            app.table.add(&id, pg).await;

            if let Some(cl) = Self::content_length(&res) {
                app.table.set_total(&id, cl).await;
            }

            let result = Self::read_chunks(app, &id, &mut res).await;
            let ret = match result {
                Ok(true) => {
                    let mut pg = app.table.delete(&id).await.unwrap();
                    pg.flush().await?;
                    app.lock_copy.copy(&pg.path(), dest, name, ext).await
                },
                Ok(false) => {
                    Ok(())
                },
                Err(err) => {
                    Err(err)
                }
            };
            ret
        } else {
            Err(Error::HttpStatusError)
        }
    }

    async fn read_chunks(app: &App, id: &str, res: &mut reqwest::Response) -> Result<bool> {
        while let Some(byte) = res.chunk().await? {
            match app.table.is_canceled(id).await {
                Some(true) => {
                    return Ok(false);
                },
                Some(false) => {
                    app.table.progress(id, byte).await?;
                },
                None => {
                    return Ok(false);
                }
            }
        }
        return Ok(true);
    }

    fn content_length(res: &Response) -> Option<u64> {
        res.headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok()?.parse().ok())
    }
}
