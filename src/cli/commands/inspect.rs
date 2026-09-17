use std::{fs, path::Path};
use vessel::{VesselError, error::io_error, format, png};

pub fn run(image: &Path) -> Result<(), VesselError> {
    let bytes = fs::read(image).map_err(|source| io_error(image, source))?;
    let document = png::validator::validate(&bytes)?;
    println!("PNG: {} bytes", bytes.len());
    println!("Chunks: {}", document.chunks.len());
    for chunk in &document.chunks {
        println!(
            "{} {} bytes",
            String::from_utf8_lossy(&chunk.kind),
            chunk.data.len()
        );
    }
    let vessels: Vec<_> = document
        .chunks
        .iter()
        .filter(|chunk| chunk.kind == format::VESSEL_TYPE)
        .collect();
    println!("Vessel chunks: {}", vessels.len());
    if let Some(vessel) = vessels.first() {
        let payload = format::chunk::decode(&vessel.data)?;
        println!("Payload: {} bytes", payload.len());
    }
    Ok(())
}
