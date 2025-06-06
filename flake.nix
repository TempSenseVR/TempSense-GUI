{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = {self, nixpkgs, ...}: let
    systems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = nixpkgs.lib.genAttrs systems;
    pkgsFor = system: import nixpkgs {
      inherit system;
    };
  in {
    packages = forAllSystems (system: let
        pkgs = pkgsFor system;
      in {
        tempsense = pkgs.rustPlatform.buildRustPackage rec {
          pname = "tempsense";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustc
            cargo
          ];

          buildInputs = with pkgs; [
            wayland
            udev openssl

            # GUI libs
            fontconfig
            libGL libxkbcommon

            # X11 libs
            xorg.libXi xorg.libX11
            xorg.libXcursor xorg.libXrandr
          ];

          # Force linking so stuff can be dlopen'd
          RUSTFLAGS = map (a: "-C link-arg=${a}") [
            "-Wl,--push-state,--no-as-needed"
            "-lwayland-client"
            "-lxkbcommon"
            "-lEGL"
            "-Wl,--pop-state"
          ];

          meta = with pkgs.lib; {
            license = licenses.mit;
            platforms = platforms.linux;
            homepage = "https://github.com/TempSenseVR/TempSense-GUI";
          };
        };
      }
    );

    devShells = forAllSystems (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.mkShell rec {
        buildInputs = with pkgs; [
          wayland
          pkg-config
          rustup mold
          udev openssl

          # GUI libs
          fontconfig
          libGL libxkbcommon

          # X11 libs
          xorg.libXi xorg.libX11
          xorg.libXcursor xorg.libXrandr
        ];

        shellHook = ''
          rustup default 1.86.0
          rustup component add rust-src rust-std
          rustup component add rust-docs rust-analyzer
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
          export RUSTFLAGS="$RUSTFLAGS -C linker=${pkgs.clang}/bin/clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold"
        '';
      };
    });
  };
}
