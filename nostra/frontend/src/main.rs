#![allow(non_snake_case)]

use dioxus::prelude::*;
mod v2;
use v2::api::{health_snapshot, institutions_seed};
use v2::pages::{HomePage, InstitutionsPage, QuestionsPage};
use v2::types::AppRoute;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut route = use_signal(|| AppRoute::Home);
    let health = health_snapshot();
    let institutions = institutions_seed();

    rsx! {
        div {
            class: "min-h-screen bg-zinc-950 text-zinc-100",
            header {
                class: "px-6 py-4 border-b border-zinc-800 bg-zinc-900/70",
                h1 { class: "text-xl font-semibold", "Nostra Frontend v2 Shell" }
                p { class: "text-sm text-zinc-400", "Incremental restoration path with explicit boundaries." }
            }
            main {
                class: "p-6",
                if route() == AppRoute::Home {
                    HomePage {
                        health: health,
                        on_open_institutions: move |_| route.set(AppRoute::Institutions),
                        on_open_questions: move |_| route.set(AppRoute::Questions)
                    }
                } else if route() == AppRoute::Questions {
                    QuestionsPage {
                        on_back: move |_| route.set(AppRoute::Home)
                    }
                } else {
                    InstitutionsPage {
                        institutions: institutions.clone(),
                        on_back: move |_| route.set(AppRoute::Home)
                    }
                }
            }
        }
    }
}
