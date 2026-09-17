use crate::{
    error::VesselError,
    format::{VESSEL_TYPE, header},
    png::PngChunk,
};

pub fn encode(payload: &[u8], digest: [u8; 32]) -> Result<PngChunk, VesselError> {
    let length = u64::try_from(payload.len())
        .map_err(|_| VesselError::InvalidStructure("payload length exceeds u64".into()))?;
    let mut data = header::encode(length, digest);
    data.extend_from_slice(payload);
    PngChunk::new(VESSEL_TYPE, data)
}

pub fn decode(data: &[u8]) -> Result<&[u8], VesselError> {
    let (_, declared, digest) = header::decode(data)?;
    let available = data
        .len()
        .checked_sub(header::HEADER_LEN)
        .ok_or(VesselError::MalformedVessel("payload is absent".into()))?;
    let expected = usize::try_from(declared).map_err(|_| VesselError::PayloadLength {
        declared,
        available,
    })?;
    if expected != available {
        return Err(VesselError::PayloadLength {
            declared,
            available,
        });
    }
    let payload = &data[header::HEADER_LEN..];
    if sha256(payload) != digest {
        return Err(VesselError::PayloadDigest);
    }
    Ok(payload)
}

fn sha256(payload: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(payload).into()
}
