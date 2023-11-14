use crate::components::icons::icon_check::IconCheck;
use crate::components::icons::icon_stop::IconStop;
use leptos::*;

#[component]
pub fn CheckIcon(#[prop(into)] is_checked: MaybeSignal<bool>) -> impl IntoView {
    move || {
        if is_checked.get() {
            view! { <IconCheck/> }
        } else {
            view! { <IconStop/> }
        }
    }
}
