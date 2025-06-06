use actix_files::NamedFile;
use actix_web::{web, App, HttpServer};

mod ws;

const BASE_HTML: &str = "public";

#[tokio::main]
async fn main() {
    HttpServer::new(|| {
        App::new()
        .route("/ws", web::get().to(ws::connection))
        .default_service(web::route().to( || async {NamedFile::open_async("base.html").await.unwrap() }))
    }).bind("0.0.0.0:80").unwrap().run().await.unwrap();
}