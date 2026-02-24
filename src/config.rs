/// Configuration for the Composer proto plugin.
/// Users can override these in `.prototools` under `[tools.composer]`.
#[derive(Debug, schematic::Schematic, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ComposerPluginConfig {
    /// Custom COMPOSER_HOME directory.
    pub composer_home: Option<String>,
}

impl Default for ComposerPluginConfig {
    fn default() -> Self {
        Self {
            composer_home: None,
        }
    }
}
