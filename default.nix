{
  pkgs ? import <nixpkgs> { },
}:
pkgs.rustPlatform.buildRustPackage (finalAttrs: {
  pname = "domainwatch";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;

  nativeBuildInputs = with pkgs; [
    rustPlatform.bindgenHook
    pkg-config
  ];

  buildInputs = with pkgs; [
    xorgproto
    libx11
    libXi
    libxtst
  ];
})
