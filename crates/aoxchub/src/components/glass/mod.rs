use dioxus::prelude::*;

#[component]
pub fn GlassSurface(
    children: Element, 
    class: Option<String>,
    intensity: Option<&'static str> // "low", "high"
) -> Element {
    let blur = match intensity.unwrap_or("high") {
        "low" => "backdrop-blur-md",
        _ => "backdrop-blur-[40px]",
    };
    
    rsx! {
        div { 
            class: "bg-white/[0.02] {blur} border border-white/10 rounded-[2.5rem] shadow-[0_22px_70px_4px_rgba(0,0,0,0.56)] {class}",
            {children}
        }
    }
}
