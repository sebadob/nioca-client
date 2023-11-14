use crate::components::animation::theme_switch::color_scheme_pref_init;
pub use crate::components::animation::theme_switch::ColorSchemePref;
pub use crate::constants::DARK_MODE_COOKIE;
use crate::routes::index::Index;
use error_template::{AppError, ErrorTemplate};
use leptos::nonce::use_nonce;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
// use serde::{Deserialize, Serialize};

mod components;
mod constants;
mod error_template;
mod routes;
mod utils;

// #[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct SsrInitialContext {
    pub color_scheme_pref: ColorSchemePref,
    pub request_path: String,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // TODO currently, the user sees a short flash with the wrong color scheme if it was manually
    // overwritten with the theme switch -> find a way to get rid of this flash without actually
    // cutting our the other scheme completely
    //
    // to make a possible dark mode overwrite work, we need to manage the state in the context
    // from the highest level
    // we don't care about the signal here, the init fn will provide one, if none exists
    color_scheme_pref_init();

    view! {
        <Meta http_equiv="Content-Security-Policy" content=move || {
            use_nonce().map(|nonce| {
                format!(
                    "default-src 'self'; script-src 'nonce-{nonce}' \
                    'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline';"
                )
            })
            .unwrap_or_default()
            }
        />

        <Title text="Nioca-Client"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <Routes />
        </Router>
    }
}

#[component]
pub fn RoutesGenerator() -> impl IntoView {
    view! {
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <Routes />
        </Router>
    }
}

#[component]
fn Routes() -> impl IntoView {
    view! {
        <main>
            <AnimatedRoutes
                start="fade-in-150"
                outro="fade-out-150"
            >
                <Route path="" view=|| view! { <Index /> }/>
                // <Route
                //     path="/settings"
                //     view=|| view! { <WithI18n route="/settings".to_string()/> }
                // />
                // <Route
                //     path="/logout"
                //     view=|| view! { <WithI18n route="/logout".to_string()/> }
                // />
            </AnimatedRoutes>
        </main>
    }
}
