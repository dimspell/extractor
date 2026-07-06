use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::path::Path;

// SNF (Sound File) Format
//
// The SNF format is a simple container for PCM audio data used in the Dispel game.
// It contains a minimal header followed by raw PCM audio samples.
//
// File Structure:
// [SNF Header] (20 bytes)
// [Unknown field] (2 bytes) - Typically contains value 8
// [PCM Audio Data] (variable size, specified in header)
//
// SNF Header (20 bytes):
// - data_size (i32): Size of the audio data in bytes
// - pcmaudio_format (i16): Audio format (typically 1 for PCM)
// - number_of_channels (i16): Number of audio channels (1=mono, 2=stereo)
// - sample_rate (i32): Sampling rate in Hz (e.g., 44100)
// - byte_rate (i32): Byte rate (sample_rate * number_of_channels * bits_per_sample/8)
// - block_align (i16): Block alignment (number_of_channels * bits_per_sample/8)
// - bits_per_sample (i16): Bits per sample (typically 16)
//
// After the header, there are 2 bytes that typically contain the value 8.
// The remainder of the file contains raw PCM audio data.

#[derive(Debug, Clone)]
pub struct SnfFile {
    pub pcmaudio_format: i16,
    pub number_of_channels: i16,
    pub sample_rate: i32,
    pub byte_rate: i32,
    pub block_align: i16,
    pub bits_per_sample: i16,
    pub data_size: i32,
    pub pcm_data: Vec<u8>,
}

impl SnfFile {
    pub fn duration_secs(&self) -> f32 {
        if self.byte_rate > 0 {
            self.data_size as f32 / self.byte_rate as f32
        } else {
            0.0
        }
    }

    /// Parse WAV bytes into an SnfFile.
    /// Validates RIFF/WAVE header, fmt chunk, data chunk.
    pub fn from_wav_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        if bytes.len() < 12 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "WAV file too short: missing RIFF header",
            ));
        }
        if &bytes[0..4] != b"RIFF" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not a RIFF file",
            ));
        }
        if &bytes[8..12] != b"WAVE" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not a WAVE file",
            ));
        }

        let mut pos = 12usize; // skip RIFF header
        let mut fmt_tag: Option<i16> = None;
        let mut num_channels: Option<i16> = None;
        let mut sample_rate: Option<i32> = None;
        let mut byte_rate: Option<i32> = None;
        let mut block_align: Option<i16> = None;
        let mut bits_per_sample: Option<i16> = None;
        let mut pcm_data: Option<Vec<u8>> = None;

        while pos + 8 <= bytes.len() {
            let chunk_id = &bytes[pos..pos + 4];
            let chunk_size = u32::from_le_bytes([
                bytes[pos + 4],
                bytes[pos + 5],
                bytes[pos + 6],
                bytes[pos + 7],
            ]) as usize;

            let chunk_end = pos + 8 + chunk_size;
            if chunk_end > bytes.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "WAV chunk exceeds file bounds",
                ));
            }

            match chunk_id {
                b"fmt " => {
                    if chunk_size < 16 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "fmt chunk too small",
                        ));
                    }
                    let fmt_start = pos + 8;
                    let tag = i16::from_le_bytes([bytes[fmt_start], bytes[fmt_start + 1]]);
                    if tag != 1 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "only PCM (format tag 1) WAV files are supported",
                        ));
                    }
                    fmt_tag = Some(tag);
                    num_channels = Some(i16::from_le_bytes([
                        bytes[fmt_start + 2],
                        bytes[fmt_start + 3],
                    ]));
                    sample_rate = Some(i32::from_le_bytes([
                        bytes[fmt_start + 4],
                        bytes[fmt_start + 5],
                        bytes[fmt_start + 6],
                        bytes[fmt_start + 7],
                    ]));
                    byte_rate = Some(i32::from_le_bytes([
                        bytes[fmt_start + 8],
                        bytes[fmt_start + 9],
                        bytes[fmt_start + 10],
                        bytes[fmt_start + 11],
                    ]));
                    block_align = Some(i16::from_le_bytes([
                        bytes[fmt_start + 12],
                        bytes[fmt_start + 13],
                    ]));
                    bits_per_sample = Some(i16::from_le_bytes([
                        bytes[fmt_start + 14],
                        bytes[fmt_start + 15],
                    ]));
                }
                b"data" => {
                    let data_start = pos + 8;
                    pcm_data = Some(bytes[data_start..data_start + chunk_size].to_vec());
                }
                _ => {
                    // Unknown chunk — skip
                }
            }

            // Advance to next chunk (chunks are padded to even byte boundary)
            pos = chunk_end;
            if pos % 2 != 0 {
                pos += 1;
            }
        }

        let pcmaudio_format = fmt_tag.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "missing fmt chunk")
        })?;
        let number_of_channels = num_channels.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing channels in fmt chunk",
            )
        })?;
        let sample_rate = sample_rate.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing sample rate in fmt chunk",
            )
        })?;
        let byte_rate = byte_rate.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing byte rate in fmt chunk",
            )
        })?;
        let block_align = block_align.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing block align in fmt chunk",
            )
        })?;
        let bits_per_sample = bits_per_sample.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing bits per sample in fmt chunk",
            )
        })?;
        let raw_data = pcm_data.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "missing data chunk")
        })?;

        Ok(SnfFile {
            pcmaudio_format,
            number_of_channels,
            sample_rate,
            byte_rate,
            block_align,
            bits_per_sample,
            data_size: raw_data.len() as i32,
            pcm_data: raw_data,
        })
    }

    /// Serialize SnfFile to SNF binary format (22-byte header + PCM data).
    pub fn to_snf_bytes(&self) -> Vec<u8> {
        let header_size = 22usize;
        let mut out = Vec::with_capacity(header_size + self.pcm_data.len());

        out.extend_from_slice(&self.data_size.to_le_bytes());
        out.extend_from_slice(&self.pcmaudio_format.to_le_bytes());
        out.extend_from_slice(&self.number_of_channels.to_le_bytes());
        out.extend_from_slice(&self.sample_rate.to_le_bytes());
        out.extend_from_slice(&self.byte_rate.to_le_bytes());
        out.extend_from_slice(&self.block_align.to_le_bytes());
        out.extend_from_slice(&self.bits_per_sample.to_le_bytes());
        // Unknown field (value 8)
        out.extend_from_slice(&8i16.to_le_bytes());
        out.extend_from_slice(&self.pcm_data);
        out
    }

    pub fn to_wav_bytes(&self) -> Vec<u8> {
        // WAV layout: RIFF header (12) + fmt chunk (24) + data chunk header (8) + PCM data
        let wav_header_size = 44usize;
        let mut out = Vec::with_capacity(wav_header_size + self.pcm_data.len());

        // RIFF chunk: size = total file size - 8 ("RIFF" + size field)
        let riff_size = (36u32).saturating_add(self.data_size.max(0) as u32);
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&riff_size.to_le_bytes());
        out.extend_from_slice(b"WAVE");
        // fmt sub-chunk
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&self.pcmaudio_format.to_le_bytes());
        out.extend_from_slice(&self.number_of_channels.to_le_bytes());
        out.extend_from_slice(&self.sample_rate.to_le_bytes());
        out.extend_from_slice(&self.byte_rate.to_le_bytes());
        out.extend_from_slice(&self.block_align.to_le_bytes());
        out.extend_from_slice(&self.bits_per_sample.to_le_bytes());
        // data sub-chunk
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(self.data_size.max(0) as u32).to_le_bytes());
        out.extend_from_slice(&self.pcm_data);
        out
    }

    /// Returns `num_points` (min, max) amplitude pairs in `[-1.0, 1.0]` for waveform display.
    /// Supports 8-bit unsigned and 16-bit signed PCM.
    pub fn waveform_points(&self, num_points: usize) -> Vec<(f32, f32)> {
        if num_points == 0 || self.pcm_data.is_empty() {
            return Vec::new();
        }
        match self.bits_per_sample {
            16 => self.waveform_16bit(num_points),
            8 => self.waveform_8bit(num_points),
            _ => Vec::new(),
        }
    }

    fn waveform_16bit(&self, num_points: usize) -> Vec<(f32, f32)> {
        if self.pcm_data.len() < 2 {
            return Vec::new();
        }
        let num_samples = self.pcm_data.len() / 2;
        let chunk_size = (num_samples / num_points).max(1);
        let mut result = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let start = i * chunk_size;
            if start >= num_samples {
                break;
            }
            let end = ((i + 1) * chunk_size).min(num_samples);
            let (mut lo, mut hi) = (0i16, 0i16);
            for j in start..end {
                let idx = j * 2;
                let sample = i16::from_le_bytes([self.pcm_data[idx], self.pcm_data[idx + 1]]);
                lo = lo.min(sample);
                hi = hi.max(sample);
            }
            result.push((lo as f32 / 32768.0, hi as f32 / 32768.0));
        }
        result
    }

    // 8-bit WAV PCM uses unsigned samples (0–255, 128 = silence).
    fn waveform_8bit(&self, num_points: usize) -> Vec<(f32, f32)> {
        let num_samples = self.pcm_data.len();
        let chunk_size = (num_samples / num_points).max(1);
        let mut result = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let start = i * chunk_size;
            if start >= num_samples {
                break;
            }
            let end = ((i + 1) * chunk_size).min(num_samples);
            let (mut lo, mut hi) = (0.0f32, 0.0f32);
            for &byte in &self.pcm_data[start..end] {
                let sample = (byte as f32 - 128.0) / 128.0;
                lo = lo.min(sample);
                hi = hi.max(sample);
            }
            result.push((lo, hi));
        }
        result
    }
}

/// Reads an SNF file into memory.
pub fn read(path: &Path) -> Result<SnfFile> {
    let mut file = File::open(path)?;

    let data_size = file.read_i32::<LittleEndian>()?;
    let pcmaudio_format = file.read_i16::<LittleEndian>()?;
    let number_of_channels = file.read_i16::<LittleEndian>()?;
    let sample_rate = file.read_i32::<LittleEndian>()?;
    let byte_rate = file.read_i32::<LittleEndian>()?;
    let block_align = file.read_i16::<LittleEndian>()?;
    let bits_per_sample = file.read_i16::<LittleEndian>()?;

    // Skip the unknown 2-byte field (typically contains value 8).
    let _ = file.read_i16::<LittleEndian>()?;

    let capacity = data_size.max(0) as usize;
    let mut pcm_data = Vec::with_capacity(capacity);
    file.read_to_end(&mut pcm_data)?;

    Ok(SnfFile {
        pcmaudio_format,
        number_of_channels,
        sample_rate,
        byte_rate,
        block_align,
        bits_per_sample,
        data_size,
        pcm_data,
    })
}

/// Converts an SNF audio file to standard WAV format on disk.
pub fn extract(from: &Path, to: &Path) -> Result<()> {
    let snf = read(from)?;

    let mut out_file = File::create(to)?;
    out_file.write_all(&snf.to_wav_bytes())?;
    out_file.flush()?;

    Ok(())
}

/// Read a WAV file and parse into SnfFile.
pub fn read_wav(path: &Path) -> Result<SnfFile> {
    let bytes = std::fs::read(path)?;
    SnfFile::from_wav_bytes(&bytes)
}

/// Write SnfFile to disk in SNF format.
pub fn save(path: &Path, snf: &SnfFile) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&snf.to_snf_bytes())?;
    file.flush()?;
    Ok(())
}

/// High-level: read WAV file → parse → write as SNF.
pub fn import_wav(wav_path: &Path, snf_path: &Path) -> Result<()> {
    let snf = read_wav(wav_path)?;
    save(snf_path, &snf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_round_trip() {
        let original = SnfFile {
            pcmaudio_format: 1,
            number_of_channels: 1,
            sample_rate: 44100,
            byte_rate: 88200,
            block_align: 2,
            bits_per_sample: 16,
            data_size: 4,
            pcm_data: vec![0u8, 0, 0, 0],
        };

        let wav_bytes = original.to_wav_bytes();
        let parsed = SnfFile::from_wav_bytes(&wav_bytes).unwrap();

        assert_eq!(original.pcmaudio_format, parsed.pcmaudio_format);
        assert_eq!(original.number_of_channels, parsed.number_of_channels);
        assert_eq!(original.sample_rate, parsed.sample_rate);
        assert_eq!(original.byte_rate, parsed.byte_rate);
        assert_eq!(original.block_align, parsed.block_align);
        assert_eq!(original.bits_per_sample, parsed.bits_per_sample);
        assert_eq!(original.data_size, parsed.data_size);
        assert_eq!(original.pcm_data, parsed.pcm_data);
    }

    #[test]
    fn snf_round_trip() {
        let original = SnfFile {
            pcmaudio_format: 1,
            number_of_channels: 1,
            sample_rate: 22050,
            byte_rate: 44100,
            block_align: 2,
            bits_per_sample: 16,
            data_size: 6,
            pcm_data: vec![0u8, 0, 0, 0, 0, 0],
        };

        let _snf_bytes = original.to_snf_bytes();
        let tmp = std::env::temp_dir().join("test_snf_round_trip.snf");
        save(&tmp, &original).unwrap();
        let reloaded = read(&tmp).unwrap();
        std::fs::remove_file(&tmp).ok();

        assert_eq!(original.pcmaudio_format, reloaded.pcmaudio_format);
        assert_eq!(original.number_of_channels, reloaded.number_of_channels);
        assert_eq!(original.sample_rate, reloaded.sample_rate);
        assert_eq!(original.byte_rate, reloaded.byte_rate);
        assert_eq!(original.block_align, reloaded.block_align);
        assert_eq!(original.bits_per_sample, reloaded.bits_per_sample);
        assert_eq!(original.data_size, reloaded.data_size);
        assert_eq!(original.pcm_data, reloaded.pcm_data);
    }

    #[test]
    fn import_wav_export_snf() {
        let original = SnfFile {
            pcmaudio_format: 1,
            number_of_channels: 2,
            sample_rate: 48000,
            byte_rate: 192000,
            block_align: 4,
            bits_per_sample: 16,
            data_size: 8,
            pcm_data: vec![0u8; 8],
        };

        let wav_path = std::env::temp_dir().join("test_import_export.wav");
        let snf_path = std::env::temp_dir().join("test_import_export.snf");

        std::fs::write(&wav_path, &original.to_wav_bytes()).unwrap();
        import_wav(&wav_path, &snf_path).unwrap();
        let reloaded = read(&snf_path).unwrap();

        std::fs::remove_file(&wav_path).ok();
        std::fs::remove_file(&snf_path).ok();

        assert_eq!(original.pcmaudio_format, reloaded.pcmaudio_format);
        assert_eq!(original.number_of_channels, reloaded.number_of_channels);
        assert_eq!(original.sample_rate, reloaded.sample_rate);
        assert_eq!(original.pcm_data, reloaded.pcm_data);
    }
}
