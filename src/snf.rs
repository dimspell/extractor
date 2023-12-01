use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::path::Path;

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

    // What is it? It is was =8
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
