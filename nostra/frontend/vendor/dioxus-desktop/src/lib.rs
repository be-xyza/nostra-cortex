// Dummy dioxus-desktop

pub mod launch {
    pub fn launch<App, Props, Cfg>(_app: App, _props: Props, _cfg: Cfg)
    where
        App: 'static,
        Props: 'static,
        Cfg: 'static,
    {
        // No-op
    }
}
