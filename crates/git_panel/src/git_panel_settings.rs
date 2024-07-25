use anyhow;
use gpui::Pixels;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsSources};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GitPanelDockPosition {
    Left,
    Right,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct GitPanelSettings {
    pub button: bool,
    pub default_width: Pixels,
    pub dock: GitPanelDockPosition
}

#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, Debug)]
pub struct GitPanelSettingsContent {
    /// Whether to show the git panel button in the status bar.
    ///
    /// Default: true
    pub button: Option<bool>,
    /// Customise default width (in pixels) taken by git panel
    ///
    /// Default: 240
    pub default_width: Option<f32>,
    /// The position of git panel
    ///
    /// Default: left
    pub dock: Option<GitPanelDockPosition>,
}

impl Settings for GitePanelSettings {
    const KEY: Option<&'static str> = Some("git_panel");

    type FileContent = GitPanelSettingsContent;

    fn load(
        sources: SettingsSources<Self::FileContent>,
        _: &mut gpui::AppContext,
    ) -> anyhow::Result<Self> {
        sources.json_merge()
    }
}