{
  description = "ice";

  nixConfig = {
    extra-substituters = [
      "https://mirrors.ustc.edu.cn/nix-channels/store"
    ];
    trusted-substituters = [
      "https://mirrors.ustc.edu.cn/nix-channels/store"
    ];
  };


  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # clang
            # llvmPackages_16.bintools
            # libusb1
            # openssl
            # pkg-config
            libiconv
            curl
            git-cliff
            (rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" ];
            })
          ]
          ++
          (pkgs.lib.optionals pkgs.stdenv.isDarwin(with pkgs.darwin.apple_sdk.frameworks; [
            SystemConfiguration
          #   IOKit
            Security
            CoreFoundation
          #   AppKit
          ]
          ++ 
          [
            libiconv-darwin
          ]))
          ;
        };
      }
    );
}
