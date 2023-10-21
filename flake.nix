{
    description = "A very basic flake";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay.url = "github:oxalica/rust-overlay";
        ros-overlay.url = "github:lopsided98/nix-ros-overlay";
    };

    outputs = inputs@{ self, nixpkgs, flake-utils, rust-overlay, ros-overlay, ... }:
        flake-utils.lib.eachSystem [ "x86_64-linux" ] (system: 
            let
                pkgs = import nixpkgs {
                    inherit system;
                    overlays = [
                        rust-overlay.overlays.default
                        ros-overlay.overlays.default
                    ];
                };
                stdenv = pkgs.stdenv;
                lib = pkgs.lib;
            in rec {
                devShell = pkgs.mkShell {
                    buildInputs = with pkgs; [ 
                        cmake
                        rust-analyzer
                        rust-bin.stable.latest.default
                        pkg-config
                        openssl.dev
                        rosPackages.humble.ros-core
                        colcon
                        #llvmPackages_16.clang-unwrapped
                    ];

                    LIBCLANG_PATH = "${pkgs.llvmPackages_16.libclang.lib}/lib";

                    shellHook = ''
                        # From: https://github.com/NixOS/nixpkgs/blob/1fab95f5190d087e66a3502481e34e15d62090aa/pkgs/applications/networking/browsers/firefox/common.nix#L247-L253
                        # Set C flags for Rust's bindgen program. Unlike ordinary C
                        # compilation, bindgen does not invoke $CC directly. Instead it
                        # uses LLVM's libclang. To make sure all necessary flags are
                        # included we need to look in a few places.
                        export BINDGEN_EXTRA_CLANG_ARGS="$(< ${stdenv.cc}/nix-support/libc-crt1-cflags) \
                          $(< ${stdenv.cc}/nix-support/libc-cflags) \
                          $(< ${stdenv.cc}/nix-support/cc-cflags) \
                          $(< ${stdenv.cc}/nix-support/libcxx-cxxflags) \
                          ${lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${lib.getVersion stdenv.cc.cc}/include"} \
                          ${lib.optionalString stdenv.cc.isGNU "-isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include"} \
                        "
                      '';
                };
            });
}
