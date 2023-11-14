// use crate::components::icons::icon_clock::IconClockSolid;
// use crate::components::icons::icon_database::IconDatabaseSolid;
// use crate::components::icons::icon_folder::IconFolderSolid;
// use crate::components::icons::icon_home::IconHomeSolid;
// use crate::components::icons::icon_logout::IconLogoutSolid;
// use crate::components::icons::icon_scissors::IconScissorsSolid;
// use crate::components::icons::icon_users::IconUsersSolid;
// use crate::components::icons::icon_wrench_screw::IconWrenchScrewSolid;
// use crate::components::nav::nav_index::NavIndex;
// use crate::components::nav::nav_inventory::NavInventory;
// use crate::components::nav::nav_time::NavTime;
// use crate::components::nav::nav_work::NavWork;
// use leptos::*;
// use leptos_router::A;
// use web_sys::MouseEvent;
//
// #[derive(Debug, Clone)]
// pub struct NavContext {
//     pub view_mode: RwSignal<ViewMode>,
//     pub expanded_navs: NavContextExpanded,
// }
//
// impl NavContext {
//     pub fn new() -> Self {
//         Self {
//             view_mode: create_rw_signal(ViewMode::Desktop), // nothing else implemented yet
//             expanded_navs: NavContextExpanded {
//                 main_nav: create_rw_signal(true),
//                 inventory_sub_1: create_rw_signal(true),
//             },
//         }
//     }
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub enum ViewMode {
//     Desktop,
//     // Tablet,
//     // TabletWide,
//     // Phone,
//     // PhoneWide,
// }
//
// #[derive(Debug, Clone)]
// pub struct NavContextExpanded {
//     pub main_nav: RwSignal<bool>,
//     pub inventory_sub_1: RwSignal<bool>,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// enum SubRoute {
//     Index,
//     Inventory,
//     Time,
//     Settings,
//     Work,
// }
//
// impl SubRoute {
//     pub fn as_class(&self) -> &str {
//         match self {
//             Self::Index => "nav-sub-menu-index",
//             Self::Inventory => "nav-sub-menu-inventory",
//             Self::Time => "nav-sub-menu-timetracking",
//             Self::Settings => "nav-sub-menu-settings",
//             Self::Work => "nav-sub-menu-work",
//         }
//     }
// }
//
// #[cfg(not(target_arch = "wasm32"))]
// fn get_initial_path() -> String {
//     use crate::SsrInitialContext;
//
//     match use_context::<SsrInitialContext>() {
//         None => {
//             // this happens during the initial route generation from leptos options
//             "/".to_string()
//         }
//         Some(ctx) => ctx.request_path,
//     }
// }
//
// #[cfg(target_arch = "wasm32")]
// fn get_initial_path() -> String {
//     crate::utils::helpers::get_path()
// }
//
// fn get_current_sub_route(route: &str) -> SubRoute {
//     if route.starts_with("/timetracking") {
//         SubRoute::Time
//     } else if route.starts_with("/inventory") {
//         SubRoute::Inventory
//     } else if route.starts_with("/settings") {
//         SubRoute::Settings
//     } else if route.starts_with("/work") {
//         SubRoute::Work
//     } else {
//         // Index must always come last!
//         SubRoute::Index
//     }
// }
//
// #[component]
// pub fn NavMain() -> impl IntoView {
//     let t = store_value(t);
//     let current_route = create_rw_signal(get_initial_path());
//     let current_sub_menu = create_rw_signal(get_current_sub_route(
//         current_route.get_untracked().as_str(),
//     ));
//     let current_sub_menu_class =
//         create_rw_signal(current_sub_menu.get_untracked().as_class().to_string());
//     provide_context(current_route);
//     provide_context(t);
//
//     // we need to provide a context for the different nav sub menus too to save some state if
//     // they are expanded or not
//     // can be used later on to easily "just open all nav's" if a user is coming from a
//     // noscript context
//     provide_context(NavContext::new());
//     let is_main_expanded = use_context::<NavContext>().unwrap().expanded_navs.main_nav;
//
//     let burger_class = create_rw_signal(Some("nav-burger-x"));
//     let nav_main_class = create_rw_signal("expanded");
//     let icon_width = "40px";
//
//     let collapse = move || {
//         burger_class.set(None);
//         nav_main_class.set("collapsed");
//         is_main_expanded.set(false);
//     };
//
//     let expand = move || {
//         burger_class.set(Some("nav-burger-x"));
//         nav_main_class.set("expanded");
//         is_main_expanded.set(true);
//     };
//
//     // checks if the nav should be collapsed depending on the inner window width
//     let check_should_collapse = move || {
//         let width = window().inner_width().unwrap().unchecked_into_f64();
//         let is_main_exp = is_main_expanded.get_untracked();
//
//         if width < 900.0 && is_main_exp {
//             collapse();
//         } else if width >= 900.0 && !is_main_exp {
//             expand();
//         }
//     };
//
//     create_effect(move |_| {
//         let width = window().inner_width().unwrap().unchecked_into_f64();
//         if width < 900.0 {
//             collapse();
//         }
//
//         window_event_listener(ev::resize, move |_ev| {
//             check_should_collapse();
//         });
//     });
//
//     create_effect(move |_| {
//         let curr = current_route.get();
//         let new_sub = get_current_sub_route(curr.as_str());
//         // important to to a get_untracked -> only update if necessary to prevent re-renderings
//         // on "internal" route changes
//         if current_sub_menu.get_untracked() != new_sub {
//             current_sub_menu_class.set(new_sub.as_class().to_string());
//             current_sub_menu.set(new_sub);
//         } else {
//             check_should_collapse();
//         }
//     });
//
//     let on_burger_click = move |_ev: MouseEvent| {
//         if is_main_expanded.get_untracked() {
//             collapse();
//         } else {
//             expand();
//         }
//     };
//
//     view! {
//         <nav class=move || nav_main_class.get()>
//             <div id="nav-main-menu-wrapper">
//                 <div id="nav-main-menu">
//                     <div
//                         id="nav-burger"
//                         class=move || burger_class.get()
//                         on:click=on_burger_click
//                     >
//                         <div class="nav-burger-bar1"></div>
//                         <div class="nav-burger-bar2"></div>
//                         <div class="nav-burger-bar3"></div>
//                     </div>
//
//                     <div id="nav-logo">
//                         <img src="/static/nav-logo.jpg" alt="Nav Logo" width="100%"/>
//                     </div>
//
//                     <NavMainEntry href="/">
//                         <IconHomeSolid width=icon_width/>
//                     </NavMainEntry>
//
//                     <NavMainEntry href="/timetracking">
//                         <IconClockSolid width=icon_width/>
//                     </NavMainEntry>
//
//                     <NavMainEntry href="/work">
//                         <IconScissorsSolid width=icon_width/>
//                     </NavMainEntry>
//
//                     <NavMainEntry href="/inventory">
//                         <IconDatabaseSolid width=icon_width/>
//                     </NavMainEntry>
//
//                     <NavMainEntry href="/files">
//                         <IconFolderSolid width=icon_width/>
//                     </NavMainEntry>
//
//                     <NavMainEntry href="/users">
//                         <IconUsersSolid width=icon_width/>
//                     </NavMainEntry>
//
//                     <NavMainEntry href="/settings">
//                         <IconWrenchScrewSolid width=icon_width/>
//                     </NavMainEntry>
//
//                     <NavMainEntry href="/logout">
//                         <IconLogoutSolid width=icon_width/>
//                     </NavMainEntry>
//
//                     // if you add a new route, don't forget to add it to the SubRoute enum above
//                     // and down below to the `if ![..]` list
//                 </div>
//                 <div id="nav-sub-menu-wrapper" class=move || current_sub_menu_class.get()>
//                     {move || {
//                         match current_sub_menu.get() {
//                             SubRoute::Inventory => view! { <NavInventory/> },
//                             SubRoute::Time => view! { <NavTime/> },
//                             SubRoute::Work => view! { <NavWork/> },
//                             _ => view! { <NavIndex/> },
//                         }
//                     }}
//                 </div>
//             </div>
//         </nav>
//     }
// }
//
// #[component]
// fn NavMainEntry(href: &'static str, children: Children) -> impl IntoView {
//     let href = store_value(href.to_string());
//     let current_route =
//         use_context::<RwSignal<String>>().expect("nav entry signal does not exist in context");
//
//     let class = move || {
//         let curr = current_route.get();
//         if href.get_value() == "/" {
//             let (_, rest) = curr.split_once('/').unwrap();
//             let (route, _) = rest.split_once('/').unwrap_or((rest, ""));
//             if ![
//                 "inventory",
//                 "files",
//                 "users",
//                 "settings",
//                 "timetracking",
//                 "work",
//                 "logout",
//             ]
//             .contains(&route)
//             {
//                 "nav-main-entry nav-main-entry-sel"
//             } else {
//                 "nav-main-entry"
//             }
//         } else if curr.starts_with(&href.get_value()) {
//             "nav-main-entry nav-main-entry-sel"
//         } else {
//             "nav-main-entry"
//         }
//     };
//     let on_click = move |_| current_route.set(href.get_value().clone());
//
//     view! {
//         <div class=move || class()>
//             <A href=href.get_value() class=class on:click=on_click>
//                 {&children()}
//             </A>
//         </div>
//     }
// }
