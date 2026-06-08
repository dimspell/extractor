#[cfg(test)]
mod auto_save_tests {
    use crate::auto_save::DraftManager;
    use std::path::{Path, PathBuf};

    /// Create a DraftManager whose persist path points into a temp directory
    /// so `save_draft()` / `discard_draft()` never touch the real config.
    fn dm_with_temp_path() -> (DraftManager, PathBuf) {
        let tmp = std::env::temp_dir().join("dispel-test-auto-save-dm");
        let _ = std::fs::create_dir_all(&tmp);
        let persist = tmp.join("drafts.json");
        (DraftManager::with_persist_path(persist), tmp)
    }

    // ── In-memory operations (no disk I/O) ──────────────────────────────────

    #[test]
    fn test_draft_manager_new_defaults() {
        let dm = DraftManager::new();
        assert!(dm.is_auto_save_enabled(), "auto_save_enabled should be true");
        assert_eq!(dm.draft_count(), 0, "draft_count should be 0");
        assert!(dm.pending_drafts().is_empty(), "pending_drafts should be empty");
    }

    #[test]
    fn test_draft_manager_toggle_auto_save() {
        let mut dm = DraftManager::new();
        assert!(dm.is_auto_save_enabled());

        dm.toggle_auto_save();
        assert!(!dm.is_auto_save_enabled(), "off after first toggle");

        dm.toggle_auto_save();
        assert!(dm.is_auto_save_enabled(), "on after second toggle");

        dm.toggle_auto_save();
        assert!(!dm.is_auto_save_enabled(), "off after third toggle");
    }

    // ── File-backed operations (use temp persist path) ──────────────────────

    #[test]
    fn test_draft_manager_save_and_has_draft() {
        let (mut dm, _tmp) = dm_with_temp_path();
        let path = Path::new("/draft/test.ini");

        assert!(!dm.has_draft(path), "no draft before save");
        dm.save_draft(path, b"hello world");
        assert!(dm.has_draft(path), "draft exists after save");
    }

    #[test]
    fn test_draft_manager_get_draft_returns_content() {
        let (mut dm, _tmp) = dm_with_temp_path();
        let path = Path::new("/draft/data.bin");
        let content = b"\x00\x01\x02\xFF\xFE";

        dm.save_draft(path, content);
        let draft = dm.get_draft(path).expect("draft should exist");
        assert_eq!(draft.content.as_slice(), content, "content bytes match");
        assert_eq!(draft.file_path, path, "file path matches");
    }

    #[test]
    fn test_draft_manager_clear_draft() {
        let (mut dm, _tmp) = dm_with_temp_path();
        let path = Path::new("/draft/clear_me.ini");

        dm.save_draft(path, b"content");
        assert_eq!(dm.draft_count(), 1);

        dm.clear_draft(path);
        assert!(!dm.has_draft(path), "draft cleared");
        assert_eq!(dm.draft_count(), 0, "count is 0 after clear");
    }

    #[test]
    fn test_draft_manager_discard_draft() {
        let (mut dm, _tmp) = dm_with_temp_path();
        let path = Path::new("/draft/discard_me.ini");

        dm.save_draft(path, b"content");
        assert!(dm.has_draft(path));

        dm.discard_draft(path);
        assert!(!dm.has_draft(path), "draft discarded");
    }

    #[test]
    fn test_draft_manager_draft_count_and_pending() {
        let (mut dm, _tmp) = dm_with_temp_path();
        let path_a = Path::new("/draft/a.ini");
        let path_b = Path::new("/draft/b.ini");

        dm.save_draft(path_a, b"alpha");
        assert_eq!(dm.draft_count(), 1);

        dm.save_draft(path_b, b"beta");
        assert_eq!(dm.draft_count(), 2);

        let pending = dm.pending_drafts();
        assert_eq!(pending.len(), 2, "two pending drafts");
        let paths: Vec<_> = pending.iter().map(|d| d.file_path.as_path()).collect();
        assert!(paths.contains(&path_a), "path_a in pending");
        assert!(paths.contains(&path_b), "path_b in pending");
    }

    #[test]
    fn test_draft_manager_serde_roundtrip() {
        let (mut dm, _tmp) = dm_with_temp_path();
        dm.save_draft(Path::new("/serde/a.txt"), b"serde content A");
        dm.save_draft(Path::new("/serde/b.txt"), b"serde content B");

        let json = serde_json::to_string(&dm).expect("serialize to JSON");
        // persist_path is #[serde(skip)], so deserialized DraftManager gets
        // the default persist_path. That's fine — we only test data fidelity.
        let restored: DraftManager =
            serde_json::from_str(&json).expect("deserialize from JSON");

        assert_eq!(restored.draft_count(), 2, "restored has 2 drafts");
        assert!(restored.is_auto_save_enabled(), "auto_save_enabled preserved");
        assert!(restored.has_draft(Path::new("/serde/a.txt")), "draft A restored");
        assert!(restored.has_draft(Path::new("/serde/b.txt")), "draft B restored");

        let content_a = restored.get_draft(Path::new("/serde/a.txt")).unwrap();
        assert_eq!(
            content_a.content.as_slice(),
            b"serde content A",
            "content A preserved"
        );
    }

    #[test]
    fn test_draft_manager_apply_draft() {
        let (mut dm, tmp) = dm_with_temp_path();
        let file_path = tmp.join("apply_test.bin");

        // Write original content to the temp file
        std::fs::write(&file_path, b"original content").unwrap();

        // Save draft with modified content
        dm.save_draft(&file_path, b"modified draft content");

        // Apply the draft — should overwrite the file
        dm.apply_draft(&file_path).expect("apply_draft should succeed");

        let after = std::fs::read(&file_path).expect("read file after apply");
        assert_eq!(
            after.as_slice(),
            b"modified draft content",
            "file content updated by apply_draft"
        );

        // Cleanup
        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_draft_manager_apply_draft_no_draft_returns_error() {
        let dm = DraftManager::new();
        let path = Path::new("/nonexistent/draft.bin");
        let result = dm.apply_draft(path);
        assert!(result.is_err(), "should return error when no draft exists");
    }

    #[test]
    fn test_draft_manager_check_conflicts_no_conflict() {
        let (mut dm, tmp) = dm_with_temp_path();
        let file_path = tmp.join("stable.bin");

        std::fs::write(&file_path, b"stable content").unwrap();
        dm.save_draft(&file_path, b"draft content");

        // Immediately check — no external modification, so no conflicts
        let conflicts = dm.check_conflicts();
        assert!(
            conflicts.is_empty(),
            "no conflicts expected right after save_draft"
        );

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_draft_manager_check_conflicts_with_touch() {
        let (mut dm, tmp) = dm_with_temp_path();
        let file_path = tmp.join("touch_test.bin");

        std::fs::write(&file_path, b"initial content").unwrap();
        dm.save_draft(&file_path, b"draft content");

        // Modify the file externally (change mtime)
        std::fs::write(&file_path, b"modified externally").unwrap();

        // Now there should be a conflict
        let conflicts = dm.check_conflicts();
        assert_eq!(conflicts.len(), 1, "one conflict expected after external touch");
        assert_eq!(
            conflicts[0].file_path,
            file_path,
            "conflict file path matches"
        );

        let _ = std::fs::remove_file(&file_path);
    }
}
