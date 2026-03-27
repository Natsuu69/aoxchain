use dioxus::prelude::*;
use crate::route::Route;

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside { class: "w-80 h-full bg-[#040406]/80 backdrop-blur-[50px] border-r border-white/5 flex flex-col p-8 z-50",
            // Brand Logo
            div { class: "flex items-center gap-4 mb-16 px-2",
                div { class: "w-12 h-12 bg-gradient-to-tr from-blue-600 via-indigo-600 to-purple-700 rounded-2xl shadow-2xl shadow-blue-500/20 flex items-center justify-center",
                    Activity { class: "text-white w-6 h-6" }
                }
                div {
                    h1 { class: "text-xl font-black tracking-tighter text-white leading-none", "AOXCHAIN" }
                    span { class: "text-[10px] text-blue-500 font-bold tracking-[0.2em] uppercase", "Mission Control" }
                }
            }

            // High-Level Navigation
            nav { class: "flex-1 space-y-2",
                NavSection { title: "Infrastructure" }
                SidebarLink { to: Route::Home {}, icon: LayoutDashboard {}, label: "Overview" }
                SidebarLink { to: Route::Home {}, icon: Lan {}, label: "HyperVM Monitor" }
                SidebarLink { to: Route::Home {}, icon: Map {}, label: "Consensus Map" }
                
                NavSection { title: "Financials" }
                SidebarLink { to: Route::Home {}, icon: WalletMinimal {}, label: "Vaults & Assets" }
                SidebarLink { to: Route::Home {}, icon: HandCoins {}, label: "Staking Hub" }
            }
            
            // User Session Glass Box
            UserSessionCard {}
        }
    }
}
