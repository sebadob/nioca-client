use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, FocusEvent, HtmlInputElement};

#[component]
pub fn Textarea(
    name: &'static str,
    label: String,
    #[prop(optional)] input_class: &'static str,
    /// show a custom error message with the browsers native capabilities
    err_msg: String,
    #[prop(optional, default = false)] disabled: bool,
    #[prop(optional, default = false)] required: bool,
    /// validate text input length
    #[prop(optional)]
    max_length: Option<u16>,
    /// browsers native pattern validation
    #[prop(optional)]
    pattern: Option<&'static str>,
    #[prop(optional, default = 5)] rows: u16,
    #[prop(optional)] value: Option<RwSignal<String>>,
) -> impl IntoView {
    let id = store_value(format!("input-{}", name));
    let label = store_value(label);
    let is_err = create_rw_signal(false);
    let err_msg = store_value(err_msg);
    let pattern = store_value(pattern.map(|p| js_sys::RegExp::new(p, "gm")));

    let on_blur = move |ev: FocusEvent| {
        let target = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
        if let Some(p) = pattern.get_value() {
            target.set_custom_validity("");
            if !p.test(&event_target_value(&ev)) {
                is_err.set(true);
                target.set_custom_validity(&err_msg.get_value());
            }
        }
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
            <textarea
                id=id.get_value()
                name=name
                class=input_class
                disabled=disabled
                placeholder=label.get_value()
                on:input=on_input
                on:invalid=on_invalid
                on:blur=on_blur
                maxlength=max_length
                required=required
                rows=rows
                prop:value=move || value.map(|v| v.get())
            />
            <label for=id.get_value() class="lib-input-label noselect">
                <div class="lib-input-label-inner font-label">
                    {label.get_value().to_uppercase()}
                    {required.then(|| view! {  <span aria-label="required">" *"</span> })}
                </div>
                <AnimatedShow
                    when=is_err
                    show_class="fade-in-250"
                    hide_class="fade-ou-250"
                    hide_delay=core::time::Duration::from_millis(250)
                >
                    <div class="err lib-input-err">
                        {move || err_msg.get_value()}
                    </div>
                </AnimatedShow>
            </label>
        </div>
    }
}
