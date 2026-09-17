{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "vessel";
  version = "0.1.0";
  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;
  meta = {
    description = "A vessel for executable payloads";
    homepage = "https://github.com/playfairs/vessel";
    license = lib.licenses.unlicense;
    maintainers = [ ];
  };
}
