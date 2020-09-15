use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use async_std::prelude::*;
use data_encoding::HEXUPPER;
use futures::{StreamExt, TryStreamExt};
use ring::digest::{Context, SHA256};
use uuid::Uuid;

pub struct Storage {
    path: String,
}

impl Storage {
    pub fn new(path: String) -> Storage {
        Storage { path: path }
    }

    pub fn from_env() -> Storage {
        let path;
        match std::env::var("STORAGE_DIR") {
            Ok(value) => path = value,
            Err(_) => path = "./tmp/".to_string()
        }
        Storage::new(path)
    }

    pub fn setup(&self) {
        std::fs::create_dir_all(self.path.clone()).unwrap();
    }

    pub async fn upload_file(&self, mut payload: Multipart) -> Result<HttpResponse, Error> {
        while let Ok(Some(mut field)) = payload.try_next().await {
            let filename: String = Uuid::new_v4().to_string();
            let filepath = format!("{}{}", self.path, sanitize_filename::sanitize(filename));
            let mut f = async_std::fs::File::create(&filepath).await?;

            let mut context = Context::new(&SHA256);
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                context.update(&data);
                f.write_all(&data).await?;
            }
            let hash = HEXUPPER.encode(context.finish().as_ref());
            let final_path = format!("{}{}", self.path, hash);
            if async_std::path::Path::new(&final_path).exists().await {
                println!("exists!! {}", final_path);
                async_std::fs::remove_file(filepath).await?;
            } else {
                async_std::fs::rename(filepath, final_path).await?;
            }
        }
        Ok(HttpResponse::Ok().into())
    }
}
