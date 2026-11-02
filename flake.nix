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

      rust-miri = pkgs.rust-bin.nightly."2026-06-26".default.override {
        extensions = [ "rust-src" "miri" ];
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

      mkRepoCommand = name: inputs: command: pkgs.writeShellApplication {
        inherit name;
        runtimeInputs = [ pkgs.git pkgs.coreutils ] ++ inputs;
        text = ''
          repo_root="$(git rev-parse --show-toplevel)"
          cd "$repo_root"
          ${command}
        '';
      };

      preparePlayground = ''
        if [ ! -e tools/playground/src/playground.rs ]; then
          cp tools/playground/src/playground.template.rs tools/playground/src/playground.rs
        fi
      '';

      mkRustCommandWith = toolchain: name: args: mkRepoCommand name [ toolchain ] ''
        ${preparePlayground}
        exec ${toolchain}/bin/cargo ${pkgs.lib.escapeShellArgs args} "$@"
      '';
      mkRustCommand = mkRustCommandWith rust;

      # Run every check and fail if any check fails, like independent CI jobs.
      mkCheckCommand = name: checks: pkgs.writeShellApplication {
        inherit name;
        runtimeInputs = checks;
        text = ''
          status=0
          ${pkgs.lib.concatMapStringsSep "\n" (check: ''
            echo "==> ${check.name}"
            ${pkgs.lib.getExe check} "$@" || status=1
          '') checks}
          exit "$status"
        '';
      };

      fmt-java = mkExtractorCommand "fmt-java" [ "spotlessApply" ];
      lint-java-pmd = mkExtractorCommand "lint-java-pmd" [ "lintPmd" ];
      lint-java-error-prone = mkExtractorCommand "lint-java-error-prone" [ "lintErrorProne" ];
      lint-java-all = mkExtractorCommand "lint-java-all" [ "lintPmd" "lintErrorProne" "--continue" ];
      javaBasicChecks = [ "spotlessCheck" "lintPmd" "lintErrorProne" ];
      ci-java-basic = mkExtractorCommand "ci-java-basic" (javaBasicChecks ++ [ "--continue" ]);
      ci-java-full = mkExtractorCommand "ci-java-full" (javaBasicChecks ++ [ "build" "--continue" ]);

      fmt-rust = mkRustCommand "fmt-rust" [ "fmt" "--all" ];
      lint-rust-clippy = mkRustCommand "lint-rust-clippy" [
        "clippy" "--workspace" "--no-deps" "--all-features" "--all-targets" "--" "-D" "warnings"
      ];
      check-fmt-rust = mkRustCommand "check-fmt-rust" [ "fmt" "--all" "--" "--check" ];
      test-rust-basic = mkRustCommand "test-rust-basic" [ "test" "--workspace" "--all-features" "--all-targets" ];
      test-rust-docs = mkRustCommand "test-rust-docs" [ "test" "--workspace" "--all-features" "--doc" ];
      test-rust-nbt = mkRustCommand "test-rust-nbt" [ "test" "-p" "chunkedge_nbt" "--all-targets" ];
      test-rust-miri = mkRustCommandWith rust-miri "test-rust-miri" [ "miri" "test" "--workspace" "--all-features" "--doc" ];
      test-rust-full = mkCheckCommand "test-rust-full" [
        test-rust-basic test-rust-docs test-rust-nbt test-rust-miri
      ];

      check-rust-docs = mkRepoCommand "check-rust-docs" [ rust ] ''
        ${preparePlayground}
        export RUSTDOCFLAGS="-D warnings"
        exec ${rust}/bin/cargo doc --workspace --no-deps --all-features --document-private-items "$@"
      '';
      check-rust-unused-deps = mkRepoCommand "check-rust-unused-deps" [ rust pkgs.cargo-machete ] ''
        exec cargo machete --with-metadata "$@"
      '';
      mkDepgraphCommand = name: action: mkRepoCommand name [ rust pkgs.cargo-depgraph pkgs.graphviz pkgs.diffutils ] ''
        generated="$(mktemp)"
        trap 'rm -f "$generated"' EXIT
        cargo depgraph --workspace-only "$@" | tred | dot -Tsvg > "$generated"
        ${action}
      '';
      check-rust-depgraph = mkDepgraphCommand "check-rust-depgraph" ''
        diff -u assets/depgraph.svg "$generated"
      '';
      generate-depgraph = mkDepgraphCommand "generate-depgraph" ''
        cat "$generated" > assets/depgraph.svg
      '';
      check-typos = mkRepoCommand "check-typos" [ pkgs.typos ] ''
        exec typos "$@"
      '';
      ci-rust-full = mkCheckCommand "ci-rust-full" [
        check-fmt-rust test-rust-full lint-rust-clippy check-rust-docs
        check-rust-unused-deps check-rust-depgraph
      ];

      ci-full = mkCheckCommand "ci-full" [ ci-rust-full ci-java-full check-typos ];

      appNativeBuildInputs = with pkgs; [
          # required for the packet inspector on nix
          pkg-config

          # Custom commands for the repo
          fmt-java
          fmt-rust
          lint-java-pmd
          lint-java-error-prone
          lint-java-all
          ci-java-basic
          ci-java-full
          lint-rust-clippy
          test-rust-basic
          test-rust-full
          ci-rust-full
          check-typos
          generate-depgraph
          ci-full
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
        devShells.default = pkgs.mkShell {
            nativeBuildInputs = appNativeBuildInputs;
            buildInputs = appBuildInputs;
            shellHook = ''
                export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath appBuildInputs}"
                export JAVA_HOME="${pkgs.jdk25}"
            '';
        };
    });
}
