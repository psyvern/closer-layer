{
  description = "A wayland native, highly customizable runner.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default-linux";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    flake-parts,
    systems,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [flake-parts.flakeModules.easyOverlay];
      systems = import systems;

      perSystem = {
        self',
        config,
        pkgs,
        ...
      }: let
        inherit (pkgs) callPackage;
      in {
        packages = let
          lockFile = ./Cargo.lock;
        in {
          default = self'.packages.closer-layer;

          closer-layer = callPackage ./nix/closer-layer.nix {inherit inputs lockFile;};
        };

        # Set up an overlay from packages exposed by this flake
        overlayAttrs = config.packages;

        devShells = {
          default = pkgs.mkShell {
            inputsFrom = builtins.attrValues self'.packages;
            packages = with pkgs; [
              rustc
              gcc
              cargo
              clippy
              rustfmt

              graphene
              gobject-introspection
            ];

            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };

          nix = pkgs.mkShellNoCC {
            packages = with pkgs; [
              alejandra # formatter
              statix # linter
              deadnix # dead-code finder
            ];
          };
        };

        # provide the formatter for nix fmt
        formatter = pkgs.alejandra;
      };

      flake = {
        # homeManagerModules = {
        #   closer-layer = import ./nix/modules/home-manager.nix self;
        #   default = self.homeManagerModules.closer-layer;
        # };
      };
    };
}
