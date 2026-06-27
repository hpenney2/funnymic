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
  cargoLock = {
    lockFile = ./src-tauri/Cargo.lock;
  };
  src = builtins.filterSource (
    path: type: !(type == "directory" && baseNameOf path == "node_modules")
  ) ./.;

  pnpmDeps = pkgs.fetchPnpmDeps {
    inherit (finalAttrs) pname version src;
    inherit pnpm;
    fetcherVersion = 4;
    hash = "sha256-rYbjU09KmqGxn4326BIla1irNSUi8jg+xTKvMp7vtQM=";
  };

  nativeBuildInputs =
    with pkgs;
    [
      cargo-tauri.hook

      # rustPlatform.bindgenHook
      pkg-config

      nodejs
      pnpmConfigHook
    ]
    ++ lib.optionals stdenv.hostPlatform.isLinux [ wrapGAppsHook4 ]
    ++ [ pnpm ];

  buildInputs =
    with pkgs;
    [
      # for Tauri
      librsvg
      webkitgtk_4_1
      # openssl # needed?
      libayatana-appindicator
    ]
    ++ lib.optionals stdenv.hostPlatform.isLinux (
      with pkgs;
      [
        xorgproto
        libx11
        libXi
        libxtst
      ]
    );

  postInstall = ''
    wrapProgram $out/bin/${finalAttrs.pname} \
      --prefix LD_LIBRARY_PATH : "${
        pkgs.lib.makeLibraryPath (
          with pkgs;
          [
            gtk3
            webkitgtk_4_1
            libayatana-appindicator
          ]
        )
      }"
  ''
  + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    wrapProgram $out/bin/${finalAttrs.pname} --set WEBKIT_DISABLE_COMPOSITING_MODE "1"
  '';

  # Set our Tauri source directory
  cargoRoot = "src-tauri";
  # And make sure we build there too
  buildAndTestSubdir = finalAttrs.cargoRoot;
})
