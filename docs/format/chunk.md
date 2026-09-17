# Vessel PNG chunk

The four ASCII bytes `veSL` form the PNG chunk type. Byte 0 is lowercase, making the chunk ancillary; byte 1 is lowercase, making it private; byte 2 is uppercase, satisfying the PNG reserved-bit rule; byte 3 is uppercase, marking it unsafe to copy when the image is transformed.

The PNG chunk remains a normal length/type/data/CRC record. Its data is binary and is never interpreted as text. The PNG CRC covers the four type bytes followed by all chunk data and is calculated with the standard PNG CRC-32 polynomial.

There is exactly one `veSL` chunk in version 1. It appears immediately before `IEND`. A second chunk is an error.