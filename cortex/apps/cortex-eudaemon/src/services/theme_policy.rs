use cortex_domain::theme::policy as domain_theme_policy;
use cortex_runtime::{RuntimeError, ports::ThemePolicyAdapter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy)]
pub enum MotionPolicy {
    System,
    Reduced,
    Full,
}

#[derive(Debug, Clone, Copy)]
pub enum ContrastPreference {
    System,
    More,
    Less,
}

#[derive(Debug, Clone, Copy)]
pub enum ThemeName {
    Cortex,
}

pub struct ThemeTokens {
    pub policy: ThemeDefaultPolicy,
}

#[derive(Clone)]
pub struct ThemeDefaultPolicy {
    pub default_motion_policy: MotionPolicy,
    pub default_contrast_preference: ContrastPreference,
}

pub fn theme_tokens(_name: ThemeName) -> ThemeTokens {
    ThemeTokens {
        policy: ThemeDefaultPolicy {
            default_motion_policy: MotionPolicy::System,
            default_contrast_preference: ContrastPreference::System,
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemePolicyPreferences {
    pub motion_policy: String,
    pub density: String,
    pub contrast_preference: String,
}

impl Default for ThemePolicyPreferences {
    fn default() -> Self {
        let policy = theme_tokens(ThemeName::Cortex).policy.clone();
        Self {
            motion_policy: motion_policy_to_str(policy.default_motion_policy).to_string(),
            density: "regular".to_string(),
            contrast_preference: contrast_to_str(policy.default_contrast_preference).to_string(),
        }
    }
}

static CACHE: OnceLock<Mutex<ThemePolicyPreferences>> = OnceLock::new();

pub fn current_theme_policy() -> ThemePolicyPreferences {
    let cache = CACHE.get_or_init(|| Mutex::new(load_from_disk().unwrap_or_default()));
    match cache.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => ThemePolicyPreferences::default(),
    }
}

pub fn persist_theme_policy(
    requested: ThemePolicyPreferences,
) -> Result<ThemePolicyPreferences, String> {
    let normalized = normalize_preferences(requested);
    let path = theme_policy_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let payload = serde_json::to_string_pretty(&normalized).map_err(|err| err.to_string())?;
    fs::write(path, payload).map_err(|err| err.to_string())?;

    let cache = CACHE.get_or_init(|| Mutex::new(ThemePolicyPreferences::default()));
    if let Ok(mut guard) = cache.lock() {
        *guard = normalized.clone();
    }

    Ok(normalized)
}

pub fn normalize_preferences(requested: ThemePolicyPreferences) -> ThemePolicyPreferences {
    let normalized =
        domain_theme_policy::normalize_preferences(domain_theme_policy::ThemePolicyPreferences {
            motion_policy: requested.motion_policy,
            density: requested.density,
            contrast_preference: requested.contrast_preference,
        });
    ThemePolicyPreferences {
        motion_policy: normalized.motion_policy,
        density: normalized.density,
        contrast_preference: normalized.contrast_preference,
    }
}

pub fn theme_policy_style(preferences: &ThemePolicyPreferences) -> String {
    domain_theme_policy::theme_policy_style(&domain_theme_policy::ThemePolicyPreferences {
        motion_policy: preferences.motion_policy.clone(),
        density: preferences.density.clone(),
        contrast_preference: preferences.contrast_preference.clone(),
    })
}

pub struct DesktopThemePolicyAdapter;

impl ThemePolicyAdapter for DesktopThemePolicyAdapter {
    fn current(&self) -> Result<domain_theme_policy::ThemePolicyPreferences, RuntimeError> {
        let current = current_theme_policy();
        Ok(domain_theme_policy::ThemePolicyPreferences {
            motion_policy: current.motion_policy,
            density: current.density,
            contrast_preference: current.contrast_preference,
        })
    }

    fn persist(
        &self,
        requested: domain_theme_policy::ThemePolicyPreferences,
    ) -> Result<domain_theme_policy::ThemePolicyPreferences, RuntimeError> {
        let saved = persist_theme_policy(ThemePolicyPreferences {
            motion_policy: requested.motion_policy,
            density: requested.density,
            contrast_preference: requested.contrast_preference,
        })
        .map_err(RuntimeError::Storage)?;

        Ok(domain_theme_policy::ThemePolicyPreferences {
            motion_policy: saved.motion_policy,
            density: saved.density,
            contrast_preference: saved.contrast_preference,
        })
    }
}

fn load_from_disk() -> Result<ThemePolicyPreferences, String> {
    let path = theme_policy_path();
    if !path.exists() {
        return Ok(ThemePolicyPreferences::default());
    }

    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let decoded =
        serde_json::from_str::<ThemePolicyPreferences>(&raw).map_err(|err| err.to_string())?;
    Ok(normalize_preferences(decoded))
}

fn theme_policy_path() -> PathBuf {
    if let Ok(path) = std::env::var("CORTEX_THEME_POLICY_PATH") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    home::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".cortex")
        .join("theme_policy_preferences.json")
}

fn motion_policy_to_str(value: MotionPolicy) -> &'static str {
    match value {
        MotionPolicy::System => "system",
        MotionPolicy::Reduced => "reduced",
        MotionPolicy::Full => "full",
    }
}

fn contrast_to_str(value: ContrastPreference) -> &'static str {
    match value {
        ContrastPreference::System => "system",
        ContrastPreference::More => "more",
        ContrastPreference::Less => "less",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_normalization_round_trip() {
        let input = ThemePolicyPreferences {
            motion_policy: "REDUCED".to_string(),
            density: "CoMFortable".to_string(),
            contrast_preference: "MORE".to_string(),
        };

        let normalized = normalize_preferences(input);
        assert_eq!(normalized.motion_policy, "reduced");
        assert_eq!(normalized.density, "comfortable");
        assert_eq!(normalized.contrast_preference, "more");
    }

    #[test]
    fn policy_style_includes_density_and_motion_controls() {
        let css = theme_policy_style(&ThemePolicyPreferences {
            motion_policy: "reduced".to_string(),
            density: "compact".to_string(),
            contrast_preference: "system".to_string(),
        });

        assert!(css.contains("--cortex-density-scale"));
        assert!(css.contains("transition-duration: 0.01ms"));
    }
}
