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
            default = apps.friday;
            friday = utils.lib.mkApp {
              drv = packages.default;
              exePath = "/bin/friday";
            };
            fridaypi = utils.lib.mkApp {
              drv = packages.default;
              exePath = "/bin/fridaypi";
            };
          };

          # switched to rustup for targets as defined in https://nixos.wiki/wiki/Rust
          devShell =
            let
              rustVersion = "1.69";
            in
            mkShell {
              packages = [
                rustup
                cargo-audit
                vhs
                libiconv
                just
                (callPackage ./nix/spin.nix { })
              ];

              shellHook = ''
                export FRIDAY_FILE=testing
                rustup default ${rustVersion}
                rustup target add wasm32-wasi
              '';
            };
        }
      );
}
