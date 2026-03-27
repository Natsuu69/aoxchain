use dioxus::prelude::*;
use crate::state::GlobalChainState;
use crate::components::glass::GlassSurface;
use diox_lucide::icons::*;

#[component]
pub fn Home() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();

    rsx! {
        div { class: "flex flex-col gap-10 animate-in fade-in duration-700",
            
            // 📊 Top Real-time Metrics
            div { class: "grid grid-cols-1 lg:grid-cols-4 gap-6",
                for (i, label) in ["Current Block", "TPS Aggregate", "Network Health", "Total Staked"].iter().enumerate() {
                    MetricPanel { label: *label, index: i }
                }
            }

            // 🏎️ HyperVM Lane Monitor Section
            div { class: "grid grid-cols-1 xl:grid-cols-3 gap-8",
                GlassSurface { class: "xl:col-span-2 p-10",
                    header { class: "flex justify-between items-center mb-10",
                        div {
                            h2 { class: "text-2xl font-black tracking-tighter text-white", "HyperVM Execution Lanes" }
                            p { class: "text-slate-500 text-sm", "Real-time multi-runtime load distribution" }
                        }
                        ActivityIndicator {}
                    }
                    div { class: "space-y-8",
                        for lane in chain().lanes() {
                            LaneRow { lane: lane }
                        }
                    }
                }

                // 🔐 ZK-Proof Audit Stream
                GlassSurface { class: "p-8",
                    h3 { class: "text-sm font-bold text-blue-400 uppercase tracking-widest mb-6", "Live ZKP Verifications" }
                    div { class: "space-y-4",
                        for _ in 0..6 {
                            ZkpProofItem {}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LaneRow(lane: crate::types::LaneStatus) -> Element {
    let color = if lane.is_active { "bg-blue-500" } else { "bg-slate-700" };
    rsx! {
        div { class: "group relative p-6 rounded-3xl bg-white/[0.01] border border-white/5 hover:bg-white/[0.03] transition-all",
            div { class: "flex justify-between items-center",
                div { class: "flex items-center gap-4",
                    div { class: "p-3 rounded-2xl bg-white/5", Cpu { class: "w-5 h-5 text-blue-400" } }
                    div {
                        h4 { class: "font-bold text-white", "{lane.kind:?}" }
                        p { class: "text-[10px] font-mono text-slate-500", "LATEST_CHECKPOINT: {lane.last_checkpoint}" }
                    }
                }
                div { class: "text-right",
                    span { class: "text-xl font-mono text-white", "{lane.tps} " }
                    span { class: "text-[10px] text-slate-500", "TPS" }
                }
            }
            // Load Bar
            div { class: "mt-4 h-1.5 w-full bg-white/5 rounded-full overflow-hidden",
                div { 
                    class: "h-full {color} shadow-[0_0_15px_rgba(59,130,246,0.5)] transition-all duration-1000", 
                    style: "width: {lane.load_percent}%" 
                }
            }
        }
    }
}
