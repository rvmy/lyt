{
  description = "OSDev Rust dev shell";

  inputs = {
    nixpkgs-unstable.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs-unstable,
      rust-overlay,
    }:
    let
      system = "x86_64-linux";

      pkgs-unstable = import nixpkgs-unstable {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      };
    in
    {
      devShells.${system}.default = pkgs-unstable.mkShell {
        packages = [
          pkgs-unstable.rust-analyzer
          pkgs-unstable.rust-bin.stable.latest.default
          pkgs-unstable.wayland
          #pkgs-unstable.xdg-desktop-portal
          #pkgs-unstable.xdg-desktop-portal-gtk
          # pkgs-unstable.pkgconfig
          # pkgs-unstable.wayland-protocols
          pkgs-unstable.libxkbcommon
          pkgs-unstable.libX11
          # pkgs-unstable.pkgconfig
          # pkgs-unstable.xorg.libX11
        ];

        shellHook = ''
          export DEV_SHELL=1
          export LD_LIBRARY_PATH=${
            pkgs-unstable.lib.makeLibraryPath [
              pkgs-unstable.libxkbcommon
              pkgs-unstable.libGL
              pkgs-unstable.wayland
            ]
          }
          echo "🚀 Dev shell loaded"
          exec fish
        '';
      };
    };
}
