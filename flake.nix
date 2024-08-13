{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    poetry2nix = {
      url = "github:nix-community/poetry2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, poetry2nix }:
    let
      overlays = [
        rust-overlay.overlays.default
        (final: prev: {
          rustToolchain =
            let
              rust = prev.rust-bin;
            in
            if builtins.pathExists ./rust-toolchain.toml then
              rust.fromRustupToolchainFile ./rust-toolchain.toml
            else if builtins.pathExists ./rust-toolchain then
              rust.fromRustupToolchainFile ./rust-toolchain
            else
              rust.stable.latest.default.override {
                extensions = [ "rust-src" "rustfmt" ];
              };
        })
        poetry2nix.overlays.default
      ];
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system:
      f {
        pkgs = import nixpkgs {
          inherit overlays system;
          };
        # poetryEnv = nixpkgs.poetry2nix.mkPoetryEnv {
        #   projectDir = ./.;
        # };
      }
      );
    in
    {
      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShellNoCC {
          buildInputs = [pkgs.rustToolchain ];
          packages = with pkgs; [
            # poetryEnv
            poetry
            pre-commit
            openssl
            pkg-config
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer
            avrdude
            # Required for error ld: framework not found SystemConfiguration
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          shellHook = ''
            pre-commit install --install-hooks
          '';
        };
      });
    };
}
