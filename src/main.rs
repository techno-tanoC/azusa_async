#[macro_use] extern crate log;

mod app;
mod download;
mod lock_copy;
mod progress;
mod item;
mod table;

use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use warp::Filter;
use warp::http::StatusCode;

use app::App;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    warp::serve(routes()).run(([0, 0, 0, 0], 3000)).await;
}

async fn fetch(app: Arc<App>) -> Result<impl warp::Reply, Infallible> {
    debug!("[GET] /download");

    let mut vec: Vec<_> = app.table.to_vec().await.into_iter().map(|(i, pg)| {
        item::Item::from_progress(i, pg)
    }).collect();
    vec.sort();

    Ok(warp::reply::json(&vec))
}

#[derive(Deserialize, Debug)]
struct Start {
    url: String,
    name: String,
    ext: String,
}

async fn start(start: Start, app: Arc<App>) -> Result<impl warp::Reply, Infallible> {
    info!("[POST] /download {:?}", &start);

    let path = ".";

    tokio::spawn(async move {
        app.client.start(&app, &start.url, &path, &start.name, &start.ext).await;
    });

    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, Debug)]
struct Cancel {
    id: String,
}

async fn cancel(cancel: Cancel, app: Arc<App>) -> Result<impl warp::Reply, Infallible> {
    info!("[DELETE] /download {:?}", &cancel);

    tokio::spawn(async move {
        app.table.cancel(&cancel.id).await;
    });

    Ok(StatusCode::NO_CONTENT)
}

fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let app = Arc::new(App::new());

    let get = warp::path!("download")
        .and(warp::get())
        .and(with_app(app.clone()))
        .and_then(fetch);

    let post = warp::path!("download")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 1024))
        .and(warp::body::json())
        .and(with_app(app.clone()))
        .and_then(start);

    let delete = warp::path!("download")
        .and(warp::delete())
        .and(warp::query::<Cancel>())
        .and(with_app(app.clone()))
        .and_then(cancel);

    get.or(post).or(delete)
}

fn with_app(app: Arc<App>) ->  impl Filter<Extract = (Arc<App>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app.clone())
}
