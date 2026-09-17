# Command reference

Vessel treats every input as untrusted binary data. It reads input files as bytes, detects image formats from binary signatures, validates structure before changing anything, and never interprets payload bytes as text.

## Global syntax

```text
vessel <COMMAND> [OPTIONS]
```

Run `vessel --help` for the command list and `vessel <COMMAND> --help` for command-specific help. Paths are passed directly to the filesystem. The output path is required for commands that write a file. Existing output files are passed to the operating system's normal write operation and may be replaced according to filesystem permissions.

## `vessel create`

```sh
vessel create <IMAGE> <PAYLOAD> -o <OUTPUT>
```

Creates a new validated Vessel image. The implementation performs these steps in order:

1. Reads `IMAGE` completely.
2. Detects its binary format using magic bytes and structural validation.
3. Reads `PAYLOAD` completely as opaque bytes.
4. Accepts only PNG for Vessel container creation. JPEG/JPG, WebP, GIF, BMP, PBM, PGM, PPM, TGA, and TIFF are detected and validated but return an explicit unsupported-polyglot error.
5. Rejects an empty payload.
6. Rejects a PNG that already contains a `veSL` chunk.
7. Calculates SHA-256 over the payload.
8. Inserts one `veSL` chunk immediately before `IEND` without decoding pixels or changing existing PNG chunks.
9. Re-parses and validates the resulting PNG, including chunk lengths and CRCs.
10. Writes the validated bytes to `OUTPUT`.

Example:

```sh
vessel create examples/images/png/image.png examples/binaries/hello -o hello.png
```

The result is a valid PNG container. It is not a native executable image, so `chmod +x hello.png; ./hello.png` is expected to fail with an exec-format error on macOS. Use `vessel run hello.png` instead.

## `vessel embed`

```sh
vessel embed <IMAGE> <PAYLOAD> -o <OUTPUT>
```

The lower-level PNG embedding command. It reads both files, validates the input PNG, rejects empty payloads and existing `veSL` chunks, calculates SHA-256, inserts the Vessel chunk before `IEND`, and validates the resulting PNG before writing it. Unlike `create`, it operates directly through the PNG embedding path and does not run the general image-format detection command first. In the current implementation it is still PNG-only because the PNG parser owns the operation.

## `vessel extract`

```sh
vessel extract <IMAGE> -o <OUTPUT>
```

Extracts the exact payload bytes from a Vessel PNG:

1. Reads and validates the PNG signature, structure, chunk lengths, and CRCs.
2. Locates `veSL` chunks by their structured PNG chunk type, not by scanning arbitrary bytes.
3. Rejects a missing chunk or more than one Vessel chunk.
4. Validates magic, version, flags, declared payload length, and SHA-256.
5. Writes the payload bytes unchanged to `OUTPUT`.

It does not infer an executable format, decode text, add an extension, or execute the payload.

Example:

```sh
vessel extract hello.png -o extracted
cmp examples/binaries/hello extracted
```

## `vessel inspect`

```sh
vessel inspect <IMAGE>
```

Reads and validates the image, then prints its detected format, byte count, and capability. For PNG it also prints each chunk type and data length, the number of Vessel chunks, and the decoded payload length when a Vessel chunk exists. It does not write files or execute payloads.

Current capability values are:

- `ContainerOnly`: the format can currently carry a validated Vessel container, as with PNG.
- `Unsupported`: the format can be detected and structurally validated, but Vessel does not currently insert a container into it.
- `ExecutablePolyglot`: reserved for a format-specific native executable strategy; none is currently implemented.

## `vessel verify`

```sh
vessel verify <IMAGE>
```

Validates the image without writing it. For every supported image format it checks the format signature and structural bounds. For PNG it additionally checks PNG chunk CRCs, the Vessel chunk count, Vessel header, payload length, and payload SHA-256 when a Vessel chunk is present.

For non-PNG formats, successful verification means only that the image itself is structurally valid. It does not claim that a Vessel container is present, because non-PNG container insertion is not implemented.

## `vessel run`

```sh
vessel run <IMAGE> [ARGUMENT...]
```

Runs the payload from a verified Vessel PNG without executing the image file:

1. Reads and detects the image.
2. Requires PNG format.
3. Extracts and verifies the Vessel payload, including its SHA-256 digest.
4. Creates an unpredictable temporary directory using the operating system temporary area.
5. Writes the payload to a file named `payload` inside that directory.
6. Sets Unix permissions to `0700`.
7. Starts the temporary file directly with `std::process::Command`, never through a shell.
8. Forwards every argument with its original boundary and inherits the current environment.
9. Waits for the process and returns its exit code. A signal termination is mapped to `128 + signal` where supported.
10. Removes the temporary directory when the command returns.

Example:

```sh
vessel run hello.png foo "two words"
```

The payload receives two arguments: `foo` and `two words`. No `chmod` is needed on `hello.png`, and the current working directory is not used to locate the payload.

## Errors and safety

Typical failures identify filesystem access, invalid or truncated image data, unsupported image formats, malformed Vessel headers, unsupported Vessel versions, invalid payload lengths, digest mismatches, duplicate Vessel chunks, and unavailable executable-polyglot support. Vessel never silently repairs malformed input, executes an image, searches arbitrary bytes for a payload, or invokes a shell command assembled from arguments.