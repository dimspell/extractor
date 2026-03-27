use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
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

#[derive(Debug)]
struct SnfFileHeader {
    data_size: i32,
    pcmaudio_format: i16,
    number_of_channels: i16,
    sample_rate: i32,
    byte_rate: i32,
    block_align: i16,
    bits_per_sample: i16,
}

/// Converts an SNF audio file to standard WAV format
///
/// This function reads a Dispel game SNF audio file and converts it to a standard
/// RIFF WAVE format that can be played by any audio player.
///
/// # SNF to WAV Conversion Process:
///
/// 1. Read the SNF header (20 bytes) containing audio metadata
/// 2. Skip the unknown 2-byte field (typically value 8)
/// 3. Write standard RIFF WAVE header using the SNF metadata
/// 4. Copy the raw PCM audio data from SNF to WAV format
///
/// # Arguments
///
/// * `from` - Path to the input SNF file
/// * `to` - Path where the output WAV file will be saved
///
/// # Returns
///
/// Result<()> indicating success or failure
///
/// # WAV File Structure Created:
///
/// RIFF Header (12 bytes):
/// - "RIFF" signature (4 bytes)
/// - File size - 8 (4 bytes)
/// - "WAVE" signature (4 bytes)
///
/// fmt Chunk (24 bytes):
/// - "fmt " signature (4 bytes)
/// - Chunk size (4 bytes) - always 16
/// - Audio format (2 bytes) - from SNF header
/// - Number of channels (2 bytes) - from SNF header
/// - Sample rate (4 bytes) - from SNF header
/// - Byte rate (4 bytes) - from SNF header
/// - Block align (2 bytes) - from SNF header
/// - Bits per sample (2 bytes) - from SNF header
///
/// data Chunk (8 + data_size bytes):
/// - "data" signature (4 bytes)
/// - Data size (4 bytes) - from SNF header
/// - Raw PCM audio data - copied from SNF file
pub fn extract(from: &Path, to: &Path) -> Result<()> {
    let mut in_file = File::open(from)?;

    let snf: SnfFileHeader = SnfFileHeader {
        data_size: in_file.read_i32::<LittleEndian>()?,
        pcmaudio_format: in_file.read_i16::<LittleEndian>()?,
        number_of_channels: in_file.read_i16::<LittleEndian>()?,
        sample_rate: in_file.read_i32::<LittleEndian>()?,
        byte_rate: in_file.read_i32::<LittleEndian>()?,
        block_align: in_file.read_i16::<LittleEndian>()?,
        bits_per_sample: in_file.read_i16::<LittleEndian>()?,
    };

    // Skip the unknown 2-byte field (typically contains value 8)
    _ = in_file.read_i16::<LittleEndian>()?;

    let mut out_file = File::create(to)?;
    write!(out_file, "RIFF")?;
    out_file.write_i32::<LittleEndian>(snf.data_size + 44)?;
    write!(out_file, "WAVE")?;
    write!(out_file, "fmt ")?;
    out_file.write_i32::<LittleEndian>(16)?;
    out_file.write_i16::<LittleEndian>(snf.pcmaudio_format)?;
    out_file.write_i16::<LittleEndian>(snf.number_of_channels)?;
    out_file.write_i32::<LittleEndian>(snf.sample_rate)?;
    out_file.write_i32::<LittleEndian>(snf.byte_rate)?;
    out_file.write_i16::<LittleEndian>(snf.block_align)?;
    out_file.write_i16::<LittleEndian>(snf.bits_per_sample)?;
    write!(out_file, "data")?;
    out_file.write_i32::<LittleEndian>(snf.data_size)?;

    // Copy raw PCM audio data from SNF to WAV format
    loop {
        let mut buf = [0; 2];
        let num_bytes = in_file.read(&mut buf)?;
        if num_bytes == 0 {
            break;
        }

        out_file.write(&buf)?;
    }
    out_file.flush()?;

    Ok(())
}
