{
  description = "Rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
          (_: prev: {
            rust-toolchain = prev.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          })
        ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell rec {
        strictDeps = true;

        nativeBuildInputs = with pkgs; [
          gcc
          pkg-config
          sqlx-cli
          rust-toolchain
        ];

        buildInputs = with pkgs; [
          atk
          glib
          gtk3
          cairo
          pango
          openssl
          xdotool
          wayland
          xorg.libXi
          gdk-pixbuf
          xorg.libxcb
          xorg.libX11
          libxkbcommon
          vulkan-loader
          xorg.libXrandr
          xorg.libXcursor
          gobject-introspection
          libayatana-appindicator
        ];

        shellHook = ''
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${toString (pkgs.lib.makeLibraryPath buildInputs)}";
        '';
      };
    };
}
