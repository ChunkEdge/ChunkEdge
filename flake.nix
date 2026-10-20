{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
  flake-utils.lib.eachSystem
    [ "x86_64-linux" "aarch64-linux" ]
    (system:
    let
      overlays = [ (import rust-overlay)  ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      # Pinned to the same toolchain CI uses (see .github/workflows).
      # selectLatestNightlyWith is intentionally NOT used: a moving nightly
      # silently changes the toolchain under developers (edition2024 needs
      # Cargo >= 1.85; the workspace currently requires Rust 1.97.1).
      rust = pkgs.rust-bin.stable."1.97.1".default.override {
        extensions = [ "rust-src" "rust-analyzer" "rustfmt" "clippy" ];
      };

      appNativeBuildInputs = with pkgs; [
          # required for the packet inspector on nix
          pkg-config
      ];
      appBuildInputs = with pkgs; [
          rust
          # dependencies for the packet inspector
          udev alsa-lib vulkan-loader wayland
          libX11 libXcursor libXi libXrandr
          libxkbcommon wayland
          # Java: the extractor targets Minecraft 26.x, whose bytecode and
          # Gradle/Loom toolchain require JDK 25 to compile and run. It is
          # the only JDK on PATH (and JAVA_HOME) so `java`/`javac` are
          # unambiguous. The extractor uses its own Gradle wrapper, so no
          # system Gradle package is needed.
          jdk25 jdt-language-server
      ];
    in 
    rec
    {
        devShell = pkgs.mkShell {
            nativeBuildInputs = appNativeBuildInputs;
            buildInputs = appBuildInputs;    
            shellHook = ''
                export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath appBuildInputs}"
                export JAVA_HOME="${pkgs.jdk25}"
            '';
        };
    });
}
