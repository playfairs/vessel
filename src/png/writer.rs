use crate::{error::VesselError, png::PngDocument};

pub fn write(document: &PngDocument) -> Result<Vec<u8>, VesselError> {
    document.encode()
}
