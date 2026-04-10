use dioxus::prelude::*;
use nostra_media::{Composition, SpringOptions, interpolate, spring};

#[component]
pub fn MediaLab() -> Element {
    let mut current_frame = use_signal(|| 0.0);
    let mut playing = use_signal(|| false);

    let composition = Composition::new("Nostra Promo", 1280, 720, 30.0, 300);

    // Derived values using our math primitives
    let opacity = interpolate(*current_frame.read(), &[0.0, 30.0], &[0.0, 1.0], None);

    let scale = spring(
        *current_frame.read(),
        composition.fps,
        Some(SpringOptions {
            from: 0.8,
            to: 1.0,
            ..Default::default()
        }),
    );

    rsx! {
        div {
            class: "flex flex-col h-full bg-[#111] text-[#ccc] font-sans",

            // Header (39px)
            div {
                class: "h-[39px] px-[16px] flex items-center border-b border-[#2f363d] bg-[#1f2428]",
                span { class: "font-semibold", "{composition.id}" }
                div { class: "flex-1" }
                span { class: "text-[12px] opacity-60", "{composition.width}x{composition.height} @ {composition.fps}fps" }
            }

            // Preview Area
            div {
                class: "flex-1 flex items-center justify-center p-[16px] overflow-hidden bg-black",
                div {
                    class: "relative shadow-2xl transition-all duration-75",
                    style: "width: {composition.width / 2}px; height: {composition.height / 2}px; background: #222; transform: scale({scale}); opacity: {opacity};",

                    // Center placeholder
                    div {
                        class: "absolute inset-0 flex items-center justify-center",
                        span { class: "text-2xl font-bold", "nostra-media" }
                    }
                }
            }

            // Controls & Timeline
            div {
                class: "flex-none bg-[#1f2428] border-t border-[#2f363d] px-[16px] py-4 flex flex-col gap-6",

                // Seek Bar (5px height, 12px knob pattern)
                div {
                    class: "relative w-full h-[5px] bg-[#2f363d] rounded-full group cursor-pointer",
                    onclick: move |e| {
                        // Simplified seek for Dioxus 0.7 compatibility
                        let percent = e.data().page_coordinates().x / 1280.0; // Approximation
                        current_frame.set(percent * composition.duration_in_frames as f64);
                    },
                    div {
                        class: "absolute top-0 left-0 h-full bg-[#0b84f3] rounded-full",
                        style: "width: {(*current_frame.read() / composition.duration_in_frames as f64) * 100.0}%;"
                    }
                    div {
                        class: "absolute top-1/2 -ml-[6px] -mt-[6px] w-[12px] h-[12px] bg-white rounded-full shadow opacity-0 group-hover:opacity-100 transition-opacity",
                        style: "left: {(*current_frame.read() / composition.duration_in_frames as f64) * 100.0}%;"
                    }
                }

                // Buttons
                div {
                    class: "flex items-center gap-6",
                    button {
                        class: "w-8 h-8 flex items-center justify-center hover:text-white transition-colors",
                        onclick: move |_| playing.toggle(),
                        if *playing.read() {
                            "pause"
                        } else {
                            "play"
                        }
                    }
                }

                // Timeline Tracks (Production Style)
                div {
                    class: "flex flex-col gap-[2px]",

                    // Track 1: Video (50px)
                    div {
                        class: "h-[50px] bg-[#111] border border-[#2f363d] rounded relative group overflow-hidden",
                        div { class: "absolute left-0 top-0 bottom-0 w-[80%] bg-[#0b84f3] opacity-20" }
                        div { class: "absolute left-2 top-2 text-[10px] font-mono opacity-60", "video_layer_0" }

                        // Mock Keyframe
                        div {
                            class: "absolute top-1/2 -mt-1 w-2 h-2 bg-white rotate-45 border border-black cursor-move",
                            style: "left: 30%;",
                        }
                    }

                    // Track 2: Audio (25px)
                    div {
                        class: "h-[25px] bg-[#111] border border-[#2f363d] rounded relative group overflow-hidden",
                        div { class: "absolute left-0 top-0 bottom-0 w-[100%] bg-green-500 opacity-20" }
                        div { class: "absolute left-2 top-1 text-[10px] font-mono opacity-60", "audio_background" }
                    }
                }
            }
        }
    }
}
