//! Boundary tests between the shared file-type catalog and the GUI detector.
//!
//! The catalog in `dispel-core` owns filename detection; `EditorType::from_path`
//! must agree with it for every catalogued key, otherwise opening a file from
//! the CLI and from the GUI lands in different editors.

use dispel_core::commands::registry::{self, DetectResult};

use crate::workspace::EditorType;

#[test]
fn test_detect_agrees_with_from_path_for_every_catalog_key() {
    for ft in registry::FILE_TYPES.iter() {
        let path = ft.sample_path();
        match registry::detect(&path) {
            DetectResult::Single(detected) => {
                assert_eq!(
                    detected.key, ft.key,
                    "catalog detection drift for {}",
                    ft.key
                )
            }
            other => panic!(
                "sample path {} gave {:?}",
                path.display(),
                match other {
                    DetectResult::Single(_) => unreachable!(),
                    DetectResult::Ambiguous(candidates) => {
                        format!(
                            "ambiguous: {:?}",
                            candidates.iter().map(|c| c.key).collect::<Vec<_>>()
                        )
                    }
                    DetectResult::None => "none".to_string(),
                }
            ),
        }

        let expected = EditorType::from_catalog_editor(ft.editor)
            .unwrap_or_else(|| panic!("no GUI editor for catalog id '{}'", ft.editor));
        let actual = EditorType::from_path(&path);
        assert_eq!(
            actual,
            expected,
            "from_path({}) = {:?}, expected {:?} for key '{}'",
            path.display(),
            actual,
            expected,
            ft.key
        );
    }
}

#[test]
fn test_catalog_multimagic_alias_resolves_to_magic() {
    let ft = registry::detect(std::path::Path::new("MulMagic.db"));
    assert!(
        matches!(ft, DetectResult::Single(f) if f.key == "magic"),
        "MulMagic.db must resolve to the magic catalog entry"
    );
}
