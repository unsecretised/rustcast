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
    pkgs.libayatana-appindicator
  ];

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.libayatana-appindicator}/lib:$LD_LIBRARY_PATH
  '';
}
