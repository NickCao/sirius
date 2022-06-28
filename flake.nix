{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let pkgs = import nixpkgs { inherit system; overlays = [ self.overlay ]; }; in
        rec {
          defaultPackage = pkgs.sirius;
          devShell = pkgs.mkShell {
            nativeBuildInputs = [ pkgs.rust-analyzer pkgs.rustfmt ];
            inputsFrom = [ defaultPackage ];
          };
        }
      ) //
    {
      overlay = final: prev: {
        sirius = final.rustPlatform.buildRustPackage {
          name = "sirius";
          src = self;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "libnar-0.1.0" = "sha256-rzAxomiOtMuvJMTvbMnW12POUV5t177wgdi3pAHSjFE=";
            };
          };
        };
      };
    };
}
