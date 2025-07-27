{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      perSystem =
        {
          pkgs,
          ...
        }:
        let
          rust-toolchain = pkgs.symlinkJoin {
            name = "rust-toolchain";
            paths = with pkgs; [
              rustc
              rustfmt
              cargo
              cargo-watch
              rust-analyzer
              rustPlatform.rustcSrc
              cargo-dist
              cargo-tarpaulin
              cargo-insta
              cargo-machete
              cargo-edit
              cargo-flamegraph
            ];
          };
          buildInputs = with pkgs; [
            # Wayland and graphics support
            wayland
            wayland-protocols
            libxkbcommon
            vulkan-loader
            libGL
            # # X11 fallback support
            # xorg.libX11
            # xorg.libXcursor
            # xorg.libXrandr
            # xorg.libXi
            # xorg.libXinerama
          ];
        in
        {
          devShells.default = pkgs.mkShell {
            RUST_BACKTRACE = "full";
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
            packages = [
              rust-toolchain
              pkgs.clippy
              pkgs.hyperfine
              pkgs.flamelens
            ] ++ buildInputs;
          };
        };
    };
}
