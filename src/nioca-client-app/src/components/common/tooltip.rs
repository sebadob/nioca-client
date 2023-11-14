use leptos::*;

#[component]
pub fn Tooltip(
    children: ChildrenFn,
    text: String,
    #[prop(default = false)] dotted: bool,
) -> impl IntoView {
    let class = if dotted {
        "lib-tooltip lib-tooltip-dotted"
    } else {
        "lib-tooltip"
    };

    view! {
        <div class=class>
            <span class="lib-tooltip-text">
                {text}
            </span>
            {children()}
        </div>
    }
}
