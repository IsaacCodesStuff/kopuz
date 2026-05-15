use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;

#[derive(Props, Clone, PartialEq)]
pub struct RadioProps {
    pub config: Signal<config::AppConfig>,
}

#[derive(PartialEq, Clone)]
struct RadioStream {
    name: &'static str,
    id: &'static str,
    icon: &'static str,
}

#[derive(PartialEq, Clone)]
struct RadioStation {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    icon: &'static str,
    streams: &'static [RadioStream],
}

const STATIONS: &[RadioStation] = &[
    RadioStation {
        id: "listen_moe",
        name: "LISTEN.moe",
        description: "Anime and Korean pop music, 24/7.",
        icon: "fa-solid fa-radio",
        streams: &[
            RadioStream { name: "J-Pop", id: "jpop", icon: "fa-solid fa-music" },
            RadioStream { name: "K-Pop", id: "kpop", icon: "fa-solid fa-compact-disc" },
        ],
    },
    RadioStation {
        id: "j1",
        name: "J1 Tokyo",
        description: "Current chart hits and classic songs from Japan.",
        icon: "fa-solid fa-radio",
        streams: &[
            RadioStream { name: "J1 HITS", id: "J1HITS", icon: "fa-solid fa-fire" },
            RadioStream { name: "J1 GOLD", id: "J1GOLD", icon: "fa-solid fa-compact-disc" },
        ],
    },
    RadioStation {
        id: "doujinstyle",
        name: "Doujinstyle",
        description: "Doujin music, video game soundtracks, and more.",
        icon: "fa-solid fa-radio",
        streams: &[
            RadioStream { name: "Live Stream", id: "main", icon: "fa-solid fa-play" },
        ],
    },
    RadioStation {
        id: "vocaloid",
        name: "Vocaloid Radio",
        description: "Vocaloid Hits from Japan. We play it all!",
        icon: "fa-solid fa-radio",
        streams: &[
            RadioStream { name: "Live Stream", id: "main", icon: "fa-solid fa-play" },
        ],
    }
];

#[component]
pub fn Radio(props: RadioProps) -> Element {
    let mut ctrl = use_context::<PlayerController>();

    rsx! {
        div { class: "p-8 w-full h-full flex flex-col overflow-y-auto bg-black/20",
            div { class: "mb-8",
                h1 { class: "text-4xl font-extrabold text-transparent bg-clip-text from-indigo-400 to-purple-400 mb-2",
                    "Radio Stations"
                }
                p { class: "text-white/60 text-lg",
                    "Tune in to live internet radio streams"
                }
            }

            div { class: "flex flex-col gap-6 max-w-4xl",
                for station in STATIONS {
                    div {
                        class: "group relative rounded-2xl overflow-hidden border border-white/5 transition-all duration-300 hover:border-white/20 hover:bg-white/5 cursor-pointer hover:shadow-[0_0_30px_rgba(255,255,255,0.03)]",
                        onclick: move |_| {
                            if !station.streams.is_empty() {
                                ctrl.play_radio(station.id, station.streams[0].id);
                            }
                        },
                        div { class: "absolute inset-0 opacity-50 pointer-events-none group-hover:opacity-70 transition-opacity" }
                        div { class: "p-6 relative z-10 flex flex-col md:flex-row md:items-center gap-6",
                            // Station Info
                            div { class: "flex items-center gap-4 flex-1",
                                div { class: "w-14 h-14 rounded-full bg-white/10 flex items-center justify-center text-white/80 shadow-[0_0_15px_rgba(255,255,255,0.05)] group-hover:scale-105 transition-transform",
                                    i { class: "{station.icon} text-2xl" }
                                }
                                div {
                                    h2 { class: "text-2xl font-bold text-white mb-1", "{station.name}" }
                                    p { class: "text-white/60 text-sm", "{station.description}" }
                                }
                            }

                            if station.streams.len() > 1 {
                                div { class: "flex flex-wrap items-center gap-3",
                                    for stream in station.streams {
                                        button {
                                            class: "px-4 py-2 rounded-xl bg-black/40 hover:bg-white/20 border border-white/10 hover:border-white/40 text-white transition-colors flex items-center gap-2",
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                ctrl.play_radio(station.id, stream.id);
                                            },
                                            i { class: "{stream.icon} text-sm text-white/70" }
                                            "{stream.name}"
                                        }
                                    }
                                }
                            } else {
                                div { class: "flex flex-wrap items-center gap-3 text-white/50 group-hover:text-white/80 transition-colors",
                                    i { class: "fa-solid fa-play text-xl" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
