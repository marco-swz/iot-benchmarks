let
    rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
    pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };

in pkgs.mkShell {
    buildInputs = with pkgs; [
        cmake
        rust-analyzer
        rust-bin.stable.latest.default
        pkgconfig
        openssl.dev
    ];
}
