pub mod chunk;
pub mod header;
pub mod payload;
pub mod version;

pub const VESSEL_TYPE: [u8; 4] = *b"veSL";
pub const MAGIC: [u8; 4] = *b"VSL1";
