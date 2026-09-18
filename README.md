# Vessel

> A vessel for executable payloads.

Vessel embeds an arbitrary binary file in a private PNG ancillary chunk and recovers it byte-for-byte. It validates the PNG and every chunk before writing, leaves image data and existing chunks untouched, and verifies the resulting file again before returning it.

## Installation

```sh
nix develop
nox task build
```

## Installation

Inside the Nix development shell, build Vessel with Nox:

```sh
nix develop
nox task build
```

To install it into a user-local directory:

```sh
nox install --prefix "$HOME/.local"
export PATH="$HOME/.local/bin:$PATH"
```

The executable is then available as `vessel`.

## Commands

```sh
vessel embed image.png program -o vessel.png
vessel create image.png program -o vessel.png
vessel extract vessel.png -o program
vessel inspect vessel.png
vessel verify vessel.png
vessel run vessel.png --argument
```

The complete command reference, including input/output behavior and validation stages, is in [docs/commands.md](docs/commands.md).

`embed` and `create` combine a valid PNG image with one opaque payload. They do not combine two existing Vessel images or two payloads. Embedding rejects empty payloads and images that already contain a Vessel chunk. Extraction rejects missing, malformed, corrupt, unsupported, or duplicate Vessel chunks.

`create` is the format-detecting, final-validation PNG container workflow. `run` extracts a verified payload to a secure temporary executable and invokes it directly; it does not execute the image file. This project does not claim a native macOS executable-image polyglot for PNG or any other tested image format because the loader and image-format requirements are incompatible at byte zero.

## Executable-image polyglot status

Vessel is a container-first tool, not a cross loader magic trick.

The current verified state is:

- `PNG` + `veSL` container: supported
- `PNG` as a native Mach-O executable: not supported
- `JPEG` / `JPG` / `WebP` / `GIF` / `BMP` / `Netpbm` / `TGA` / `TIFF`: supported as image validation only; no executable polyglot claim

The reason is not merely that a naive concatenation fails. The actual binary invariant is stricter: macOS executes files whose first 4 bytes are a Mach-O magic value such as `0xfeedface`, `0xcefaedfe`, or `0xcafebabe`, while a PNG must begin with `89 50 4e 47 0d 0a 1a 0a` at byte offset zero. Those values are mutually exclusive at offset zero, so a single physical file cannot normally satisfy both parsers under the OS loader rules. See [docs/polyglot.md](docs/polyglot.md).

## Format

Vessel uses the valid PNG ancillary private chunk type `veSL`. It is inserted immediately before `IEND`; `IHDR`, `IDAT`, metadata, ordering, and compressed image bytes are otherwise preserved. The chunk contains a 48-byte big-endian header followed by opaque payload bytes. The header stores magic `VSL1`, version `1`, zero flags, a 64-bit payload length, and a SHA-256 digest. The exact byte layout is in [docs/format/layout.md](docs/format/layout.md).

The `veSL` container is the compiled binary payload format. The human-authored project description format is a separate future concept documented in [docs/vsl.md](docs/vsl.md). The actual current implementation is the binary container and the extraction/runtime flow, not a project compiler.

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
