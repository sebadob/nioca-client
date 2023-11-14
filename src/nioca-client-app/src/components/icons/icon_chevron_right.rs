use leptos::*;

#[component]
pub fn IconChevronRight(
    #[prop(default = "1.5rem")] width: &'static str,
    #[prop(default = 0.9)] opacity: f32,
) -> impl IntoView {
    view! {
        <svg
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width=2
            width=width
            opacity=opacity
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M8.25 4.5l7.5 7.5-7.5 7.5"
            ></path>
        </svg>
    }
}
