{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem
      (system:
        let
          name = "friday";
          version = "latest";
          pkgs = import nixpkgs { inherit system; };
        in
        with pkgs;
        rec {
          packages = {
            default = packages.${name};
            "${name}" = rustPlatform.buildRustPackage {
              pname = name;
              version = version;
              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
              };

              meta = with lib; {
                description = "Manage stuff to do on fridays.";
                homepage = "https://github.com/brumhard/${name}";
                maintainers = with maintainers; [ brumhard ];
                license = licenses.mit;
              };
            };
          };

          apps = {
            default = apps.cli;
            cli = utils.lib.mkApp {
              drv = packages.default;
              exePath = "/bin/cli";
            };
            server = utils.lib.mkApp {
              drv = packages.default;
              exePath = "/bin/server";
            };
          };

          devShell = mkShell {
            packages = [
              rustc
              cargo
              rustfmt
              rust-analyzer
              clippy
            ];

            shellHook = ''
              export RUST_SRC_PATH="${rustPlatform.rustLibSrc}";
              export FRIDAY_FILE=testing
            '';
          };
        }
      );
}
