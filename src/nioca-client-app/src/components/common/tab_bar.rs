use leptos::*;

#[component]
pub fn TabBar(
    tabs: Vec<String>,
    selected: RwSignal<String>,
    #[prop(default = "lib-tab-bar")] class_outer: &'static str,
    #[prop(default = "lib-tab-bar-inner")] class_inner: &'static str,
    #[prop(default = "lib-tab-bar-btn")] class_button: &'static str,
) -> impl IntoView {
    let tabs = store_value(tabs);

    let on_click = move |index: usize| {
        let name = tabs.get_value().get(index).unwrap().clone();
        selected.set(name);
    };

    view! {
        <div class=class_outer>
            <For
                each=move || tabs.get_value().into_iter().enumerate()
                key=|(_count, tab)| tab.to_string()
                children=move |(count, tab)| view! {
                    <div class=class_inner>
                        <button class=class_button on:click=move |_| on_click(count)>
                            {tab}
                        </button>
                    </div>
                }
            />
        </div>
    }
}
