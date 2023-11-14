use leptos::*;

#[component]
pub fn IconClock(
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
                d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z"
            />
        </svg>
    }
}

#[component]
pub fn IconClockSolid(
    #[prop(default = "1.5rem")] width: &'static str,
    #[prop(default = 0.9)] opacity: f32,
) -> impl IntoView {
    view! {
        <svg
            fill="currentColor"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width=0.5
            width=width
            opacity=opacity
        >
            <path
                fill-rule="evenodd"
                d="M12 2.25c-5.385 0-9.75 4.365-9.75 9.75s4.365 9.75 9.75 9.75 9.75-4.365 9.75-9.75S17.385 2.25 12 \
                2.25zM12.75 6a.75.75 0 00-1.5 0v6c0 .414.336.75.75.75h4.5a.75.75 0 000-1.5h-3.75V6z"
                clip-rule="evenodd"
            />
        </svg>
    }
}
