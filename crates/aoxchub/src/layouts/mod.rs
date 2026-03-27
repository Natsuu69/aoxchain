// src/layouts/mod.rs içinde: pub mod admin_layout; pub use admin_layout::AdminLayout;

use dioxus::prelude::*;
use crate::components::navigation::{Sidebar, Header};

#[component]
pub fn AdminLayout() -> Element {
    rsx! {
        div { 
            class: "flex h-screen w-full bg-[#050507] text-slate-300 font-sans overflow-hidden relative",
            
            // Arka plan ışık efektleri (Glassmorphism için derinlik sağlar)
            div { class: "absolute -top-40 -left-40 w-[500px] h-[500px] bg-purple-900/20 rounded-full blur-[120px] pointer-events-none" }
            div { class: "absolute top-1/2 -right-20 w-[400px] h-[400px] bg-blue-900/10 rounded-full blur-[100px] pointer-events-none" }

            // Sabit Sidebar
            Sidebar {}

            div { class: "flex flex-col flex-1 relative z-10",
                // Üst Bar
                Header {}

                // Dinamik İçerik Alanı
                main { class: "flex-1 overflow-y-auto p-6 md:p-10 custom-scrollbar",
                    Outlet::<crate::Route> {}
                }
            }
        }
    }
}
