use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, FocusEvent, HtmlInputElement};

#[component]
pub fn DateInputRaw(
    name: &'static str,
    /// show a custom error message with the browsers native capabilities
    #[prop(optional)]
    #[allow(unused_variables)]
    err_msg: Option<String>,
    #[prop(optional, default = "date")] typ: &'static str,
    #[prop(optional, default = false)] disabled: bool,
    #[prop(optional, default = false)] required: bool,
    /// validate a possible min date for datetime-local types only
    #[prop(optional)]
    min: Option<RwSignal<String>>,
    #[prop(optional)] value: Option<RwSignal<String>>,
) -> impl IntoView {
    let id = store_value(format!("input-{}", name));
    let is_err = create_rw_signal(false);

    let on_blur = move |ev: FocusEvent| {
        let target = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
        target.report_validity();
    };

    let on_input = move |ev: Event| {
        is_err.update(|err| {
            if *err {
                *err = false;
            }
        });
        if let Some(val) = value {
            val.set(event_target_value(&ev));
        }
    };

    let on_invalid = move |ev: Event| {
        ev.prevent_default();
        is_err.set(true);
    };

    view! {
        <div class="lib-input-container">
            <input
                type=typ
                id=id.get_value()
                name=name
                class="input-date-raw"
                disabled=disabled
                on:input=on_input
                on:invalid=on_invalid
                on:blur=on_blur
                min=move || min.map(|m| m.get())
                required=required
                prop:value=move || value.map(|v| v.get())
            />
            <label for=id.get_value() class="lib-input-label noselect">
                <AnimatedShow
                    when=is_err
                    show_class="fade-in-250"
                    hide_class="fade-ou-250"
                    hide_delay=core::time::Duration::from_millis(250)
                >
                    <div class="err lib-input-err">
                        {required.then(|| view! { <span aria-label="required">" *"</span> })}
                        {view! { <span>err_msg</span> }}
                    </div>
                </AnimatedShow>
            </label>
        </div>
    }
}
