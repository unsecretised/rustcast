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
      devShells.${system}.default = pkgs.mkShell {
        strictDeps = true;

        nativeBuildInputs = [
          pkgs.rust-toolchain
          pkgs.pkg-config
          pkgs.openssl
          pkgs.sqlx-cli
          pkgs.gcc
        ];

        shellHook = ''
          export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig
        '';
      };
    };
}
