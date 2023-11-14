use crate::components::icons::icon_moon::IconMoon;
use crate::components::icons::icon_sun::IconSun;
use crate::components::inputs::switch::Switch;
use crate::constants::DARK_MODE_COOKIE;
use crate::utils::helpers::{get_cookie, get_document, set_cookie};
use crate::SsrInitialContext;
use leptos::leptos_dom::is_browser;
use leptos::*;
// use serde::{Deserialize, Serialize};
use std::ops::Add;

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub enum ColorSchemePref {
    Dark,
    Light,
}

impl From<Option<&str>> for ColorSchemePref {
    fn from(value: Option<&str>) -> Self {
        if let Some(c) = value {
            match c {
                "dark" => Self::Dark,
                "light" => Self::Light,
                _ => Self::Light,
            }
        } else {
            Self::Light
        }
    }
}

impl ColorSchemePref {
    pub fn as_str(&self) -> &str {
        match self {
            ColorSchemePref::Dark => "dark",
            ColorSchemePref::Light => "light",
        }
    }
}

pub fn color_scheme_pref_init() -> RwSignal<ColorSchemePref> {
    let sig = use_context::<RwSignal<ColorSchemePref>>();
    match sig {
        None => {
            let pref = if is_browser() {
                match get_cookie(DARK_MODE_COOKIE) {
                    None => {
                        let media_query = window()
                            .match_media("(prefers-color-scheme: dark)")
                            .expect("MediaQueryListResult");
                        match media_query {
                            None => ColorSchemePref::Light,
                            Some(q) => {
                                if q.matches() {
                                    ColorSchemePref::Dark
                                } else {
                                    ColorSchemePref::Light
                                }
                            }
                        }
                    }
                    Some(value) => {
                        let pref = ColorSchemePref::from(Some(value.as_str()));
                        let body = get_document().body().unwrap();
                        match &pref {
                            ColorSchemePref::Dark => {
                                body.class_list().add_1("dark-theme").unwrap();
                                body.class_list().remove_1("light-theme").unwrap();
                            }
                            ColorSchemePref::Light => {
                                body.class_list().remove_1("dark-theme").unwrap();
                                body.class_list().add_1("light-theme").unwrap();
                            }
                        }
                        pref
                    }
                }
            } else {
                match use_context::<SsrInitialContext>() {
                    None => ColorSchemePref::Light,
                    Some(ctx) => ctx.color_scheme_pref,
                }
            };
            let sig = create_rw_signal(pref);
            provide_context(sig);
            sig
        }
        Some(sig) => sig,
    }
}

#[component]
pub fn ThemeSwitch() -> impl IntoView {
    let color_scheme_preferred = color_scheme_pref_init();
    let is_checked = create_rw_signal(match color_scheme_preferred.get_untracked() {
        ColorSchemePref::Dark => true,
        ColorSchemePref::Light => false,
    });
    let is_first_render = store_value(true);

    create_effect(move |_| {
        let body = get_document().body().unwrap();
        let new_pref = if is_checked.get() {
            body.class_list().add_1("dark-theme").unwrap();
            body.class_list().remove_1("light-theme").unwrap();
            ColorSchemePref::Dark
        } else {
            body.class_list().remove_1("dark-theme").unwrap();
            body.class_list().add_1("light-theme").unwrap();
            ColorSchemePref::Light
        };

        if is_first_render.get_value() {
            is_first_render.set_value(false);
            // do not set the manual overwrite cookie on the very first render
        } else {
            let exp = chrono::Utc::now().add(chrono::Duration::days(365));
            set_cookie(DARK_MODE_COOKIE, new_pref.as_str(), exp);
        }
        color_scheme_preferred.set(new_pref);
    });

    view! {
        <div class="flex lib-theme-switch">
            <div class="lib-theme-icon-sun">
                <IconSun/>
            </div>
            <Switch is_checked />
            <div class="lib-theme-icon-moon">
                <IconMoon/>
            </div>
        </div>
    }
}
