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

        in
        {
          devShells.default = pkgs.mkShell {
            RUST_BACKTRACE = "full";
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
            # Wayland and graphics libraries
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.vulkan-loader
              pkgs.libGL
              pkgs.xorg.libX11
              pkgs.xorg.libXcursor
              pkgs.xorg.libXrandr
              pkgs.xorg.libXi
            ];
            # Environment variables for graphics
            DISPLAY = ":0";
            WAYLAND_DISPLAY = "wayland-1";
            packages = [
              rust-toolchain
              pkgs.clippy
              pkgs.hyperfine
              pkgs.flamelens
              # Wayland and graphics support
              pkgs.wayland
              pkgs.wayland-protocols
              pkgs.wayland-scanner
              pkgs.libxkbcommon
              pkgs.vulkan-headers
              pkgs.vulkan-loader
              pkgs.libGL
              pkgs.pkg-config
              # X11 fallback support
              pkgs.xorg.libX11
              pkgs.xorg.libXcursor
              pkgs.xorg.libXrandr
              pkgs.xorg.libXi
              pkgs.xorg.libXinerama
            ];
          };
        };
    };
}
