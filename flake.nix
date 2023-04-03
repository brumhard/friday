{
  description = "local development setup";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    utils.url = "github:numtide/flake-utils";
  };

  # see here for builds: https://serokell.io/blog/practical-nix-flakes
  # or https://nix-community.github.io/dream2nix/subsystems/rust.html

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      # see https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/9
        # -> might be nice to switch to https://github.com/oxalica/rust-overlay at some time
      with pkgs;
      {
        devShell = mkShell {
          packages = [
            rustc
            cargo
            rustfmt
            rust-analyzer
            clippy
          ];

          shellHook = ''
            export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          '';
        };
      }
    );
}
