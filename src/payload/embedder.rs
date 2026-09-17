use crate::{error::VesselError, format, payload::digest::sha256, png};

pub fn embed(image: &[u8], payload: &[u8]) -> Result<Vec<u8>, VesselError> {
    if payload.is_empty() {
        return Err(VesselError::EmptyPayload);
    }
    let mut document = png::validator::validate(image)?;
    if document
        .chunks
        .iter()
        .any(|chunk| chunk.kind == format::VESSEL_TYPE)
    {
        return Err(VesselError::MultipleVesselChunks);
    }
    let vessel = format::chunk::encode(payload, sha256(payload))?;
    let iend = document
        .chunks
        .iter()
        .position(|chunk| chunk.kind == *b"IEND")
        .ok_or(VesselError::MissingIend)?;
    document.chunks.insert(iend, vessel);
    let output = document.encode()?;
    png::validator::validate(&output)?.encode()
}
