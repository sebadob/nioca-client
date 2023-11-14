use crate::DEV_MODE;
use axum::body::Body;
use axum::extract::State;
use axum::{
    body::{boxed, Full},
    http::{header, Response, StatusCode},
    response,
};
use http::{Request, Uri};
use leptos::*;
use std::borrow::Cow;
use tracing::error;

#[derive(rust_embed::RustEmbed)]
#[folder = "../../target/site/"]
struct Assets;

pub(crate) async fn file_and_error_handler(
    uri: Uri,
    State(_options): State<LeptosOptions>,
    req: Request<Body>,
) -> response::Response {
    let (_, path) = uri.path().split_at(1); // split off the first `/`
    let mime = mime_guess::from_path(path);

    let is_dev_mode = *DEV_MODE.get().unwrap();

    let accept_encoding = req
        .headers()
        .get("accept-encoding")
        .map(|h| h.to_str().unwrap_or("none"))
        .unwrap_or("none");
    let (path, encoding) = if is_dev_mode {
        // during DEV, don't care about the precompression -> faster workflow
        (Cow::from(path), "none")
    } else if accept_encoding.contains("br") {
        (Cow::from(format!("{}.br", path)), "br")
    } else if accept_encoding.contains("gzip") {
        (Cow::from(format!("{}.gz", path)), "gzip")
    } else {
        (Cow::from(path), "none")
    };

    match Assets::get(path.as_ref()) {
        Some(content) => {
            let body = boxed(Full::from(content.data));

            match is_dev_mode {
                true => Response::builder()
                    .header(header::CONTENT_TYPE, mime.first_or_octet_stream().as_ref())
                    .header(header::CONTENT_ENCODING, encoding)
                    .body(body)
                    .unwrap(),
                false => Response::builder()
                    .header(header::CACHE_CONTROL, "max-age=86400")
                    .header(header::CONTENT_TYPE, mime.first_or_octet_stream().as_ref())
                    .header(header::CONTENT_ENCODING, encoding)
                    .body(body)
                    .unwrap(),
            }
        }

        None => {
            error!("Asset {} not found", path);
            // for a in Assets::iter() {
            //     tracing::debug!("Available asset: {}", a);
            // }
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("not found")))
                .unwrap()
        }
    }
}
