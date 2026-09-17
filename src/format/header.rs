use crate::{
    error::VesselError,
    format::{MAGIC, version::CURRENT},
};

pub const HEADER_LEN: usize = 48;

pub fn encode(payload_len: u64, digest: [u8; 32]) -> Vec<u8> {
    let mut header = Vec::with_capacity(HEADER_LEN);
    header.extend_from_slice(&MAGIC);
    header.extend_from_slice(&CURRENT.to_be_bytes());
    header.extend_from_slice(&0u16.to_be_bytes());
    header.extend_from_slice(&payload_len.to_be_bytes());
    header.extend_from_slice(&digest);
    header
}

pub fn decode(data: &[u8]) -> Result<(u16, u64, [u8; 32]), VesselError> {
    if data.len() < HEADER_LEN {
        return Err(VesselError::MalformedVessel(
            "header is shorter than 48 bytes".into(),
        ));
    }
    if data[..4] != MAGIC {
        return Err(VesselError::MalformedVessel("magic is invalid".into()));
    }
    let version = u16::from_be_bytes(
        data[4..6]
            .try_into()
            .map_err(|_| VesselError::MalformedVessel("version is truncated".into()))?,
    );
    let flags = u16::from_be_bytes(
        data[6..8]
            .try_into()
            .map_err(|_| VesselError::MalformedVessel("flags are truncated".into()))?,
    );
    if flags != 0 {
        return Err(VesselError::MalformedVessel(
            "unsupported flags are set".into(),
        ));
    }
    let payload_len = u64::from_be_bytes(
        data[8..16]
            .try_into()
            .map_err(|_| VesselError::MalformedVessel("payload length is truncated".into()))?,
    );
    let digest: [u8; 32] = data[16..48]
        .try_into()
        .map_err(|_| VesselError::MalformedVessel("digest is truncated".into()))?;
    if version != CURRENT {
        return Err(VesselError::UnsupportedVersion(version));
    }
    Ok((version, payload_len, digest))
}
