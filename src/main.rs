use actix_multipart::Multipart;
use ring::digest::{Context, SHA256};
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::{StreamExt, TryStreamExt};
use async_std::prelude::*;
use uuid::Uuid;
use data_encoding::HEXUPPER;


async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let filename: String = Uuid::new_v4().to_string();
        let filepath = format!("./tmp/{}", sanitize_filename::sanitize(filename));
        let mut f = async_std::fs::File::create(&filepath).await?;

        let mut context = Context::new(&SHA256);
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            context.update(&data);
            f.write_all(&data).await?;
        }
        let hash = HEXUPPER.encode(context.finish().as_ref());
        let final_path = format!("./tmp/{}", hash);
        if async_std::path::Path::new(&final_path).exists().await {
            println!("exists!! {}", final_path);
            async_std::fs::remove_file(filepath).await?;
        } else {
            async_std::fs::rename(filepath, final_path).await?;
        }
    }
    Ok(HttpResponse::Ok().into())
}

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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    async_std::fs::create_dir_all("./tmp").await?;

    let ip = "0.0.0.0:3000";

    HttpServer::new(|| {
        App::new().wrap(middleware::Logger::default()).service(
            web::resource("/")
                .route(web::get().to(index))
                .route(web::post().to(save_file)),
        )
    })
    .bind(ip)?
    .run()
    .await
}
