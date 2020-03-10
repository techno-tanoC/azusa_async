use bytes::Bytes;
use reqwest::{header, Response};
use tokio::fs::File;
use tokio::io::AsyncSeek;
use tokio::prelude::*;
use tokio::stream::{Stream, StreamExt};

use super::app::App;
use super::progress::Progress;
use super::table::Table;
use super::error::{Result, Error};

pub struct Download(reqwest::Client);

impl Download {
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("ClientBuilder::build()");
        Download(client)
    }

    pub async fn start(&self, app: &App, url: &str, name: &str, ext: &str) -> Result<()> {
        let mut temp: File = tempfile::tempfile()?.into();
        let res = self.0.get(url).send().await?;

        debug!("{:?}", &res);

        if res.status().is_success() {
            let id = Table::generate_id();
            let pg = Progress::new(name.to_string());
            app.table.add(&id, pg).await;

            info!("name: {:?}, url: {:?}, id: {:?}", &name, url, &id);
            let ret = Self::download(app, &id, res, &mut temp, name, ext).await;

            app.table.delete(&id).await;
            ret
        } else {
            Err(Error::NonSuccessStatusError(format!("{:?}", res)))
        }
    }

    async fn download<W>(app: &App, id: &str, res: reqwest::Response, w: &mut W, name: &str, ext: &str) -> Result<()>
        where
            W: AsyncRead + AsyncWrite + AsyncSeek + Unpin
    {
        if let Some(cl) = Self::content_length(&res) {
            app.table.set_total(&id, cl).await;
        }

        let mut stream = res.bytes_stream();
        let flag = Self::read_stream(app, &id, &mut stream, w).await?;

        if flag {
            app.lock_copy.copy(w, &app.path, name, ext).await?;
        }

        Ok(())
    }

    async fn read_stream<W, S>(app: &App, id: &str, stream: &mut S, f: &mut W) -> Result<bool>
        where
            W: AsyncWrite + Unpin,
            S: Stream<Item = reqwest::Result<Bytes>> + Unpin
    {
        while let Some(bytes) = stream.next().await.transpose()? {
            match app.table.is_canceled(id).await {
                Some(true) => {
                    debug!("canceled id: {:?}", id);
                    return Ok(false);
                },
                Some(false) => {
                    app.table.progress(id, bytes.as_ref().len() as u64).await;
                    f.write_all(&bytes).await?;
                },
                None => {
                    error!("read_chunks id not found in table: id {:?}", id);
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn content_length(res: &Response) -> Option<u64> {
        res.headers()
            .get(header::CONTENT_LENGTH)?
            .to_str().ok()?.parse().ok()
    }
}
