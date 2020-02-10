mod app;
mod download;
mod lock_copy;
mod progress;
mod table;

use tokio::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new().unwrap();
    let app = app::App::new();
    let download = download::Download::new();

    println!("Hello, world!");

    rt.block_on(async {
        download.start(&app, url, &".", name, ext).await;
    });

    println!("See you, world!");
}
