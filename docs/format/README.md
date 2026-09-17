# Vessel format

Version 1 stores one opaque payload in a PNG `veSL` chunk. The chunk is private, ancillary, reserved-bit compliant, and unsafe-to-copy. It is placed immediately before `IEND`.

Embedding rejects an existing Vessel chunk. Extraction and verification reject duplicates rather than selecting one silently.