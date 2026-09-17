# Vessel

> A vessel for executable payloads.

Vessel embeds an arbitrary binary file in a private PNG ancillary chunk and recovers it byte-for-byte. It validates the PNG and every chunk before writing, leaves image data and existing chunks untouched, and verifies the resulting file again before returning it.

## Installation

```sh
nix develop
nox task build
```

## Usage

```sh
vessel embed image.png program -o vessel.png
vessel create image.png program -o vessel.png
vessel extract vessel.png -o program
vessel inspect vessel.png
vessel verify vessel.png
vessel run vessel.png --argument
```

Embedding rejects empty payloads and images that already contain a Vessel chunk. Extraction rejects missing, malformed, corrupt, unsupported, or duplicate Vessel chunks. Vessel never executes payloads.

`create` is the validated PNG container workflow. `run` extracts a verified payload to a secure temporary executable and invokes it directly; it does not execute the image file. Native macOS executable-image polyglots are currently unsupported because the image and Mach-O formats require incompatible byte-zero signatures.

## Format

Vessel uses the valid PNG ancillary private chunk type `veSL`. It is inserted immediately before `IEND`; `IHDR`, `IDAT`, metadata, ordering, and compressed image bytes are otherwise preserved. The chunk contains a 48-byte big-endian header followed by opaque payload bytes. The header stores magic `VSL1`, version `1`, zero flags, a 64-bit payload length, and a SHA-256 digest. The exact byte layout is in [docs/format/layout.md](docs/format/layout.md).

Only one payload is supported in version 1. Vessel does not infer or store an executable format.

## Development

Nox is the primary project interface:

```sh
nox task build
nox task test
nox task check
nox task format
nox task release
nox task cli
```

Cargo remains the Rust package manager and lockfile owner.

## Security and limitations

Vessel is a container, not an execution mechanism. Treat embedded bytes and input PNGs as untrusted. SHA-256 detects payload changes but does not provide authenticity. The implementation keeps the chunk table in memory and does not decode pixels.
