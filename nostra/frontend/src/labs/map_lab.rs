use crate::components::map_view::MapView;
use crate::types::UserProfile;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct MapLabProps {
    pub user_profile: UserProfile,
    pub on_update_profile: EventHandler<UserProfile>,
}

#[component]
pub fn MapLab(props: MapLabProps) -> Element {
    let profile = &props.user_profile;

    // Clone for closures
    let props_london = props.clone();
    let props_nyc = props.clone();
    let props_tokyo = props.clone();
    let props_reset = props.clone();

    // Mock markers for demo
    let markers = vec![];

    rsx! {
        div {
            class: "flex flex-col h-full p-6 space-y-6 overflow-y-auto",
            div { class: "flex items-center justify-between",
                div {
                    h2 { class: "text-2xl font-bold tracking-tight", "Location Services Lab" }
                    p { class: "text-muted-foreground", "Visualize and verify geospatial context injection." }
                }
                div { class: "flex gap-2",
                    button {
                        class: "px-3 py-1 text-xs font-medium rounded-md border bg-background hover:bg-accent hover:text-accent-foreground",
                        onclick: move |_| {
                            let mut new_profile = props_london.user_profile.clone();
                            new_profile.geo_location = Some(crate::types::GeoLocation {
                                latitude: 51.5074,
                                longitude: -0.1278,
                                precision: Some(10.0),
                            });
                             new_profile.jurisdiction = Some(crate::types::Jurisdiction {
                                country_code: "GB".to_string(),
                                region: Some("England".to_string()),
                                city: Some("London".to_string()),
                            });
                            props_london.on_update_profile.call(new_profile);
                        },
                        "📍 London"
                    }
                    button {
                        class: "px-3 py-1 text-xs font-medium rounded-md border bg-background hover:bg-accent hover:text-accent-foreground",
                        onclick: move |_| {
                            let mut new_profile = props_nyc.user_profile.clone();
                            new_profile.geo_location = Some(crate::types::GeoLocation {
                                latitude: 40.7128,
                                longitude: -74.0060,
                                precision: Some(5.0),
                            });
                            new_profile.jurisdiction = Some(crate::types::Jurisdiction {
                                country_code: "US".to_string(),
                                region: Some("NY".to_string()),
                                city: Some("New York".to_string()),
                            });
                            props_nyc.on_update_profile.call(new_profile);
                        },
                        "📍 NYC"
                    }
                    button {
                        class: "px-3 py-1 text-xs font-medium rounded-md border bg-background hover:bg-accent hover:text-accent-foreground",
                        onclick: move |_| {
                            let mut new_profile = props_tokyo.user_profile.clone();
                            new_profile.geo_location = Some(crate::types::GeoLocation {
                                latitude: 35.6762,
                                longitude: 139.6503,
                                precision: Some(2.0),
                            });
                            new_profile.jurisdiction = Some(crate::types::Jurisdiction {
                                country_code: "JP".to_string(),
                                region: Some("Kanto".to_string()),
                                city: Some("Tokyo".to_string()),
                            });
                            props_tokyo.on_update_profile.call(new_profile);
                        },
                        "📍 Tokyo"
                    }
                    button {
                        class: "px-3 py-1 text-xs font-medium rounded-md border bg-background hover:bg-accent hover:text-accent-foreground text-red-500",
                        onclick: move |_| {
                            let mut new_profile = props_reset.user_profile.clone();
                            new_profile.geo_location = None;
                            new_profile.jurisdiction = None;
                            props_reset.on_update_profile.call(new_profile);
                        },
                        "Reset"
                    }
                }
            }

            div { class: "grid gap-6 md:grid-cols-3",
                 div { class: "md:col-span-2 space-y-4",
                    div { class: "card border rounded-lg shadow-sm bg-card text-card-foreground p-6",
                         h3 { class: "font-semibold mb-4", "World Map" }
                         MapView {
                            location: profile.geo_location.clone(),
                            markers: markers
                         }
                    }
                 }

                 div { class: "space-y-4",
                    div { class: "card border rounded-lg shadow-sm bg-card text-card-foreground p-6",
                        h3 { class: "font-semibold mb-4", "Jurisdiction Context" }
                        div { class: "space-y-2 text-sm",
                            if let Some(jur) = &profile.jurisdiction {
                                div { class: "flex justify-between border-b pb-2",
                                    span { class: "text-muted-foreground", "Country" }
                                    span { class: "font-mono font-medium", "{jur.country_code}" }
                                }
                                if let Some(r) = &jur.region {
                                    div { class: "flex justify-between border-b pb-2",
                                        span { class: "text-muted-foreground", "Region" }
                                        span { class: "font-mono font-medium", "{r}" }
                                    }
                                }
                                if let Some(c) = &jur.city {
                                    div { class: "flex justify-between border-b pb-2",
                                        span { class: "text-muted-foreground", "City" }
                                        span { class: "font-mono font-medium", "{c}" }
                                    }
                                }
                            } else {
                                div { class: "p-4 bg-muted/50 rounded text-muted-foreground italic text-center",
                                    "No jurisdiction inferred from current location."
                                }
                            }
                        }
                    }

                    div { class: "card border rounded-lg shadow-sm bg-card text-card-foreground p-6",
                         h3 { class: "font-semibold mb-4", "Raw Geo-Metadata" }
                         div { class: "bg-muted p-3 rounded font-mono text-xs overflow-x-auto",
                            if let Some(loc) = &profile.geo_location {
                                "{loc:#?}"
                            } else {
                                "None"
                            }
                         }
                    }
                 }
            }
        }
    }
}
