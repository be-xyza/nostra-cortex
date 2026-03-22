use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub tokens: ThemeTokens,
}

impl Theme {
    pub fn to_css_variables(&self) -> String {
        self.tokens.to_css_variables()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            id: "default-theme".to_string(),
            name: "Cortex Default".to_string(),
            version: "0.0.1".to_string(),
            base: None,
            tokens: ThemeTokens::default(),
        }
    }
}

#[derive(Debug)]
pub struct ThemeManager {
    pub active_theme: Theme,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            active_theme: Theme::default(),
        }
    }

    pub fn load_from_value(&mut self, value: serde_json::Value) -> Result<(), serde_json::Error> {
        let theme: Theme = serde_json::from_value(value)?;
        self.active_theme = theme;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ThemeTokens {
    #[serde(default)]
    pub colors: ColorTokens,
    #[serde(default)]
    pub typography: TypographyTokens,
    #[serde(default)]
    pub spacing: SpacingTokens,
    #[serde(default)]
    pub radii: RadiiTokens,
    #[serde(default)]
    pub shadows: ShadowTokens,
    #[serde(default)]
    pub motion: MotionTokens,
}

impl ThemeTokens {
    pub fn to_css_variables(&self) -> String {
        let mut css = String::new();

        // Colors
        css.push_str(&format!(
            "--ctx-color-background: {};\n",
            self.colors.background
        ));
        css.push_str(&format!("--ctx-color-surface: {};\n", self.colors.surface));
        css.push_str(&format!(
            "--ctx-color-primary: {};\n",
            self.colors.brand.primary
        ));
        css.push_str(&format!(
            "--ctx-color-secondary: {};\n",
            self.colors.brand.secondary
        ));
        css.push_str(&format!(
            "--ctx-color-tertiary: {};\n",
            self.colors.brand.tertiary
        ));

        // Semantic colors
        if !self.colors.semantic.success.is_empty() {
            css.push_str(&format!(
                "--ctx-color-success: {};\n",
                self.colors.semantic.success
            ));
        }
        if !self.colors.semantic.warning.is_empty() {
            css.push_str(&format!(
                "--ctx-color-warning: {};\n",
                self.colors.semantic.warning
            ));
        }
        if !self.colors.semantic.error.is_empty() {
            css.push_str(&format!(
                "--ctx-color-error: {};\n",
                self.colors.semantic.error
            ));
        }
        if !self.colors.semantic.info.is_empty() {
            css.push_str(&format!(
                "--ctx-color-info: {};\n",
                self.colors.semantic.info
            ));
        }

        // Motion Durations
        css.push_str(&format!(
            "--ctx-motion-duration-instant: {};\n",
            self.motion.durations.instant
        ));
        css.push_str(&format!(
            "--ctx-motion-duration-fast: {};\n",
            self.motion.durations.fast
        ));
        css.push_str(&format!(
            "--ctx-motion-duration-normal: {};\n",
            self.motion.durations.normal
        ));
        css.push_str(&format!(
            "--ctx-motion-duration-slow: {};\n",
            self.motion.durations.slow
        ));

        // Motion Easings
        css.push_str(&format!(
            "--ctx-motion-easing-linear: {};\n",
            self.motion.easings.linear
        ));
        css.push_str(&format!(
            "--ctx-motion-easing-ease-in: {};\n",
            self.motion.easings.ease_in
        ));
        css.push_str(&format!(
            "--ctx-motion-easing-ease-out: {};\n",
            self.motion.easings.ease_out
        ));
        css.push_str(&format!(
            "--ctx-motion-easing-ease-in-out: {};\n",
            self.motion.easings.ease_in_out
        ));

        // Motion Transitions
        css.push_str(&format!(
            "--ctx-motion-transition-enter: {};\n",
            self.motion.transitions.enter
        ));
        css.push_str(&format!(
            "--ctx-motion-transition-exit: {};\n",
            self.motion.transitions.exit
        ));

        css
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MotionTokens {
    #[serde(default)]
    pub durations: DurationTokens,
    #[serde(default)]
    pub easings: EasingTokens,
    #[serde(default)]
    pub transitions: TransitionTokens,
}

impl Default for MotionTokens {
    fn default() -> Self {
        Self {
            durations: DurationTokens {
                instant: "0ms".to_string(),
                fast: "150ms".to_string(),
                normal: "300ms".to_string(),
                slow: "500ms".to_string(),
            },
            easings: EasingTokens {
                linear: "linear".to_string(),
                ease_in: "ease-in".to_string(),
                ease_out: "ease-out".to_string(),
                ease_in_out: "ease-in-out".to_string(),
            },
            transitions: TransitionTokens {
                enter: "fade-in".to_string(),
                exit: "fade-out".to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DurationTokens {
    #[serde(default)]
    pub instant: String,
    #[serde(default)]
    pub fast: String,
    #[serde(default)]
    pub normal: String,
    #[serde(default)]
    pub slow: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct EasingTokens {
    #[serde(default)]
    pub linear: String,
    #[serde(default)]
    pub ease_in: String,
    #[serde(default)]
    pub ease_out: String,
    #[serde(default)]
    pub ease_in_out: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransitionTokens {
    #[serde(default)]
    pub enter: String,
    #[serde(default)]
    pub exit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ColorTokens {
    #[serde(default)]
    pub brand: BrandColors,
    #[serde(default)]
    pub semantic: SemanticColors,
    #[serde(default)]
    pub fills: LoudnessColors,
    #[serde(default)]
    pub on: LoudnessColors,
    #[serde(default)]
    pub borders: LoudnessColors,
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default = "default_surface")]
    pub surface: String,
}

fn default_background() -> String {
    "#ffffff".to_string()
}
fn default_surface() -> String {
    "#f5f5f5".to_string()
}

impl Default for ColorTokens {
    fn default() -> Self {
        Self {
            brand: BrandColors::default(),
            semantic: SemanticColors::default(),
            fills: LoudnessColors::default(),
            on: LoudnessColors::default(),
            borders: LoudnessColors::default(),
            background: default_background(),
            surface: default_surface(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrandColors {
    #[serde(default = "default_primary")]
    pub primary: String,
    #[serde(default = "default_secondary")]
    pub secondary: String,
    #[serde(default = "default_tertiary")]
    pub tertiary: String,
}

fn default_primary() -> String {
    "#000000".to_string()
}
fn default_secondary() -> String {
    "#555555".to_string()
}
fn default_tertiary() -> String {
    "#aaaaaa".to_string()
}

impl Default for BrandColors {
    fn default() -> Self {
        Self {
            primary: default_primary(),
            secondary: default_secondary(),
            tertiary: default_tertiary(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SemanticColors {
    #[serde(default)]
    pub success: String,
    #[serde(default)]
    pub warning: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub info: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoudnessColors {
    #[serde(default)]
    pub quiet: String,
    #[serde(default)]
    pub normal: String,
    #[serde(default)]
    pub loud: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TypographyTokens {
    #[serde(default)]
    pub families: HashMap<String, String>,
    #[serde(default)]
    pub sizes: HashMap<String, String>,
    #[serde(default)]
    pub weights: HashMap<String, u16>,
    #[serde(default)]
    pub line_heights: HashMap<String, f32>,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        let mut families = HashMap::new();
        families.insert("body".to_string(), "sans-serif".to_string());
        families.insert("heading".to_string(), "sans-serif".to_string());

        let mut sizes = HashMap::new();
        sizes.insert("body".to_string(), "1rem".to_string());

        let mut weights = HashMap::new();
        weights.insert("normal".to_string(), 400);
        weights.insert("bold".to_string(), 700);

        let mut line_heights = HashMap::new();
        line_heights.insert("body".to_string(), 1.5);

        Self {
            families,
            sizes,
            weights,
            line_heights,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpacingTokens {
    #[serde(default = "default_unit")]
    pub unit: String,
    #[serde(default)]
    pub scale: HashMap<String, String>,
}

fn default_unit() -> String {
    "rem".to_string()
}

impl Default for SpacingTokens {
    fn default() -> Self {
        let mut scale = HashMap::new();
        scale.insert("0".to_string(), "0".to_string());
        scale.insert("1".to_string(), "0.25rem".to_string());

        Self {
            unit: default_unit(),
            scale,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RadiiTokens {
    #[serde(default = "default_sm")]
    pub sm: String,
    #[serde(default = "default_md")]
    pub md: String,
    #[serde(default = "default_lg")]
    pub lg: String,
    #[serde(default = "default_full")]
    pub full: String,
}

fn default_sm() -> String {
    "0.125rem".to_string()
}
fn default_md() -> String {
    "0.25rem".to_string()
}
fn default_lg() -> String {
    "0.5rem".to_string()
}
fn default_full() -> String {
    "9999px".to_string()
}

impl Default for RadiiTokens {
    fn default() -> Self {
        Self {
            sm: default_sm(),
            md: default_md(),
            lg: default_lg(),
            full: default_full(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShadowTokens {
    #[serde(default = "default_shadow")]
    pub sm: String,
    #[serde(default = "default_shadow")]
    pub md: String,
    #[serde(default = "default_shadow")]
    pub lg: String,
    #[serde(default = "default_shadow")]
    pub xl: String,
}

fn default_shadow() -> String {
    "none".to_string()
}

impl Default for ShadowTokens {
    fn default() -> Self {
        Self {
            sm: "0 1px 2px 0 rgba(0, 0, 0, 0.05)".to_string(),
            md: "0 4px 6px -1px rgba(0, 0, 0, 0.1)".to_string(),
            lg: "0 10px 15px -3px rgba(0, 0, 0, 0.1)".to_string(),
            xl: "0 20px 25px -5px rgba(0, 0, 0, 0.1)".to_string(),
        }
    }
}
