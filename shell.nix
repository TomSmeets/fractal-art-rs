{ pkgs ? import <nixpkgs> {} }: with pkgs; stdenv.mkDerivation {
    name = "fractal-art-rs";

    buildInputs = [
        xorg.libxcb
    ];

    nativeBuildInputs = [
        cargo
        python3
    ];
}
