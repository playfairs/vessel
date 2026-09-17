# Vessel documentation

The format specification is split into [chunk.md](format/chunk.md) and [layout.md](format/layout.md). The implementation deliberately uses a normal PNG ancillary chunk instead of bytes after `IEND`.

## Executable-image capability

Vessel validates all image formats in the repository's integration corpus. Version 1 currently creates verified PNG Vessel containers and provides `vessel run`, which securely extracts and directly launches the verified payload. It does not claim that the image file itself is a native macOS executable.

The requested native Mach-O polyglot is not available for these formats under their normal specifications: the macOS loader requires a Mach-O magic at byte offset zero, while each image parser requires a different signature at byte offset zero. Appending a Mach-O or payload after an image is not a valid executable polyglot and is therefore rejected rather than reported as support.

| Format | Detect | Validate | Vessel container | Executable polyglot |
| --- | --- | --- | --- | --- |
| PNG | yes | yes | yes | unsupported |
| JPEG / JPG | yes | yes | not implemented | unsupported |
| WebP | yes | yes | not implemented | unsupported |
| GIF | yes | yes | not implemented | unsupported |
| BMP | yes | yes | not implemented | unsupported |
| PBM / PGM / PPM | yes | yes | not implemented | unsupported |
| TGA | yes | yes | not implemented | unsupported |
| TIFF | yes | yes | not implemented | unsupported |