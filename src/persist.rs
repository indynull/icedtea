//! Save and restore window geometry, splits, docks, theme, and density.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::density::DensityName;
use crate::layout::{Axis, SplitState};

/// Window position and size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowGeom {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for WindowGeom {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 960.0,
            height: 640.0,
        }
    }
}

/// Which edge a dock pane sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockEdge {
    Left,
    Right,
    Top,
    Bottom,
}

/// Saved dock pane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockLayout {
    pub edge: DockEdge,
    pub size: f32,
    pub visible: bool,
}

/// Full UI persistence blob.
///
/// ```
/// let mut ui = icedtea::persist::UiState::default();
/// ui.theme = "nord".into();
/// let json = ui.to_json().unwrap();
/// let back = icedtea::persist::UiState::from_json(&json).unwrap();
/// assert_eq!(back.theme, "nord");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiState {
    pub window: WindowGeom,
    pub splits: BTreeMap<String, f32>,
    pub docks: BTreeMap<String, DockLayout>,
    pub theme: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(default)]
    pub follow_os: bool,
    pub density: DensityName,
    #[serde(default = "default_scale")]
    pub font_scale: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

fn default_scale() -> f32 {
    1.0
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            window: WindowGeom::default(),
            splits: BTreeMap::new(),
            docks: BTreeMap::new(),
            theme: "dark".into(),
            family: None,
            follow_os: true,
            density: DensityName::Default,
            font_scale: 1.0,
            accent: None,
            workspace: None,
        }
    }
}

impl UiState {
    pub fn set_split(&mut self, id: impl Into<String>, state: SplitState) {
        self.splits.insert(id.into(), state.persist());
    }

    pub fn split(&self, id: &str, axis: Axis) -> SplitState {
        SplitState::restore(axis, self.splits.get(id).copied().unwrap_or(0.3))
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn config_path(app_id: &str) -> PathBuf {
        config_dir(
            env_path("XDG_CONFIG_HOME"),
            env_path("HOME"),
            env_path("APPDATA"),
        )
        .join(app_id)
        .join("ui.json")
    }

    pub fn load_file(path: &Path) -> Result<Self, PersistError> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&text)?)
    }

    pub fn save_file(&self, path: &Path) -> Result<(), PersistError> {
        if let Some(dir) = parent_dir(path) {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, self.to_json()?)?;
        Ok(())
    }
}

/// Directory to create before writing `path`, if any.
pub fn parent_dir(path: &Path) -> Option<&Path> {
    path.parent().filter(|d| !d.as_os_str().is_empty())
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from)
}

fn config_dir(
    xdg_config_home: Option<PathBuf>,
    home: Option<PathBuf>,
    appdata: Option<PathBuf>,
) -> PathBuf {
    xdg_config_home
        .or_else(|| home.map(|h| h.join(".config")))
        .or(appdata)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Persistence error.
#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::SplitState;

    #[test]
    fn json_roundtrip_and_split_restore() {
        let mut ui = UiState::default();
        assert!(ui.follow_os);
        ui.set_split("main", SplitState::new(Axis::Horizontal, 0.4));
        ui.docks.insert(
            "nav".into(),
            DockLayout {
                edge: DockEdge::Left,
                size: 220.0,
                visible: true,
            },
        );
        ui.theme = "light".into();
        ui.family = Some("github".into());
        ui.follow_os = true;
        ui.density = DensityName::Compact;
        ui.font_scale = 1.125;
        ui.accent = Some("#88c0d0".into());
        ui.workspace = Some(r#"{"Leaf":{"id":"e","title":"E","min":80.0,"max":10000.0}}"#.into());
        ui.window.x = 12.0;
        let json = ui.to_json().unwrap();
        let back = UiState::from_json(&json).unwrap();
        assert_eq!(back, ui);
        assert!((back.split("main", Axis::Horizontal).ratio - 0.4).abs() < 0.02);
        assert_eq!(back.split("missing", Axis::Vertical).axis, Axis::Vertical);
        assert!(UiState::from_json("not-json").is_err());
        let p = UiState::config_path("icedtea-test");
        assert!(p.ends_with("ui.json"));
        let dir = std::env::temp_dir().join("icedtea-persist-test");
        let file = dir.join("ui.json");
        ui.save_file(&file).unwrap();
        let loaded = UiState::load_file(&file).unwrap();
        assert_eq!(loaded.theme, "light");
        assert_eq!(loaded.family.as_deref(), Some("github"));
        assert!(loaded.follow_os);
        let named_only = UiState::from_json(r#"{"window":{"x":0,"y":0,"width":1,"height":1},"splits":{},"docks":{},"theme":"nord","density":"Default"}"#).unwrap();
        assert_eq!(named_only.theme, "nord");
        assert!(named_only.family.is_none());
        assert!(!named_only.follow_os);
        assert!((named_only.font_scale - 1.0).abs() < f32::EPSILON);
        assert!(named_only.accent.is_none());
        assert!(named_only.workspace.is_none());
        assert!((back.font_scale - 1.125).abs() < f32::EPSILON);
        assert_eq!(back.accent.as_deref(), Some("#88c0d0"));
        assert!(UiState::load_file(Path::new("/no/such/icedtea.json")).is_err());
        let blocker = std::env::temp_dir().join("icedtea-persist-not-a-dir");
        std::fs::write(&blocker, b"x").unwrap();
        assert!(ui.save_file(&blocker.join("ui.json")).is_err());
        let _ = std::fs::remove_file(&blocker);
        let rel = std::env::temp_dir().join("icedtea-rel-only.json");
        ui.save_file(&rel).unwrap();
        let _ = std::fs::remove_file(&rel);
        let cwd_file = Path::new("icedtea-persist-cwd.json");
        ui.save_file(cwd_file).unwrap();
        let loaded_cwd = UiState::load_file(cwd_file).unwrap();
        assert_eq!(loaded_cwd.theme, "light");
        let _ = std::fs::remove_file(cwd_file);
        assert!(parent_dir(Path::new("ui.json")).is_none());
        assert!(parent_dir(&std::env::temp_dir().join("ui.json")).is_some());
        assert_eq!(
            config_dir(Some(PathBuf::from("xdg")), None, None),
            PathBuf::from("xdg")
        );
        assert_eq!(
            config_dir(None, Some(PathBuf::from("home")), None),
            PathBuf::from("home").join(".config")
        );
        assert_eq!(
            config_dir(None, None, Some(PathBuf::from("appdata"))),
            PathBuf::from("appdata")
        );
        assert_eq!(config_dir(None, None, None), PathBuf::from("."));
        let io_err = PersistError::from(std::io::Error::other("disk"));
        assert!(io_err.to_string().contains("io:"));
        let json_err = PersistError::from(serde_json::from_str::<UiState>("[").unwrap_err());
        assert!(json_err.to_string().contains("json:"));
        for edge in [
            DockEdge::Left,
            DockEdge::Right,
            DockEdge::Top,
            DockEdge::Bottom,
        ] {
            let _ = edge;
        }
    }
}
