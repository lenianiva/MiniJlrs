{ pkgs ? import <nixpkgs> {} }:

with pkgs;

mkShell {
  DYLD_LIBRARY_PATH="${pkgs.julia-bin}/lib";
  buildInputs = [
    pkgs.julia-bin
  ];
}
