use bytes::Bytes;
use reqwest::{header, Response};
use tokio::fs::File;
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
        let id = Table::generate_id();
        let pg = Progress::new(name.to_string());
        app.table.add(&id, pg).await;
        let mut temp: File = tempfile::tempfile()?.into();
        let ret = self.download(app, &id, url, &mut temp, name, ext).await;
        app.table.delete(&id).await;
        ret
    }

    async fn download(&self, app: &App, id: &str, url: &str, temp: &mut File, name: &str, ext: &str) -> Result<()> {
        let res = self.0.get(url).send().await?;

        debug!("{:?}", &res);

        if res.status().is_success() {
            info!("name: {:?}, url: {:?}, id: {:?}", &name, url, &id);
            if let Some(cl) = Self::content_length(&res) {
                app.table.set_total(&id, cl).await;
            }

            let mut stream = res.bytes_stream();
            let flag = Self::read_stream(&app.table, &id, &mut stream, temp).await?;

            if flag {
                app.lock_copy.copy(temp, name, ext).await?;
            }
            Ok(())
        } else {
            Err(Error::NonSuccessStatusError(format!("{:?}", res)))
        }
    }

    async fn read_stream<W, S>(table: &Table, id: &str, stream: &mut S, f: &mut W) -> Result<bool>
        where
            W: AsyncWrite + Unpin,
            S: Stream<Item = reqwest::Result<Bytes>> + Unpin
    {
        while let Some(bytes) = stream.next().await.transpose()? {
            match table.is_canceled(id).await {
                Some(true) => {
                    debug!("canceled id: {:?}", id);
                    return Ok(false);
                },
                Some(false) => {
                    table.progress(id, bytes.as_ref().len() as u64).await;
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

#[cfg(test)]
mod tests {
    use super::*;

    use http::response;
    use std::io::Cursor;

    #[tokio::test]
    async fn read_stream_test1() {
        let table = Table::new();
        let (id, name) = ("1".to_string(), "progress".to_string());
        let pg = Progress::new(name.clone());
        table.add(&id, pg).await;

        let ret = {
            let mut to: Cursor<Vec<u8>> = Cursor::new(vec![]);
            let chunks: Vec<reqwest::Result<bytes::Bytes>> = vec![
                Ok("hello".to_string().into()),
                Ok("hello".to_string().into())
            ];
            let mut stream = tokio::stream::iter(chunks);
            Download::read_stream(&table, &id, &mut stream, &mut to).await
        };

        assert!(ret.unwrap());
        assert_eq!(table.to_vec().await, [(id.clone(), Progress { name, total: 0, size: 10, canceled: false })]);

        table.delete(&id).await;
    }

    #[tokio::test]
    async fn read_stream_test2() {
        let table = Table::new();
        let (id, name) = ("1".to_string(), "progress".to_string());
        let pg = Progress::new(name.clone());
        table.add(&id, pg).await;

        {
            let mut to: Cursor<Vec<u8>> = Cursor::new(vec![]);
            let chunks: Vec<reqwest::Result<bytes::Bytes>> = vec![
                Ok("hello".to_string().into()),
                Ok("hello".to_string().into())
            ];
            let mut stream = tokio::stream::iter(chunks);
            Download::read_stream(&table, &id, &mut stream, &mut to).await.unwrap();
        }

        {
            let mut to: Cursor<Vec<u8>> = Cursor::new(vec![]);
            let chunks: Vec<reqwest::Result<bytes::Bytes>> = vec![
                Ok("hello".to_string().into()),
                Ok("hello".to_string().into())
            ];
            let mut stream = tokio::stream::iter(chunks);
            Download::read_stream(&table, &id, &mut stream, &mut to).await.unwrap();
        }

        assert_eq!(table.to_vec().await, [(id.clone(), Progress { name, total: 0, size: 20, canceled: false })]);

        table.delete(&id).await;
    }

    #[tokio::test]
    async fn read_stream_test3() {
        let table = Table::new();
        let (id, name) = ("1".to_string(), "progress".to_string());
        let pg = Progress::new(name.clone());
        table.add(&id, pg).await;
        table.cancel(&id).await;

        let ret = {
            let mut to: Cursor<Vec<u8>> = Cursor::new(vec![]);
            let chunks: Vec<reqwest::Result<bytes::Bytes>> = vec![
                Ok("hello".to_string().into()),
                Ok("hello".to_string().into())
            ];
            let mut stream = tokio::stream::iter(chunks);
            Download::read_stream(&table, &id, &mut stream, &mut to).await
        };

        assert!(!ret.unwrap());
        assert_eq!(table.to_vec().await, [(id.clone(), Progress { name, total: 0, size: 0, canceled: true })]);

        table.delete(&id).await;
    }

    #[test]
    fn content_length_test() {
        let mut res = response::Response::new("".to_string());
        res.headers_mut().insert(header::CONTENT_LENGTH, "1000".parse().unwrap());
        assert_eq!(Download::content_length(&res.into()), Some(1000));

        let mut res = response::Response::new("".to_string());
        res.headers_mut().insert(header::CONTENT_LENGTH, "invalid".parse().unwrap());
        assert_eq!(Download::content_length(&res.into()), None);

        let res = response::Response::new("".to_string()).into();
        assert_eq!(Download::content_length(&res), None);
    }
}
