use std::{fs, path::Path};
use vessel::{VesselError, error::io_error, image, png};

pub fn run(image: &Path) -> Result<(), VesselError> {
    let bytes = fs::read(image).map_err(|source| io_error(image, source))?;
    let format = image::detect(&bytes)?;
    if format != image::ImageFormat::Png {
        println!(
            "valid {} image; capability: {:?}",
            format.name(),
            format.capability()
        );
        return Ok(());
    }
    let document = png::validator::validate(&bytes)?;
    let vessels: Vec<_> = document
        .chunks
        .iter()
        .filter(|chunk| chunk.kind == vessel::format::VESSEL_TYPE)
        .collect();
    if vessels.len() > 1 {
        return Err(VesselError::MultipleVesselChunks);
    }
    if let Some(vessel) = vessels.first() {
        vessel::format::chunk::decode(&vessel.data)?;
    }
    println!(
        "valid PNG{}",
        if vessels.is_empty() {
            ""
        } else {
            " with valid Vessel payload"
        }
    );
    Ok(())
}
