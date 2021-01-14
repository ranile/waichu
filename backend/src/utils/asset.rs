use bytes::BufMut;
use common::errors::ApiError;
use common::Asset;
use futures::future::BoxFuture;
use futures::TryStreamExt;
use image::ImageFormat;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use warp::http::StatusCode;
use warp::multipart::{FormData, Part};
use warp::{multipart, Filter, Rejection};

lazy_static! {
    pub static ref ASSETS_PATH: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

pub trait AssetExt {
    fn path(&self) -> BoxFuture<anyhow::Result<PathBuf>>;
    fn save(&self) -> BoxFuture<anyhow::Result<()>>;
    fn delete(&self) -> BoxFuture<anyhow::Result<()>>;
}

impl AssetExt for Asset {
    fn path(&self) -> BoxFuture<anyhow::Result<PathBuf>> {
        Box::pin(async move {
            Ok(
                PathBuf::from_str(&ASSETS_PATH.lock().await.as_ref().unwrap())?
                    .join(format!("{}.jpeg", &self.uuid)),
            )
        })
    }

    fn save(&self) -> BoxFuture<anyhow::Result<()>> {
        Box::pin(async move {
            let path = self.path().await?;
            let bytes = self.bytes.clone();

            tokio::task::spawn_blocking(move || {
                let img = image::load_from_memory(&bytes).unwrap();

                img.save_with_format(path, ImageFormat::Jpeg)?;
                Ok::<_, anyhow::Error>(())
            })
            .await??;

            Ok(())
        })
    }

    fn delete(&self) -> BoxFuture<anyhow::Result<()>> {
        Box::pin(async move {
            let path = self.path().await?;

            fs::remove_file(path).await?;
            Ok(())
        })
    }
}

// only works for images but that's only what i should need
async fn upload(form: FormData) -> Result<Asset, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        ApiError::new_with_message_and_status(&e.to_string(), StatusCode::BAD_REQUEST)
            .into_rejection()
    })?;

    let parts = parts
        .into_iter()
        .filter(|part| part.name() == "file")
        .collect::<Vec<Part>>();

    if parts.len() != 1 {
        return Err(ApiError::new_with_message_and_status(
            "only one part with name of `file` is allowed",
            StatusCode::BAD_REQUEST,
        )
        .into_rejection());
    }

    let part = parts.into_iter().next().unwrap();

    let bytes = part
        .stream()
        .try_fold(Vec::new(), |mut vec, data| {
            vec.put(data);
            async move { Ok(vec) }
        })
        .await
        .map_err(|e| {
            ApiError::new_with_message_and_status(&e.to_string(), StatusCode::BAD_REQUEST)
                .into_rejection()
        })?;

    let bytes = Arc::new(bytes);
    let img = {
        let bytes = Arc::clone(&bytes);
        spawn_blocking(move || image::load_from_memory(&*bytes))
            .await
            .unwrap()
    };

    if img.is_ok() {
        Ok(Asset::new(bytes))
    } else {
        Err(
            ApiError::new_with_message_and_status("invalid image type", StatusCode::BAD_REQUEST)
                .into_rejection(),
        )
    }
}

pub fn multipart() -> impl Filter<Extract = (Asset,), Error = Rejection> + Clone {
    multipart::form().and_then(upload)
}
