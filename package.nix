{ lib
, rustPlatform
, openssl
, pkgconf }:

rustPlatform.buildRustPackage rec {
  pname = "countr";
  version = "dev.1";

  src = ./.;

  buildInputs = [
    openssl
  ];

  nativeBuildInputs = [
    pkgconf
  ];

  cargoLock.lockFile = src + /Cargo.lock;
  doCheck = false;

  meta = with lib; {
    homepage = "https://github.com/aurakle/countr";
    description = "A backend for counting clicks";
    license = licenses.mit;
  };
}
