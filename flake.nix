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
      # Should be kept in sync with those workflows.
      rust = pkgs.rust-bin.stable."1.97.1".default.override {
        extensions = [ "rust-src" "rust-analyzer" "rustfmt" "clippy" ];
      };

      mkExtractorCommand = name: args: pkgs.writeShellApplication {
        inherit name;
        runtimeInputs = with pkgs; [ git jdk25 ];
        text = ''
          repo_root="$(git rev-parse --show-toplevel)"
          export JAVA_HOME="${pkgs.jdk25}"
          exec "$repo_root/extractor/gradlew" -p "$repo_root/extractor" ${pkgs.lib.escapeShellArgs args} "$@"
        '';
      };

      fmt-java = mkExtractorCommand "fmt-java" [ "spotlessApply" ];

      lint-java-pmd = mkExtractorCommand "lint-java-pmd" [ "lintPmd" ];
      lint-java-error-prone = mkExtractorCommand "lint-java-error-prone" [ "lintErrorProne" ];
      lint-java-all = mkExtractorCommand "lint-java-all" [ "lintPmd" "lintErrorProne" "--continue" ];

      fmt-rust = pkgs.writeShellApplication {
        name = "fmt-rust";
        runtimeInputs = with pkgs; [ git rust ];
        text = ''
          repo_root="$(git rev-parse --show-toplevel)"
          exec ${rust}/bin/cargo fmt --manifest-path "$repo_root/Cargo.toml" --all "$@"
        '';
      };

      appNativeBuildInputs = with pkgs; [
          # required for the packet inspector on nix
          pkg-config
          fmt-java
          fmt-rust
          lint-java-pmd
          lint-java-error-prone
          lint-java-all
      ];
      appBuildInputs = with pkgs; [
          rust
          # dependencies for the packet inspector
          udev alsa-lib vulkan-loader wayland
          libX11 libXcursor libXi libXrandr
          libxkbcommon wayland
          # Java: required for the extractor
          jdk25 jdt-language-server
      ];
    in
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
