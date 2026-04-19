use std::time::Instant;

#[derive(Debug, Clone)]
pub struct IndexationProgress {
    pub processed: u32,
    pub total: u32,
    pub current_path: Option<std::path::PathBuf>,
    pub files_per_second: Option<f32>,
    pub eta_seconds: Option<f32>,
}

// Test the progress tracking logic in isolation
#[test]
fn test_progress_tracking_logic() {
    const MAX_UPDATES_PER_SECOND: u32 = 10;
    let min_interval = 1.0 / MAX_UPDATES_PER_SECOND as f32;

    // Test rate limiting logic - when no last update time, should allow update
    let last_update_time: Option<Instant> = None;
    let now = Instant::now();

    let should_update = last_update_time.map_or(true, |last| {
        now.duration_since(last).as_secs_f32() >= min_interval
    });

    assert!(
        should_update,
        "First update should be allowed when no last update time"
    );

    // Test when time elapsed is sufficient
    let last_update_time = Some(Instant::now() - std::time::Duration::from_secs_f32(0.2));
    let now = Instant::now();
    let should_update = last_update_time.map_or(true, |last| {
        now.duration_since(last).as_secs_f32() >= min_interval
    });

    assert!(
        should_update,
        "Update should be allowed after sufficient time elapsed"
    );

    // Test percentage calculation
    let processed = 50;
    let total = 200;
    let percentage = (processed as f32 / total as f32) * 100.0;
    assert_eq!(percentage, 25.0, "Percentage calculation should be correct");

    // Test ETA calculation
    let elapsed_seconds = 5.0; // 50 files in 5 seconds = 10 files/sec
    let files_per_second = processed as f32 / elapsed_seconds;
    let remaining_files = (total - processed) as f32;
    let eta_seconds = remaining_files / files_per_second;

    assert_eq!(files_per_second, 10.0, "Files per second should be correct");
    assert_eq!(
        eta_seconds, 15.0,
        "ETA should be 15 seconds for remaining 150 files"
    );

    // Test progress creation
    let progress = IndexationProgress {
        processed: 25,
        total: 100,
        current_path: None,
        files_per_second: Some(8.3),
        eta_seconds: Some(9.0),
    };

    assert_eq!(progress.processed, 25);
    assert_eq!(progress.total, 100);
    assert_eq!(progress.files_per_second.unwrap(), 8.3);
    assert_eq!(progress.eta_seconds.unwrap(), 9.0);
}

#[test]
fn test_progress_status_formatting() {
    // Test the status message formatting logic
    let progress = IndexationProgress {
        processed: 42,
        total: 298,
        current_path: None,
        files_per_second: Some(123.0),
        eta_seconds: Some(2.1),
    };

    let percentage = (progress.processed as f32 / progress.total as f32) * 100.0;

    // Format ETA
    let eta_display = progress
        .eta_seconds
        .map(|eta| {
            if eta < 60.0 {
                format!("{:.0}s", eta)
            } else if eta < 3600.0 {
                format!("{:.1}m", eta / 60.0)
            } else {
                format!("{:.1}h", eta / 3600.0)
            }
        })
        .unwrap_or_else(|| "?".to_string());

    // Format speed
    let speed_display = progress
        .files_per_second
        .map(|fps| {
            if fps < 1000.0 {
                format!("{:.0} files/s", fps)
            } else {
                format!("{:.1}k files/s", fps / 1000.0)
            }
        })
        .unwrap_or_else(|| "? files/s".to_string());

    let status = format!(
        "Indexing... {:.0}% ({}/{}) - {} - ETA: {}",
        percentage, progress.processed, progress.total, speed_display, eta_display
    );

    assert!(status.contains("14%"), "Should contain percentage");
    assert!(status.contains("42/298"), "Should contain file count");
    assert!(status.contains("123 files/s"), "Should contain speed");
    assert!(status.contains("ETA: 2s"), "Should contain ETA");
}
