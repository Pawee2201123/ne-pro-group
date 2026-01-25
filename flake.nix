{
  description = "A Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Select the rust toolchain (stable, nightly, or specific version)
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # The Rust Toolchain
            rustToolchain

            # Common build tools often needed for Rust crates (e.g., openssl)
            pkg-config
            openssl
          ];

          # Environment variables often needed for Rust
          shellHook = ''
            echo "🦀 Rust environment loaded!"
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            zsh
          '';
        };
      }
    );
}
