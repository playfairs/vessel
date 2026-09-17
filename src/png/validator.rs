use crate::{error::VesselError, png::PngDocument};

pub fn validate(input: &[u8]) -> Result<PngDocument, VesselError> {
    PngDocument::parse(input)
}
