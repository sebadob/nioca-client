use crate::components::icons::icon_clipboard::IconClipboard;
use crate::components::icons::icon_eye::IconEye;
use crate::components::icons::icon_eye_slash::IconEyeSlash;
use crate::components::inputs::input::Input;
use crate::utils::helpers::copy_to_clip;
use leptos::*;

#[component]
pub fn PasswordInput(
    name: &'static str,
    label: String,
    /// show a custom error message with the browsers native capabilities
    err_msg: String,
    #[prop(optional, default = false)] is_new_password: bool,
    /// adjust autocomplete
    #[prop(optional, default = "on")]
    autocomplete: &'static str,
    #[prop(optional, default = false)] disabled: bool,
    #[prop(optional, default = false)] show_copy: bool,
    /// browsers native pattern validation
    pattern: &'static str,
    #[prop(optional, into)] value: Option<RwSignal<String>>,
) -> impl IntoView {
    let autocomplete = store_value(if is_new_password {
        "new-password"
    } else {
        autocomplete
    });
    let input_class = if show_copy {
        "lib-password-input-copy"
    } else {
        "lib-password-input"
    };
    let show = create_rw_signal(false);
    let rw_value = create_rw_signal(if let Some(v) = value {
        v.get_untracked()
    } else {
        String::default()
    });

    create_effect(move |_| {
        if let Some(val) = value {
            val.set(rw_value.get());
        }
    });

    let copy_to_clip = move |_| {
        copy_to_clip(&rw_value.get());
    };

    view! {
        <div>
            <div class="relative">
                <div
                    class="absolute pointer lib-password-icon-eye"
                    on:click=move |_| show.update(|s| *s = !*s)
                >
                    {move || if show.get() {
                        view! {  <IconEye opacity=0.8 /> }
                    } else {
                        view! {  <IconEyeSlash opacity=0.8 /> }
                    }}
                </div>
                {show_copy.then(|| view! {
                    <div
                        class="absolute pointer lib-password-icon-copy"
                        on:click=copy_to_clip
                    >
                        <IconClipboard opacity=0.7 />
                    </div>
                })}
            </div>
            <Input
                typ="password"
                name=name
                label=label
                input_class=input_class
                err_msg=err_msg
                autocomplete=autocomplete.get_value()
                disabled=disabled
                pattern=pattern
                required=true
                switch_to_text=show
                value=rw_value
            />
        </div>
    }
}
