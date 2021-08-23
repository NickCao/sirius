{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let pkgs = import nixpkgs { inherit system; overlays = [ self.overlay rust-overlay.overlay ]; }; in
        rec {
          defaultPackage = pkgs.sirius;
          devShell = pkgs.mkShell {
            buildInputs = [ (pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override { extensions = [ "rust-analyzer-preview" "rust-src" ]; })) ];
          };
        }
      ) //
    {
      overlay = final: prev:
        let
          toolchain = final.rust-bin.nightly.latest.default;
          platform = final.makeRustPlatform { cargo = toolchain; rustc = toolchain; };
        in
        {
          sirius = platform.buildRustPackage {
            name = "sirius";
            src = final.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };
        };
    };
}
