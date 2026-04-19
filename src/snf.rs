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
