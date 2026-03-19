use cortex_runtime::GatewayLegacyDispatchMode;
use std::env;

pub const DEFAULT_GATEWAY_PORT: u16 = 3000;

pub fn gateway_port() -> u16 {
    gateway_port_with_note().0
}

pub fn gateway_base() -> String {
    format!("http://127.0.0.1:{}", gateway_port())
}

pub fn gateway_legacy_dispatch_mode() -> GatewayLegacyDispatchMode {
    let raw = env::var("CORTEX_GATEWAY_LEGACY_DISPATCH_MODE")
        .unwrap_or_else(|_| "in_process".to_string())
        .trim()
        .to_ascii_lowercase();

    match raw.as_str() {
        "http_loopback" => GatewayLegacyDispatchMode::HttpLoopback,
        _ => GatewayLegacyDispatchMode::InProcess,
    }
}

pub fn gateway_port_with_note() -> (u16, Option<String>) {
    let raw = match env::var("CORTEX_GATEWAY_PORT") {
        Ok(value) => value.trim().to_string(),
        Err(_) => return (DEFAULT_GATEWAY_PORT, None),
    };

    if raw.is_empty() {
        return (
            DEFAULT_GATEWAY_PORT,
            Some("CORTEX_GATEWAY_PORT is empty; using default 3000".to_string()),
        );
    }

    match raw.parse::<u16>() {
        Ok(0) => (
            DEFAULT_GATEWAY_PORT,
            Some("CORTEX_GATEWAY_PORT cannot be 0; using default 3000".to_string()),
        ),
        Ok(port) => (port, None),
        Err(_) => (
            DEFAULT_GATEWAY_PORT,
            Some(format!(
                "CORTEX_GATEWAY_PORT='{}' is invalid; using default 3000",
                raw
            )),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_lock() -> &'static std::sync::Mutex<()> {
        use std::sync::{LazyLock, Mutex};

        static LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
        &LOCK
    }

    #[test]
    fn defaults_to_3000_when_unset() {
        let _guard = env_lock().lock().unwrap();
        std::env::remove_var("CORTEX_GATEWAY_PORT");
        let (port, note) = gateway_port_with_note();
        assert_eq!(port, 3000);
        assert!(note.is_none());
    }

    #[test]
    fn accepts_valid_port() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("CORTEX_GATEWAY_PORT", "3555");
        let (port, note) = gateway_port_with_note();
        assert_eq!(port, 3555);
        assert!(note.is_none());
    }

    #[test]
    fn invalid_port_falls_back_with_note() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("CORTEX_GATEWAY_PORT", "invalid");
        let (port, note) = gateway_port_with_note();
        assert_eq!(port, 3000);
        assert!(note.is_some());
    }

    #[test]
    fn defaults_to_in_process_dispatch_mode() {
        let _guard = env_lock().lock().unwrap();
        std::env::remove_var("CORTEX_GATEWAY_LEGACY_DISPATCH_MODE");
        assert_eq!(
            gateway_legacy_dispatch_mode(),
            GatewayLegacyDispatchMode::InProcess
        );
    }

    #[test]
    fn parses_http_loopback_dispatch_mode() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("CORTEX_GATEWAY_LEGACY_DISPATCH_MODE", "http_loopback");
        assert_eq!(
            gateway_legacy_dispatch_mode(),
            GatewayLegacyDispatchMode::HttpLoopback
        );
    }
}
