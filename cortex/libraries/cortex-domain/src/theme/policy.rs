use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThemePolicyPreferences {
    pub motion_policy: String,
    pub density: String,
    pub contrast_preference: String,
}

impl Default for ThemePolicyPreferences {
    fn default() -> Self {
        Self {
            motion_policy: "system".to_string(),
            density: "regular".to_string(),
            contrast_preference: "system".to_string(),
        }
    }
}

pub fn normalize_preferences(requested: ThemePolicyPreferences) -> ThemePolicyPreferences {
    ThemePolicyPreferences {
        motion_policy: normalize_motion_policy(&requested.motion_policy),
        density: normalize_density(&requested.density),
        contrast_preference: normalize_contrast_preference(&requested.contrast_preference),
    }
}

pub fn theme_policy_style(preferences: &ThemePolicyPreferences) -> String {
    let density_scale = match preferences.density.as_str() {
        "compact" => 0.92,
        "comfortable" => 1.08,
        _ => 1.0,
    };

    let mut css = format!(
        ":root {{ --cortex-density-scale: {density_scale}; }}\n:root[data-cortex-density='compact'] {{ --cortex-density-scale: 0.92; }}\n:root[data-cortex-density='regular'] {{ --cortex-density-scale: 1.0; }}\n:root[data-cortex-density='comfortable'] {{ --cortex-density-scale: 1.08; }}\n"
    );

    if preferences.motion_policy == "reduced" {
        css.push_str(
            "* { animation-duration: 0.01ms !important; animation-iteration-count: 1 !important; transition-duration: 0.01ms !important; scroll-behavior: auto !important; }\n",
        );
    }

    match preferences.contrast_preference.as_str() {
        "more" => css.push_str(":root { --cortex-contrast-filter: contrast(1.12); }\n"),
        "less" => css.push_str(":root { --cortex-contrast-filter: contrast(0.96); }\n"),
        _ => css.push_str(":root { --cortex-contrast-filter: none; }\n"),
    }

    css
}

pub fn normalize_motion_policy(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "full" => "full".to_string(),
        "reduced" => "reduced".to_string(),
        _ => "system".to_string(),
    }
}

pub fn normalize_density(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "compact" => "compact".to_string(),
        "comfortable" => "comfortable".to_string(),
        _ => "regular".to_string(),
    }
}

pub fn normalize_contrast_preference(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "more" => "more".to_string(),
        "less" => "less".to_string(),
        _ => "system".to_string(),
    }
}
