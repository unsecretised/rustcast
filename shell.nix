{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.openssl
    pkgs.gcc
    pkgs.pkg-config
    pkgs.glib
    pkgs.gobject-introspection
    pkgs.pango
    pkgs.gtk3
    pkgs.xdotool
  ];
}
