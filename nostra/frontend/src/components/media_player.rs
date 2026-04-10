use dioxus::prelude::*;
use nostra_media::Composition;

#[derive(Props, Clone, PartialEq)]
pub struct MediaPlayerProps {
    pub composition_id: String,
    #[props(default = false)]
    pub auto_play: bool,
    #[props(default = false)]
    pub _loop: bool,
    #[props(default = true)]
    pub show_controls: bool,
    #[props(default = 0)]
    pub initial_frame: u32,
}

#[component]
pub fn MediaPlayer(props: MediaPlayerProps) -> Element {
    let mut current_frame = use_signal(|| props.initial_frame as f64);
    let mut playing = use_signal(|| props.auto_play);

    // In a real app, we'd fetch the composition from a service
    let composition = Composition::new(&props.composition_id, 1280, 720, 30.0, 300);

    rsx! {
        div {
            class: "media-player relative bg-black aspect-video overflow-hidden group",

            // Render Surface
            div {
                class: "absolute inset-0 flex items-center justify-center",
                span { class: "text-white opacity-20", "Surface: {props.composition_id}" }
            }

            // Controls Overlay
            if props.show_controls {
                div {
                    class: "absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-black/80 to-transparent opacity-0 group-hover:opacity-100 transition-opacity flex items-center px-4 gap-4",

                    button {
                        class: "text-white hover:scale-110 transition-transform",
                        onclick: move |_| playing.toggle(),
                        if *playing.read() { "Pause" } else { "Play" }
                    }

                    div {
                        class: "flex-1 h-1 bg-white/20 rounded-full relative cursor-pointer",
                        onclick: move |e| {
                            // Simplified seek for Dioxus 0.7 compatibility
                            let percent = e.data().page_coordinates().x / 1280.0;
                            current_frame.set(percent * composition.duration_in_frames as f64);
                        },
                        div {
                            class: "absolute top-0 left-0 h-full bg-blue-500 rounded-full",
                            style: "width: {(*current_frame.read() / composition.duration_in_frames as f64) * 100.0}%;"
                        }
                    }

                    span { class: "text-xs text-white/60 tabular-nums",
                        "{current_frame.read().round()} / {composition.duration_in_frames}"
                    }
                }
            }
        }
    }
}
