{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  packages = with pkgs; [
    nil

    rustup
    typst
    sops
    pkg-config

    nodejs
  ];
}
