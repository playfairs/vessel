# Vessel documentation

The documentation is split into the [command reference](commands.md), the [veSL binary format](vesl.md), the [polyglot analysis](polyglot.md), the [vsl project format overview](vsl.md), the [chunk specification](format/chunk.md), [binary layout](format/layout.md), and [executable-image boundary](format/executable.md). The implementation intentionally stores a payload in a normal PNG ancillary chunk instead of appending an executable behind the image.

## Verified support model

Vessel validates all image formats in the repository's integration corpus. Version 1 currently creates verified PNG `veSL` containers and provides `vessel run`, which securely extracts and directly launches the verified payload. It does not claim that the image file itself is a native macOS executable.

The requested native Mach-O polyglot is not available for these formats under the operating system's loader rules: the macOS loader requires a Mach-O magic at byte offset zero, while a PNG requires a PNG signature at byte offset zero. Appending a Mach-O or payload after an image is not a valid native executable polyglot and is therefore rejected rather than reported as support.

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

The closest valid architecture is a containerized payload with a native launcher that extracts the payload and executes it from a secure temporary path. That is the supported pathway in the current implementation.