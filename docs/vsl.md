# .vsl project description format

This project does not currently implement a complete `.vsl` authoring format.

The current implementation is binary-container focused: `veSL` is the serialized container format stored within a PNG. A human-authored project description format is a future layer above the compiled container, not the implemented runtime format.

## Intended conceptual shape

The design for a future project file would be similar to:

```text
project "hello" {
    image = "images/image.png"
    output = "dist/hello.png"

    executable "main" {
        source = "binaries/hello"
    }

    entrypoint = "main"

    metadata {
        description = "Hello Vessel image"
    }
}
```

That format would resolve paths relative to the `.vsl` file itself, validate inputs, and then compile a `veSL` container and polyglot build artifact. The current implementation does not ship a parser or compiler for this format.

## Current boundary

The implemented work is:

- PNG validation,
- PNG chunk insertion for `veSL`,
- `veSL` header validation,
- payload extraction,
- secure extracted-payload execution.

This is a working container-and-runtime design. It is not a project compiler or a native executable-image polyglot builder.
