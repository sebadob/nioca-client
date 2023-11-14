use leptos::*;

#[component]
pub fn IconPlus(
    #[prop(default = "1.5rem")] width: &'static str,
    #[prop(default = 0.9)] opacity: f32,
) -> impl IntoView {
    view! {
        <svg
            fill="none"
            viewBox="0 0 24 24"
            stroke="var(--col-text)"
            stroke-width=2
            width=width
            opacity=opacity
        >
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
        </svg>
    }
}
