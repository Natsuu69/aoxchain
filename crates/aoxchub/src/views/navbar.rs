use crate::Route;
use dioxus::prelude::*;

const NAV_ITEMS: [NavItem; 8] = [
    NavItem::new("Overview", "#overview", "overview"),
    NavItem::new("Dashboard", "#dashboard", "dashboard"),
    NavItem::new("Validators", "#validators", "validators"),
    NavItem::new("RPC Monitor", "#rpc-monitor", "rpc-monitor"),
    NavItem::new("Bridge", "#bridge", "bridge"),
    NavItem::new("Governance", "#governance", "governance"),
    NavItem::new("Staking", "#staking", "staking"),
    NavItem::new("Ecosystem", "#ecosystem", "ecosystem"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NavItem {
    label: &'static str,
    href: &'static str,
    key: &'static str,
}

impl NavItem {
    const fn new(label: &'static str, href: &'static str, key: &'static str) -> Self {
        Self { label, href, key }
    }
}

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div {
            class: "app-frame",

            header {
                class: "topbar",
                role: "banner",

                div {
                    class: "topbar-inner",

                    Link {
                        class: "brand",
                        to: Route::Home {},
                        aria_label: "Navigate to AOX Hub home",

                        span {
                            class: "brand-mark",
                            aria_hidden: "true",
                            "AOX"
                        }

                        span {
                            class: "brand-text",
                            "AOX Hub Control Center"
                        }
                    }

                    div {
                        class: "topbar-status",

                        div {
                            class: "network-pill",
                            role: "status",
                            aria_live: "polite",

                            span {
                                class: "network-dot",
                                aria_hidden: "true"
                            }

                            span { "Mainnet Connected" }
                        }
                    }
                }
            }

            div {
                class: "app-layout",

                aside {
                    class: "sidebar glass",
                    role: "navigation",
                    aria_label: "Primary section navigation",

                    div {
                        class: "sidebar-inner",

                        p {
                            class: "sidebar-label",
                            "Navigation"
                        }

                        nav {
                            class: "sidebar-nav",

                            ul {
                                class: "sidebar-nav-list",

                                for item in NAV_ITEMS {
                                    li {
                                        class: "sidebar-nav-item",
                                        key: "{item.key}",

                                        a {
                                            class: "sidebar-nav-link",
                                            href: item.href,
                                            "{item.label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                main {
                    class: "main-content",
                    role: "main",

                    Outlet::<Route> {}
                }
            }

            footer {
                class: "footer",
                role: "contentinfo",

                div {
                    class: "footer-inner",

                    p {
                        "AOX Hub is synchronized with AOXC chain services, validators, bridge relays, and governance streams."
                    }
                }
            }
        }
    }
}
