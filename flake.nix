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
        # pythonPackages = pkgs.python3Packages;
      in
      {
        devShells.default = pkgs.mkShell {
          # venvDir = "./.venv";
          buildInputs = with pkgs; [
            # clang
            # llvmPackages_16.bintools
            # libusb1
            # openssl
            # pkg-config
            extism-cli
            curl
            git-cliff
            (rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" ];
            })
          ]
          ++
          (with pkgs.darwin.apple_sdk.frameworks; pkgs.lib.optionals pkgs.stdenv.isDarwin [
            SystemConfiguration
          #   IOKit
            Security
            CoreFoundation
          #   AppKit
          ]);
          # ++
          # [
          #   # A Python interpreter including the 'venv' module is required to bootstrap
          #   # the environment.
          #   pythonPackages.python

          #   # This executes some shell code to initialize a venv in $venvDir before
          #   # dropping into the shell
          #   pythonPackages.venvShellHook

          #   # Those are dependencies that we would like to use from nixpkgs, which will
          #   # add them to PYTHONPATH and thus make them accessible from within the venv.
          #   pythonPackages.numpy
          # ];

          # # Run this command, only after creating the virtual environment
          # postVenvCreation = ''
          #   unset SOURCE_DATE_EPOCH
          #   pip install -r requirements.txt
          # '';

          # # Now we can execute any commands within the virtual environment.
          # # This is optional and can be left out to run pip manually.
          # postShellHook = ''
          #   # allow pip to install wheels
          #   unset SOURCE_DATE_EPOCH
          # '';
        };
      }
    );
}
