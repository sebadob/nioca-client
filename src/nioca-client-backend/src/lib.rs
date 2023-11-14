use crate::app_state::Config;
use crate::routes_handler::leptos_routes_handler;
use crate::static_handler::file_and_error_handler;
use axum::routing::get;
use axum::Router;
use http::{header, HeaderValue};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use nioca_client_app::RoutesGenerator;
use std::sync::{Arc, OnceLock};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tower_http::ServiceBuilderExt;
use tracing::{debug, info};

mod app_state;
mod logging;
mod routes_handler;
mod static_handler;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) static DEV_MODE: OnceLock<bool> = OnceLock::new();

pub async fn run(port: u16) -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let level = logging::setup_logging();
    info!("nioca-client v{}", VERSION);
    info!("Log Level set to {}", level);

    let app_state = Config::new(port).await?;
    let leptos_options = app_state.leptos_options.clone();
    // let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000);
    let addr = app_state.leptos_options.site_addr;
    // TODO check with gbj if there is s possibility to inject additional context here
    let routes = generate_route_list(|| view! { <RoutesGenerator/> });
    debug!("leptos generated routes: {:?}", routes);

    // build middleware
    let sensitive_headers: Arc<[_]> = vec![header::AUTHORIZATION, header::COOKIE].into();
    let middleware = ServiceBuilder::new()
        // Mark the `Authorization` and `Cookie` headers as sensitive so it doesn't show in logs
        .sensitive_request_headers(sensitive_headers.clone())
        // Add high level tracing / logging to all requests
        .layer(
            TraceLayer::new_for_http()
                // .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                //     tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
                // })
                .make_span_with(DefaultMakeSpan::new().include_headers(false))
                .on_response(
                    DefaultOnResponse::new()
                        .include_headers(false)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .add_extension(app_state)
        .sensitive_response_headers(sensitive_headers)
        .append_response_header(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("SAMEORIGIN"),
        )
        .append_response_header(
            header::X_XSS_PROTECTION,
            HeaderValue::from_static("1; mode=block"),
        )
        .append_response_header(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        )
        .append_response_header(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        )
        .append_response_header(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        )
        .append_response_header(
            header::CONTENT_SECURITY_POLICY,
            // the other CSP values are set inside the meta head during SSR
            HeaderValue::from_static("frame-ancestors 'none'; object-src 'none'"),
        );
    let compression_middleware = ServiceBuilder::new().layer(CompressionLayer::new());

    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(leptos_axum::handle_server_fns).post(leptos_axum::handle_server_fns),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        // This ordering is really important:
        // the compression middleware is on its own and BEFORE the static file serving.
        // This way, the middleware does affect all routes above, which are dynamically generated.
        // These will be compressed automatically.
        // For the static files, we do not want compression on purpose, because there we just want
        // to serve pre-compressed files to not do the computation for the compression with each
        // request.
        .layer(compression_middleware.clone().into_inner())
        .fallback(file_and_error_handler)
        .layer(middleware.clone().into_inner())
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    info!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
