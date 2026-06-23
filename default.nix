{
  pkgs ? import <nixpkgs> { },
  lib ? pkgs.lib,
  stdenv ? pkgs.stdenv,
}:
let
  # https://nixos.org/manual/nixpkgs/stable/#javascript-pnpm
  # It is recommended to pin pnpm to a major version, due to regular breaking changes in the store format
  # The latest major version is always available under `pkgs.pnpm`
  pnpm = pkgs.pnpm_11;
in
pkgs.rustPlatform.buildRustPackage (finalAttrs: {
  pname = "funnymic";
  version = "0.1";
  cargoLock.lockFile = ./src-tauri/Cargo.lock;
  src = ./.;

  pnpmDeps = pkgs.fetchPnpmDeps {
    inherit (finalAttrs) pname version src;
    inherit pnpm;
    fetcherVersion = 4;
    hash = "sha256-oEC7PgTatOtmojNG/fbU0EBVc5QGMOxc2RAIYwthka8=";
  };

  nativeBuildInputs =
    with pkgs;
    [
      cargo-tauri.hook

      rustPlatform.bindgenHook
      pkg-config

      nodejs
      pnpmConfigHook
    ]
    ++ lib.optionals stdenv.hostPlatform.isLinux [ wrapGAppsHook4 ]
    ++ [ pnpm ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux (
    with pkgs;
    [
      xorgproto
      libx11
      libXi
      libxtst

      # for Tauri
      librsvg
      webkitgtk_4_1
      # openssl # needed?
      libayatana-appindicator
    ]
  );

  # Set our Tauri source directory
  cargoRoot = "src-tauri";
  # And make sure we build there too
  buildAndTestSubdir = finalAttrs.cargoRoot;
})
