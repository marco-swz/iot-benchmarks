{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs@{ self, nixpkgs, flake-utils, rust-overlay, ... }:
  flake-utils.lib.eachSystem [ "x86_64-linux" ] (system: let
	pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlay
            ];
          };
  in rec {
    devShell = pkgs.mkShell {
      buildInputs = with pkgs; [ 
	cmake
	rust-analyzer
	rust-bin.stable.latest.default
	pkg-config
	openssl.dev
      ];
    };
  });
}
