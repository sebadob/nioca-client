use crate::components::common::loading::Loading;
use leptos::ev::MouseEvent;
use leptos::*;
use std::ops::Not;

#[component]
pub fn Button<F>(
    label: String,
    on_click: F,
    #[prop(default = 2)] level: i32,
    #[prop(optional, into)] is_loading: MaybeSignal<bool>,
    #[prop(optional, into)] is_selected: MaybeSignal<bool>,
    #[prop(optional, optional, into)] disabled: MaybeSignal<bool>,
) -> impl IntoView
where
    F: FnMut(MouseEvent) + 'static,
{
    let class = move || match is_selected.get() {
        true => match level {
            1 => "lib-btn-1 lib-btn-sel",
            2 => "lib-btn-2 lib-btn-sel",
            3 => "lib-btn-3 lib-btn-sel",
            _ => "lib-btn-4 lib-btn-sel",
        },
        false => match level {
            1 => "lib-btn-1",
            2 => "lib-btn-2",
            3 => "lib-btn-3",
            _ => "lib-btn-4",
        },
    };

    view! {
        <div class="form-btn-width">
            <button class=class on:click=on_click  disabled=move || disabled.get()>
                {move || is_loading.get().not().then_some(label.clone())}
                {move || { is_loading.get().then(|| {
                    view! {
                        <div class="flex-center">
                            <Loading/>
                        </div>
                    }
                })}}
            </button>
        </div>
    }
}

#[component]
pub fn ButtonInvis<F>(
    on_click: F,
    children: Children,
    #[prop(optional, optional, into)] disabled: MaybeSignal<bool>,
) -> impl IntoView
where
    F: FnMut(MouseEvent) + 'static,
{
    view! {
        <button
            class="lib-btn-invis"
            on:click=on_click
            disabled=move || disabled.get()
        >
            {children()}
        </button>
    }
}

#[component]
pub fn ButtonForm(
    label: String,
    #[prop(optional, default = "submit")] typ: &'static str,
    #[prop(optional, default = 2)] level: i32,
    #[prop(optional, optional, into)] is_loading: MaybeSignal<bool>,
    #[prop(optional, optional, into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let class = match level {
        1 => "lib-btn-1",
        2 => "lib-btn-2",
        3 => "lib-btn-3",
        _ => "lib-btn-4",
    };

    view! {
        <div class="form-btn-width">
            <button type=typ class=class disabled=move || disabled.get()>
                {move || is_loading.get().not().then_some(label.clone())}
                {move || { is_loading.get().then(|| {
                    view! {
                        <div class="flex-center">
                            <Loading/>
                        </div>
                    }
                })}}
            </button>
        </div>
    }
}

#[component]
pub fn ButtonFormInvis(
    children: Children,
    #[prop(optional, optional, into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    view! {
        <button type="submit" class="lib-btn-invis" disabled=move || disabled.get()>
            {children()}
        </button>
    }
}

#[component]
pub fn ButtonIcon<F>(
    children: Children,
    on_click: F,
    #[prop(optional, into)] is_selected: MaybeSignal<bool>,
    #[prop(optional, optional, into)] disabled: MaybeSignal<bool>,
) -> impl IntoView
where
    F: FnMut(MouseEvent) + 'static,
{
    let class = move || match is_selected.get() {
        true => "lib-btn-2 lib-btn-icon lib-btn-sel",
        false => "lib-btn-2 lib-btn-icon",
    };

    view! {
        <div>
            <button class=class on:click=on_click  disabled=move || disabled.get()>
                {children()}
            </button>
        </div>
    }
}
