# veSL binary container

`veSL` is the compiled binary payload container used by Vessel.

## Current implementation

The current implementation stores one opaque payload in a valid PNG ancillary chunk whose type is `veSL`.

The chunk is inserted immediately before `IEND` in a PNG file. This keeps the image file valid for PNG parsing while leaving the original pixel data and chunk ordering intact.

## Binary layout

The binary payload container is structured as:

```text
veSL chunk data
├── 4 bytes: magic = "VSL1"
├── 2 bytes: version = 1 (big-endian)
├── 2 bytes: flags = 0 (big-endian)
├── 8 bytes: payload length (big-endian u64)
├── 32 bytes: SHA-256 digest of the payload
└── payload bytes
```

The exact implementation is in [format/header.rs](format/header.rs) and [format/chunk.rs](format/chunk.rs).

## Validation rules

The current decoder validates:

- minimum length of 48 bytes for the header,
- exact magic value `VSL1`,
- supported version value,
- zero flags,
- declared payload length,
- digest equality against the payload bytes,
- exactly one `veSL` chunk in a PNG.

The payload is treated as opaque bytes. Vessel does not interpret its contents as code, metadata, or an instruction stream.

## Security properties

Vessel treats all inputs as untrusted:

- it rejects malformed headers,
- it rejects unsupported versions,
- it rejects mismatched payload lengths,
- it rejects digest mismatches,
- it validates the image before writing output,
- it writes output only after validation succeeds.

This protects the extracted payload path, but it does not make the image itself a native executable.
