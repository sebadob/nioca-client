use crate::components::icons::icon_magnify::IconMagnify;
use leptos::*;
use leptos_router::ActionForm;
// use leptos_use::use_throttle_fn;
use web_sys::Event;

#[server]
pub async fn get_search(search: String) -> Result<String, ServerFnError> {
    use tracing::debug;
    debug!("search: {:?}", search);
    Ok(search.to_uppercase())
}

#[component]
pub fn SearchBar() -> impl IntoView {
    let search_trigger = create_action(|q: &String| get_search(q.clone()));
    let search_action = create_server_action::<GetSearch>();
    // check if the server has returned an error
    // let has_error = move || {
    //     search_trigger
    //         .value()
    //         .with(|val| matches!(val, Some(Err(_))))
    // };
    let search_value = create_rw_signal(String::default());

    let result = create_rw_signal(None);

    create_effect(move |_| {
        let v = search_trigger.value().get();
        result.set(v);
    });
    create_effect(move |_| {
        let v = search_action.value().get();
        result.set(v);
    });

    // let throttled_search = use_throttle_fn(
    //     move || {
    //         search_trigger.dispatch(search_value.get());
    //     },
    //     250.0,
    // );

    let on_input = move |ev: Event| {
        let val = event_target_value(&ev);
        if val.len() > 2 {
            search_value.set(val);
            // throttled_search();
            search_trigger.dispatch(search_value.get());
        }
    };

    view! {
        <ActionForm action=search_action>
            <label>
                <input
                    type="search"
                    name="search"
                    class="lib-search-input"
                    on:input=on_input
                    autocomplete="off"
                />
            </label>
            <button type="submit" class="lib-search-input-btn"><IconMagnify/></button>
        </ActionForm>
        <Transition fallback=move || ()>
            {move || result.get()}
        </Transition>
    }
}
