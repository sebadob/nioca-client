use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, FocusEvent, HtmlInputElement};

#[component]
pub fn Input(
    name: &'static str,
    label: String,
    #[prop(optional)] input_class: &'static str,
    /// show a custom error message with the browsers native capabilities
    #[prop(optional)]
    err_msg: Option<String>,
    #[prop(optional, default = "text")] typ: &'static str,
    /// adjust autocomplete
    #[prop(optional, default = "on")]
    autocomplete: &'static str,
    #[prop(optional, default = false)] disabled: bool,
    #[prop(optional, default = false)] required: bool,
    /// validate text input length
    #[prop(optional)]
    max_length: Option<u16>,
    /// validate number input min
    #[prop(optional)]
    min: Option<u16>,
    /// validate a possible min date for datetime-local types only
    #[prop(optional)]
    min_date_str: Option<RwSignal<String>>,
    /// validate a possible min date for datetime-local types only
    #[prop(optional)]
    max_date_str: Option<RwSignal<String>>,
    /// validate number input max
    #[prop(optional)]
    max: Option<u32>,
    /// browsers native pattern validation
    #[prop(optional)]
    pattern: Option<&'static str>,
    #[prop(optional, default = "1")] step: &'static str,
    /// if `true` it will switch between 'text' and the given `typ`
    #[prop(optional, into)]
    switch_to_text: MaybeSignal<bool>,
    #[prop(optional)] value: Option<RwSignal<String>>,
) -> impl IntoView {
    let id = store_value(format!("input-{}", name));
    let label = store_value(label);
    let is_err = create_rw_signal(false);
    let typ = move || if switch_to_text.get() { "text" } else { typ };
    let min_value = move || {
        if let Some(m) = min_date_str {
            Some(m.get())
        } else {
            min.map(|m| m.to_string())
        }
    };
    let max_value = move || {
        if let Some(max) = max_date_str {
            Some(max.get())
        } else {
            max.map(|max| max.to_string())
        }
    };

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
                class=input_class
                disabled=disabled
                autocomplete=autocomplete
                placeholder=label.get_value()
                on:input=on_input
                on:invalid=on_invalid
                on:blur=on_blur
                min=min_value
                max=max_value
                maxlength=max_length
                required=required
                pattern=pattern
                step=step
                prop:value=move || value.map(|v| v.get()).unwrap_or_default()
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
                        {err_msg.as_ref()}
                    </div>
                </AnimatedShow>
            </label>
        </div>
    }
}
