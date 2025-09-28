{ pkgs ? import <nixpkgs> {} }:

with pkgs;

mkShell {
  DYLD_LIBRARY_PATH="${julia}/lib";
  buildInputs = [
    pkgs.julia-bin
  ];
}
