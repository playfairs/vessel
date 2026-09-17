use std::{fs, path::Path};
use vessel::{VesselError, error::io_error, format, image, png};

pub fn run(image: &Path) -> Result<(), VesselError> {
    let bytes = fs::read(image).map_err(|source| io_error(image, source))?;
    let format = image::detect(&bytes)?;
    println!("Format: {}", format.name());
    println!("Bytes: {}", bytes.len());
    println!("Capability: {:?}", format.capability());
    if format == image::ImageFormat::Png {
        let document = png::validator::validate(&bytes)?;
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
            println!(
                "Payload: {} bytes",
                format::chunk::decode(&vessel.data)?.len()
            );
        }
    }
    Ok(())
}
