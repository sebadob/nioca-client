use leptos::*;
use web_sys::Event;

#[derive(Debug, Clone, PartialEq)]
pub struct OptionsEntry {
    pub name: String,
    pub value: String,
    pub color: Option<String>,
}

#[component]
pub fn Options(
    name: &'static str,
    label: String,
    options: Vec<OptionsEntry>,
    value: RwSignal<String>,
) -> impl IntoView {
    let options = store_value(options);

    let on_change = move |ev: Event| {
        value.set(event_target_value(&ev));
    };

    view! {
        <div class="lib-opt-select">
            <label for=name class="font-label lib-opt-select-label noselect">{label.to_uppercase()}</label>
            <select id=name name=name on:change=on_change>
                <For
                    each=move || options.get_value()
                    key=|opt| opt.value.clone()
                    children=move |opt| view! {
                        <option
                            value=opt.value.clone()
                            selected=move || value.get() == opt.value
                            // style=format!("color:{};border-bottom:1px solid {}", opt.color.as_ref().unwrap(), opt.color.as_ref().unwrap())
                        >
                            {opt.name}
                        </option>
                    }
                />
            </select>
        </div>
    }
}
