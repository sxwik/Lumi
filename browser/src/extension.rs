use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub custom_css_theme: Option<String>,
}

pub struct ExtensionRegistry {
    pub extensions: Vec<ExtensionManifest>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            extensions: vec![ExtensionManifest {
                name: "CyberDark Theme (.lpx)".to_string(),
                version: "1.0.0".to_string(),
                description: "Applies high-contrast neon accents to LumiML containers.".to_string(),
                custom_css_theme: Some("cyberdark".to_string()),
            }],
        }
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
