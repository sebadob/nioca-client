use leptos::*;

#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <div class="lib-loading">
            <div class="lib-loading-1"></div>
            <div class="lib-loading-2"></div>
            <div class="lib-loading-3"></div>
        </div>
    }
}
