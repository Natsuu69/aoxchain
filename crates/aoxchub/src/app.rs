use dioxus::prelude::*;
use crate::route::Route;
use crate::state::provide_global_state;

pub fn App() -> Element {
    // i18n ve Chain State'i burada başlatıyoruz
    provide_global_state();

    rsx! {
        // Material Icons & Tailwind Import
        link { rel: "stylesheet", href: "https://fonts.googleapis.com/icon?family=Material+Icons" }
        style { {include_str!("../assets/main.css")} }
        
        Router::<Route> {}
    }
}
