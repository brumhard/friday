{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, utils, naersk, fenix }: utils.lib.eachDefaultSystem
    (system:
      let
        name = "friday";
        version = "latest";
        # https://discourse.nixos.org/t/using-nixpkgs-legacypackages-system-vs-import/17462/7
        pkgs = nixpkgs.legacyPackages.${system};
        toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain;
          sha256 = "sha256-eMJethw5ZLrJHmoN2/l0bIyQjoTX1NsvalWSscTixpI=";
        };
        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };
      in
      with pkgs;
      rec {
        packages = {
          default = packages.${name};
          "${name}" = naersk'.buildPackage {
            inherit name version;
            src = ./.;

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
          mkShell {
            packages = [
              rustup
              cargo-audit
              vhs
              libiconv
              yq-go
              bat
              mask
            ];

            shellHook = ''
              export FRIDAY_FILE=testing
            '';
          };
      }
    );
}
