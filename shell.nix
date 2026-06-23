let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  inputsFrom = [ (pkgs.callPackage ./default.nix { }) ];

  packages = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

  shellHook = ''
    export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH" # Needed on Wayland to report the correct display scale
    export WEBKIT_DISABLE_DMABUF_RENDERER=1 # temporary bug fix as per https://github.com/tauri-apps/tauri/issues/10702#issuecomment-2327642878

    export LD_LIBRARY_PATH="${
      pkgs.lib.makeLibraryPath [
        pkgs.gtk3
        pkgs.webkitgtk_4_1
        pkgs.libayatana-appindicator
      ]
    }:$LD_LIBRARY_PATH"
  '';
}
