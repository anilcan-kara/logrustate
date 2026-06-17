use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use anyhow::{Context, Result};
use flate2::Compression;
use flate2::write::GzEncoder;

pub fn compress_file(src: &Path, dest: &Path) -> Result<()> {
    let mut input = File::open(src)
        .with_context(|| format!("Failed to open source file for compression: {:?}", src))?;

    let output = File::create(dest)
        .with_context(|| format!("Failed to create destination file for compression: {:?}", dest))?;

    let mut encoder = GzEncoder::new(output, Compression::best());
    let mut buffer = [0; 8192];

    loop {
        let count = input.read(&mut buffer)
            .with_context(|| format!("Failed to read source file: {:?}", src))?;
        if count == 0 {
            break;
        }
        encoder.write_all(&buffer[..count])
            .with_context(|| format!("Failed to write to compression encoder: {:?}", dest))?;
    }

    encoder.finish()
        .with_context(|| format!("Failed to finalize compression encoder: {:?}", dest))?;

    Ok(())
}
