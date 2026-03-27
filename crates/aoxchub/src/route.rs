use dioxus::prelude::*;
use crate::layouts::AdminLayout;
use crate::views::{Home, Wallet, Nodes};

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AdminLayout)]
        #[route("/")]
        Home {},
        #[route("/wallet")]
        Wallet {},
        #[route("/nodes")]
        Nodes {},
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}
