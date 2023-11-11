{
  description = "Nabu - A simple HTTP crates registry";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        nabu = pkgs.rustPlatform.buildRustPackage {
          pname = "nabu";
          version = "git";
          src = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };

          nativeBuildInputs = with pkgs; [ postgresql ];
        };
      in {
        packages = {
          mailconfig = nabu;
          default = nabu;
        };
        devShell = pkgs.callPackage ./shell.nix { };
      }));
}
