use leptos::*;

#[component]
pub fn IconLogoutSolid(
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
                d="M12 2.25c-5.385 0-9.75 4.365-9.75 9.75s4.365 9.75 9.75 9.75 9.75-4.365 \
                9.75-9.75S17.385 2.25 12 2.25zm-1.72 6.97a.75.75 0 10-1.06 1.06L10.94 12l-1.72 \
                1.72a.75.75 0 101.06 1.06L12 13.06l1.72 1.72a.75.75 0 101.06-1.06L13.06 \
                12l1.72-1.72a.75.75 0 10-1.06-1.06L12 10.94l-1.72-1.72z"
                clip-rule="evenodd"
            />
        </svg>
    }
}