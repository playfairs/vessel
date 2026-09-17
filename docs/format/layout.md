# Vessel data layout

All integer fields are unsigned, fixed-width, and big-endian. Offsets are relative to the first byte of the `veSL` data field. The data field is 48 bytes of header followed by payload bytes.

| Offset | Size | Field | Encoding |
| ---: | ---: | --- | --- |
| 0 | 4 | Magic | ASCII `VSL1` |
| 4 | 2 | Version | `u16`, value `1` |
| 6 | 2 | Flags | `u16`, value `0` |
| 8 | 8 | Payload length | `u64`, number of bytes at offset 48 |
| 16 | 32 | Payload digest | SHA-256 of payload bytes |
| 48 | length | Payload | Opaque bytes, including null and non-UTF-8 bytes |

The PNG chunk length equals `48 + payload length` and is encoded as the PNG `u32` length field. A payload must be non-empty. Decoders require the declared length to equal the remaining data exactly and then verify SHA-256 before returning bytes.