#![allow(unused_imports)]

use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use async_std::prelude::*;
use async_std::sync::Arc;
use data_encoding::HEXUPPER;
use futures::{StreamExt, TryStreamExt};
use once_cell::sync::Lazy;
use ring::digest::{Context, SHA256};
use uuid::Uuid;

mod storage;
use storage::Storage;

fn index() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

static STORAGE: Lazy<Storage> = Lazy::new(|| Storage::from_env());

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    async_std::fs::create_dir_all("./tmp").await?;

    let ip = "0.0.0.0:3000";

    HttpServer::new(|| {
        App::new().wrap(middleware::Logger::default()).service(
            web::resource("/")
                .route(web::get().to(index))
                .route(web::post().to(|field| STORAGE.upload_file(field))),
        )
    })
    .bind(ip)?
    .run()
    .await
}
