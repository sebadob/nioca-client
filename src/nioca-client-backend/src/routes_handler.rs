use crate::app_state::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use http::{HeaderValue, Request};
use leptos::{provide_context, view, LeptosOptions};
use leptos_axum::render_app_to_stream_with_context_and_replace_blocks;
use nioca_client_app::{App, ColorSchemePref, SsrInitialContext, DARK_MODE_COOKIE};

pub(crate) async fn leptos_routes_handler(
    _app_state: AppState,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> Response {
    let jar = CookieJar::from_headers(req.headers());

    // check for a possible dark mode preference overwrite
    let cookie = jar.get(DARK_MODE_COOKIE).map(|c| c.value());
    let color_scheme_pref = ColorSchemePref::from(cookie);

    let init_context = SsrInitialContext {
        color_scheme_pref,
        request_path: req.uri().path().to_string(),
    };

    let handler = render_app_to_stream_with_context_and_replace_blocks(
        options.clone(),
        move || {
            provide_context(init_context.clone());
        },
        || view! { <App/> },
        true,
    );
    let (mut parts, body) = handler(req).await.into_response().into_parts();

    parts
        .headers
        .insert("content-type", HeaderValue::from_static("text/html"));

    Response::from_parts(parts, body)
}
