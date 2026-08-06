//! Byte-frequency analysis, Shannon entropy, and structure heuristics for the
//! hex editor's statistics panel.
//!
//! All functions are pure — they operate on `&[u8]` slices, not on editor state.

/// Pre-computed per-row entropy cache for colour bands in the address gutter.
#[derive(Debug, Clone)]
pub struct RowEntropyCache {
    /// `(row_start_addr, entropy)` for every row in the file.
    pub rows: Vec<(u64, f64)>,
    /// Maximum entropy across all rows (for colour normalisation).
    pub max_entropy: f64,
    /// Minimum entropy across all rows (for colour normalisation).
    pub min_entropy: f64,
}

/// Byte-frequency histogram and derived statistics for a byte range.
#[derive(Debug, Clone)]
pub struct ByteStatistics {
    /// Count per byte value `0x00..=0xFF`.
    pub histogram: [u64; 256],
    /// Total bytes analysed.
    pub total: u64,
    /// Shannon entropy: `-Σ p(i)·log₂(p(i))` over the histogram.
    pub entropy: f64,
    /// Minimum byte value.
    pub min: u8,
    /// Maximum byte value.
    pub max: u8,
    /// Arithmetic mean (average byte value).
    pub mean: f64,
    /// Median byte value.
    pub median: u8,
    /// Detected structural heuristic.
    pub structure: StructureHeuristic,
    /// Number of null bytes (`0x00`).
    pub null_count: u64,
    /// Number of printable ASCII bytes (`0x20..=0x7E`).
    pub printable_count: u64,
    /// Number of high-ASCII bytes (`0x80..=0xFF`).
    pub high_ascii_count: u64,
}

/// Structural heuristic describing the analysed byte region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StructureHeuristic {
    /// All bytes are identical to `val`.
    Uniform(u8),
    /// High-entropy region — likely compressed or encrypted.
    HighEntropy,
    /// Low-entropy region — likely sparse or padding.
    LowEntropy,
    /// Mix of text and binary — typical structured file.
    Mixed,
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Compute full byte statistics for a byte slice.
pub fn compute_statistics(bytes: &[u8]) -> ByteStatistics {
    let total = bytes.len() as u64;
    if total == 0 {
        return ByteStatistics {
            histogram: [0; 256],
            total: 0,
            entropy: 0.0,
            min: 0,
            max: 0,
            mean: 0.0,
            median: 0,
            structure: StructureHeuristic::LowEntropy,
            null_count: 0,
            printable_count: 0,
            high_ascii_count: 0,
        };
    }

    let histogram = build_histogram(bytes);
    let (min, max, mean, null_count, printable_count, high_ascii_count) =
        summarise_histogram(&histogram, total);

    let entropy = compute_entropy_from_histogram(&histogram, total);
    let median = compute_median(&histogram, total);
    let structure = detect_structure(bytes, &histogram, total, entropy);

    ByteStatistics {
        histogram,
        total,
        entropy,
        min,
        max,
        mean,
        median,
        structure,
        null_count,
        printable_count,
        high_ascii_count,
    }
}

/// Build the 256-bin histogram for a byte slice.
pub fn build_histogram(bytes: &[u8]) -> [u64; 256] {
    let mut hist = [0u64; 256];
    for &b in bytes {
        hist[b as usize] += 1;
    }
    hist
}

/// Compute Shannon entropy from a pre-built histogram.
pub fn compute_entropy_from_histogram(histogram: &[u64; 256], total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let mut entropy = 0.0f64;
    for &count in histogram {
        if count > 0 {
            let p = count as f64 / total as f64;
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Convenience: compute Shannon entropy for a byte slice directly.
pub fn compute_entropy(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }
    let hist = build_histogram(bytes);
    compute_entropy_from_histogram(&hist, bytes.len() as u64)
}

/// Compute per-row entropy for every complete row in a file.
///
/// The last partial row is included if non-empty. Returns a [`RowEntropyCache`]
/// suitable for colour-band rendering in the address gutter.
pub fn compute_row_entropies(bytes: &[u8], bytes_per_row: u8) -> RowEntropyCache {
    let bpr = bytes_per_row.max(1) as usize;
    if bytes.is_empty() {
        return RowEntropyCache {
            rows: Vec::new(),
            max_entropy: 0.0,
            min_entropy: 0.0,
        };
    }
    let mut rows = Vec::with_capacity(bytes.len().div_ceil(bpr));
    let mut max_e = 0.0f64;
    let mut min_e = f64::MAX;

    for (chunk_idx, chunk) in bytes.chunks(bpr).enumerate() {
        let row_addr = (chunk_idx * bpr) as u64;
        let hist = build_histogram(chunk);
        let e = compute_entropy_from_histogram(&hist, chunk.len() as u64);
        max_e = max_e.max(e);
        min_e = min_e.min(e);
        rows.push((row_addr, e));
    }

    if min_e == f64::MAX {
        min_e = 0.0;
    }

    RowEntropyCache {
        rows,
        max_entropy: max_e,
        min_entropy: min_e,
    }
}

/// Classify a byte region's structure.
///
/// Strategy:
/// 1. If all bytes identical → `Uniform(val)`.
/// 2. If entropy > 0.9 * 8.0 = 7.2 → `HighEntropy` (near-max, likely compressed).
/// 3. If entropy < 0.3 * 8.0 = 2.4 → `LowEntropy` (sparse / padding).
/// 4. Check for repeating fixed-size pattern (period 1..=64).
/// 5. Otherwise → `Mixed`.
fn detect_structure(
    bytes: &[u8],
    histogram: &[u64; 256],
    total: u64,
    entropy: f64,
) -> StructureHeuristic {
    if total == 0 {
        return StructureHeuristic::LowEntropy;
    }

    // 1. Uniform check.
    let non_zero = histogram.iter().filter(|&&c| c > 0).count();
    if non_zero == 1 {
        for (val, &count) in histogram.iter().enumerate() {
            if count == total {
                return StructureHeuristic::Uniform(val as u8);
            }
        }
    }

    // 2. High / low entropy thresholds (based on log₂(256) = 8.0 scale).
    if entropy > 7.2 {
        return StructureHeuristic::HighEntropy;
    }
    if entropy < 2.4 {
        return StructureHeuristic::LowEntropy;
    }

    // 3. Repeating fixed-size pattern detection.
    // Only try when total >= 2*period and not too large to avoid O(n²).
    if (8..=10_000_000).contains(&total) {
        // Test periods 1..=64 (powers of 2 + small primes) for speed.
        let candidates = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 18, 20, 24, 28, 32, 36, 40, 48, 56, 64,
        ];
        let max_period = bytes.len() / 2;
        for &period in &candidates {
            if period as usize > max_period {
                continue;
            }
            if let Some(confidence) = check_repeating_period(bytes, period as usize)
                && confidence > 0.85
            {
                return StructureHeuristic::Mixed; // Not exporting period yet
            }
        }
    }

    StructureHeuristic::Mixed
}

/// Check whether `bytes` repeats with the given `period`.
/// Returns `Some(confidence)` where confidence is the fraction of boundary
/// checks that matched, or `None` if the pattern is too short to evaluate.
fn check_repeating_period(bytes: &[u8], period: usize) -> Option<f64> {
    if bytes.len() < period * 2 {
        return None;
    }
    let mut matches = 0u64;
    let mut total_checks = 0u64;

    // Sample boundaries: for each block-aligned address, check if the byte at
    // `addr` matches the byte at `addr + period` (i.e., the same position in
    // the next block). We sample a subset for large files.
    let step = if bytes.len() > 100_000 { period } else { 1 };
    let mut addr = 0usize;
    while addr + period < bytes.len() {
        if bytes[addr] == bytes[addr + period] {
            matches += 1;
        }
        total_checks += 1;
        addr += step;
    }

    if total_checks == 0 {
        return None;
    }
    Some(matches as f64 / total_checks as f64)
}

// ── Internal helpers ───────────────────────────────────────────────────────

fn summarise_histogram(hist: &[u64; 256], total: u64) -> (u8, u8, f64, u64, u64, u64) {
    let mut min = 0xFFu8;
    let mut max = 0x00u8;
    let mut sum = 0u64;
    let mut nulls = 0u64;
    let mut printables = 0u64;
    let mut high_ascii = 0u64;

    for (byte, &count) in hist.iter().enumerate() {
        if count > 0 {
            if (byte as u8) < min {
                min = byte as u8;
            }
            if (byte as u8) > max {
                max = byte as u8;
            }
            sum += byte as u64 * count;
            if byte == 0x00 {
                nulls = count;
            }
            if (0x20..=0x7E).contains(&(byte as u8)) {
                printables += count;
            }
            if byte >= 0x80 {
                high_ascii += count;
            }
        }
    }

    let mean = if total > 0 {
        sum as f64 / total as f64
    } else {
        0.0
    };

    (min, max, mean, nulls, printables, high_ascii)
}

fn compute_median(hist: &[u64; 256], total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    let mid = total.div_ceil(2);
    let mut cumulative = 0u64;
    for (byte, &count) in hist.iter().enumerate() {
        cumulative += count;
        if cumulative >= mid {
            return byte as u8;
        }
    }
    0xFF
}

// ── Color helpers for entropy bands ────────────────────────────────────────

/// Map an entropy value (0.0..=8.0) to an RGB colour for the address gutter
/// band. Low entropy → blue (sparse), high entropy → red (compressed),
/// mid-range → green (text / structured).
pub fn entropy_to_color(entropy: f64) -> (f32, f32, f32) {
    // Normalise to 0..1 on an 8.0 scale.
    let t = (entropy / 8.0).clamp(0.0, 1.0) as f32;
    // Blue (0, 0.3, 0.8) → Green (0.3, 0.8, 0.3) → Red (0.8, 0.2, 0.2)
    if t < 0.3 {
        // Blue-to-cyan gradient (low entropy → structured).
        let u = t / 0.3;
        (0.0 + u * 0.3, 0.3 + u * 0.3, 0.8 - u * 0.2)
    } else if t < 0.7 {
        // Cyan-to-green gradient (structured → typical).
        let u = (t - 0.3) / 0.4;
        (0.3 - u * 0.1, 0.6 + u * 0.2, 0.6 - u * 0.3)
    } else {
        // Green-to-red gradient (typical → high entropy).
        let u = (t - 0.7) / 0.3;
        (0.2 + u * 0.6, 0.8 - u * 0.6, 0.3 - u * 0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slice() {
        let stats = compute_statistics(&[]);
        assert_eq!(stats.total, 0);
        assert_eq!(stats.entropy, 0.0);
    }

    #[test]
    fn uniform_bytes() {
        let bytes = vec![0x42u8; 100];
        let stats = compute_statistics(&bytes);
        assert_eq!(stats.structure, StructureHeuristic::Uniform(0x42));
        assert_eq!(stats.min, 0x42);
        assert_eq!(stats.max, 0x42);
        assert!((stats.mean - 66.0).abs() < 0.01);
        assert_eq!(stats.median, 0x42);
        assert_eq!(stats.null_count, 0);
    }

    #[test]
    fn null_bytes() {
        let bytes = vec![0x00u8; 50];
        let stats = compute_statistics(&bytes);
        assert_eq!(stats.structure, StructureHeuristic::Uniform(0x00));
        assert_eq!(stats.null_count, 50);
        assert!(stats.entropy < 0.01);
        assert_eq!(stats.median, 0);
    }

    #[test]
    fn uniform_structure_all_same() {
        let bytes = [0xAAu8; 10];
        let stats = compute_statistics(&bytes);
        assert_eq!(stats.structure, StructureHeuristic::Uniform(0xAA));
    }

    #[test]
    fn entropy_range() {
        // Low entropy: all identical bytes.
        let low = compute_entropy(&[0x00; 100]);
        assert!(low < 0.1);

        // High entropy: all 256 values equally distributed.
        let all: Vec<u8> = (0..=255).cycle().take(2560).collect();
        let high = compute_entropy(&all);
        assert!((high - 8.0).abs() < 0.1, "high entropy = {high}");

        // Mid entropy: English-like ASCII text.
        let text = b"The quick brown fox jumps over the lazy dog. ";
        let mid = compute_entropy(text);
        assert!(mid > 3.0 && mid < 6.0, "mid entropy = {mid}");
    }

    #[test]
    fn histogram_counts() {
        let bytes = vec![0x00, 0x01, 0x01, 0x02, 0x02, 0x02];
        let stats = compute_statistics(&bytes);
        assert_eq!(stats.histogram[0], 1);
        assert_eq!(stats.histogram[1], 2);
        assert_eq!(stats.histogram[2], 3);
        assert_eq!(stats.total, 6);
    }

    #[test]
    fn min_max_mean_median() {
        let bytes: Vec<u8> = vec![10, 20, 30, 40, 50];
        let stats = compute_statistics(&bytes);
        assert_eq!(stats.min, 10);
        assert_eq!(stats.max, 50);
        assert!((stats.mean - 30.0).abs() < 0.01);
        assert_eq!(stats.median, 30);
    }

    #[test]
    fn row_entropies_empty() {
        let cache = compute_row_entropies(&[], 16);
        assert!(cache.rows.is_empty());
    }

    #[test]
    fn row_entropies_basic() {
        let bytes: Vec<u8> = (0..32).collect();
        let cache = compute_row_entropies(&bytes, 16);
        assert_eq!(cache.rows.len(), 2);
        // Both rows contain all 16 distinct bytes → entropy == log₂(16) = 4.0.
        for (_, e) in &cache.rows {
            assert!((e - 4.0).abs() < 0.01);
        }
    }

    #[test]
    fn entropy_to_color_clamps() {
        let (r, g, b) = entropy_to_color(0.0);
        assert!((0.0..=1.0).contains(&r));
        assert!((0.0..=1.0).contains(&g));
        assert!((0.0..=1.0).contains(&b));
        let (r2, g2, b2) = entropy_to_color(8.0);
        assert!((0.0..=1.0).contains(&r2));
        assert!((0.0..=1.0).contains(&g2));
        assert!((0.0..=1.0).contains(&b2));
        // 8.0 should be red-dominant; 0.0 should be blue-dominant.
        assert!(r2 > b2, "high entropy → red-dominant");
        let (r0, _g0, b0) = entropy_to_color(0.0);
        assert!(b0 > r0, "low entropy → blue-dominant");
    }

    #[test]
    fn structure_high_entropy() {
        // Uniform random-like data → high entropy.
        let bytes: Vec<u8> = (0..=255).cycle().take(2048).collect();
        let stats = compute_statistics(&bytes);
        assert!(stats.entropy > 7.5);
    }

    #[test]
    fn structure_low_entropy() {
        let bytes = vec![0x00u8; 64];
        let stats = compute_statistics(&bytes);
        assert_eq!(stats.structure, StructureHeuristic::Uniform(0x00));
    }
}
