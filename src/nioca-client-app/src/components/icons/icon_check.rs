use leptos::*;

#[component]
pub fn IconCheck(
    #[prop(default = "1.5rem")] width: &'static str,
    #[prop(default = 0.9)] opacity: f32,
) -> impl IntoView {
    view! {
        <svg
            fill="none"
            viewBox="0 0 24 24"
            stroke="var(--col-ok)"
            stroke-width=2
            width=width
            opacity=opacity
        >
            <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
        </svg>
    }
}
