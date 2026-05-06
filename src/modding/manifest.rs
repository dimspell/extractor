use serde::{Deserialize, Serialize};

pub const MANIFEST_VERSION: u32 = 1;

/// User-facing metadata for a mod, serialised as `manifest.json` inside a
/// mod package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModManifest {
    /// Schema version of the manifest itself; bumped if the layout changes.
    #[serde(default = "default_manifest_version")]
    pub manifest_version: u32,

    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,

    /// Mod ids this mod expects to be present (informational; not enforced
    /// at apply time in v1).
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Optional ordering hint shown to the user in the load-order UI.
    /// Lower values surface earlier; ties broken by user-visible order.
    #[serde(default)]
    pub load_order_hint: Option<i32>,
}

fn default_manifest_version() -> u32 {
    MANIFEST_VERSION
}

impl ModManifest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            manifest_version: MANIFEST_VERSION,
            name: name.into(),
            version: String::new(),
            author: String::new(),
            description: String::new(),
            dependencies: Vec::new(),
            load_order_hint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let m = ModManifest {
            manifest_version: MANIFEST_VERSION,
            name: "spelling-fixes".into(),
            version: "0.1.0".into(),
            author: "someone".into(),
            description: "Fixes hundreds of typos".into(),
            dependencies: vec!["base-balance".into()],
            load_order_hint: Some(10),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: ModManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn deserialises_with_minimal_fields() {
        let json = r#"{"name":"tiny"}"#;
        let m: ModManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.name, "tiny");
        assert_eq!(m.manifest_version, MANIFEST_VERSION);
        assert!(m.dependencies.is_empty());
        assert_eq!(m.load_order_hint, None);
    }
}
