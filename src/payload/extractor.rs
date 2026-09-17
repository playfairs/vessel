use crate::{error::VesselError, format, png};

pub fn extract(image: &[u8]) -> Result<Vec<u8>, VesselError> {
    let document = png::validator::validate(image)?;
    let vessels: Vec<_> = document
        .chunks
        .iter()
        .filter(|chunk| chunk.kind == format::VESSEL_TYPE)
        .collect();
    if vessels.len() > 1 {
        return Err(VesselError::MultipleVesselChunks);
    }
    let vessel = vessels.first().ok_or(VesselError::MalformedVessel(
        "Vessel chunk is missing".into(),
    ))?;
    Ok(format::chunk::decode(&vessel.data)?.to_vec())
}
