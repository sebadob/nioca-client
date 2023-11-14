use leptos::*;
use web_sys::MouseEvent;

#[component]
pub fn Switch(
    #[prop(optional)] name: String,
    #[prop(optional)] label: String,
    #[prop(into)] is_checked: RwSignal<bool>,
) -> impl IntoView {
    let on_click = move |ev: MouseEvent| {
        let val = event_target_checked(&ev);
        is_checked.set(val);
    };

    view! {
        <div class="flex">
            <div class="lib-switch-label font-label">
                {&label}
            </div>
            <label class="lib-switch">
                <input
                    type="checkbox"
                    name=name
                    on:click=on_click
                    prop:checked=move || is_checked.get()
                />
                <span class="lib-slider lib-slider-round"></span>
            </label>
        </div>
    }
}
