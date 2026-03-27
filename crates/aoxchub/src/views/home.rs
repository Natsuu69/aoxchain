use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::state::GlobalChainState;
use crate::types::LaneStatus;

#[component]
pub fn Home() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();
    let snapshot = chain.read().clone();

    let total_tps: f32 = snapshot.lanes.iter().map(|lane| lane.tps).sum();
    let offline_nodes = snapshot
        .nodes
        .iter()
        .filter(|node| !node.online)
        .count();

    rsx! {
        div { class: "space-y-6",
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "AOXCHUB Overview" }
                p {
                    class: "text-sm text-slate-300",
                    "Unified visibility into block production, execution lanes, validator health, and network operating posture."
                }
            }

            div { class: "grid gap-4 md:grid-cols-2 xl:grid-cols-4",
                MetricCard {
                    title: "Current Block",
                    value: format!("#{}", snapshot.height),
                    hint: "Latest finalized L1 height".to_string()
                }
                MetricCard {
                    title: "Aggregate TPS",
                    value: format!("{total_tps:.1}"),
                    hint: "Combined throughput across execution lanes".to_string()
                }
                MetricCard {
                    title: "Network Health",
                    value: format!("{:.2}%", snapshot.network_health),
                    hint: format!("{offline_nodes} offline / {} active", snapshot.active_nodes)
                }
                MetricCard {
                    title: "Total Staked",
                    value: format!("{} AOXC", snapshot.total_staked),
                    hint: "Assets committed to network security".to_string()
                }
            }

            GlassSurface { class: Some("p-5".to_string()), intensity: Some("low"),
                div { class: "space-y-4",
                    div { class: "space-y-1",
                        h3 { class: "text-lg font-semibold text-white", "Execution Lanes" }
                        p {
                            class: "text-sm text-slate-300",
                            "Lane-level throughput, load distribution, checkpoint continuity, and runtime activity status."
                        }
                    }

                    if snapshot.lanes.is_empty() {
                        div {
                            class: "rounded-xl border border-dashed border-white/10 bg-white/5 px-4 py-6 text-sm text-slate-400",
                            "No lane telemetry is currently available."
                        }
                    } else {
                        div { class: "space-y-3",
                            for lane in snapshot.lanes {
                                LaneRow { lane: lane }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Wallet() -> Element {
    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "Wallet & Treasury" }
                p {
                    class: "text-sm text-slate-300",
                    "Operational visibility for treasury custody, reward distribution cadence, and controlled hot-wallet exposure."
                }
            }

            GlassSurface { class: Some("p-5".to_string()),
                div { class: "space-y-3",
                    TreasuryRow {
                        title: "Treasury Balance",
                        value: "143,920,000 AOXC",
                        hint: "Primary reserve under multisig custody"
                    }
                    TreasuryRow {
                        title: "Reward Distribution Window",
                        value: "Every 6 hours",
                        hint: "Scheduled validator reward settlement period"
                    }
                    TreasuryRow {
                        title: "Hot Wallet Exposure",
                        value: "4.2%",
                        hint: "Controlled operational liquidity relative to total funds"
                    }
                }
            }
        }
    }
}

#[component]
pub fn Nodes() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();
    let snapshot = chain.read().clone();

    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "Validator Nodes" }
                p {
                    class: "text-sm text-slate-300",
                    "Current validator footprint, regional placement, latency posture, and online status."
                }
            }

            GlassSurface { class: Some("p-5".to_string()),
                div { class: "mb-4 flex flex-wrap items-center justify-between gap-3",
                    p { class: "text-sm text-slate-300", "Active nodes: {snapshot.active_nodes}" }
                    p { class: "text-xs uppercase tracking-wide text-slate-500", "Consensus participant registry" }
                }

                if snapshot.nodes.is_empty() {
                    div {
                        class: "rounded-xl border border-dashed border-white/10 bg-white/5 px-4 py-6 text-sm text-slate-400",
                        "No validator node data is currently available."
                    }
                } else {
                    table { class: "w-full text-left text-sm",
                        thead { class: "text-slate-400",
                            tr { class: "border-b border-white/10",
                                th { class: "pb-3", "Node" }
                                th { class: "pb-3", "Region" }
                                th { class: "pb-3", "Latency" }
                                th { class: "pb-3", "Status" }
                            }
                        }
                        tbody {
                            for node in snapshot.nodes {
                                tr { class: "border-t border-white/10",
                                    td { class: "py-3 text-white", "{node.id}" }
                                    td { class: "py-3 text-slate-300", "{node.region}" }
                                    td { class: "py-3 text-slate-300", "{node.latency_ms} ms" }
                                    td {
                                        class: if node.online {
                                            "py-3 text-emerald-300"
                                        } else {
                                            "py-3 text-rose-300"
                                        },
                                        if node.online { "Online" } else { "Offline" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn NotFoundPage(segments: Vec<String>) -> Element {
    let path = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    };

    rsx! {
        GlassSurface { class: Some("p-8".to_string()),
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "Page Not Found" }
                p { class: "text-sm text-slate-300", "Requested route: {path}" }
                p { class: "text-sm text-slate-400", "Please select a valid module from the navigation menu." }
            }
        }
    }
}

#[component]
fn MetricCard(title: &'static str, value: String, hint: String) -> Element {
    rsx! {
        GlassSurface { class: Some("p-4".to_string()), intensity: Some("low"),
            p { class: "text-xs uppercase tracking-wide text-slate-400", "{title}" }
            p { class: "mt-2 text-xl font-semibold text-white break-all", "{value}" }
            p { class: "mt-1 text-xs text-slate-400", "{hint}" }
        }
    }
}

#[component]
fn LaneRow(lane: LaneStatus) -> Element {
    let load_width = format!("{}%", lane.load_percent);
    let activity_label = if lane.is_active { "Active" } else { "Idle" };
    let activity_class = if lane.is_active {
        "text-emerald-300"
    } else {
        "text-amber-300"
    };

    rsx! {
        div { class: "rounded-xl border border-white/10 bg-white/5 p-4",
            div { class: "flex flex-col gap-3 md:flex-row md:items-center md:justify-between",
                div { class: "space-y-1",
                    p { class: "font-semibold text-white", "{lane.kind:?}" }
                    p { class: "text-xs text-slate-400", "Checkpoint: {lane.last_checkpoint}" }
                }

                div { class: "flex flex-wrap items-center gap-3",
                    p { class: "text-sm text-slate-300", "{lane.tps:.1} TPS" }
                    p { class: "text-sm text-slate-300", "Load: {lane.load_percent}%" }
                    p { class: "text-sm {activity_class}", "{activity_label}" }
                }
            }

            div { class: "mt-3 h-2 rounded-full bg-slate-800",
                div {
                    class: "h-full rounded-full bg-blue-500 transition-all",
                    style: "width: {load_width}"
                }
            }
        }
    }
}

#[component]
fn TreasuryRow(title: &'static str, value: &'static str, hint: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-white/10 bg-white/5 px-4 py-4",
            p { class: "text-xs uppercase tracking-wide text-slate-400", "{title}" }
            p { class: "mt-2 text-lg font-semibold text-white", "{value}" }
            p { class: "mt-1 text-xs text-slate-400", "{hint}" }
        }
    }
}
