{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
  flake-utils.lib.eachSystem
    [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ]
    (system:
    let
      overlays = [ (import rust-overlay)  ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      rustVersion = pkgs.lib.strings.trim (builtins.readFile ./rust-toolchain);
      rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
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

      check-fmt-java = mkExtractorCommand "check-fmt-java" [ "spotlessCheck" ];
      build-java = mkExtractorCommand "build-java" [ "build" ];
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

      build-rust-docs = mkRepoCommand "build-rust-docs" [ rust ] ''
        ${preparePlayground}
        export RUSTDOCFLAGS="--cfg unstable_doc --html-in-header docs/rustdoc-header.html"
        exec ${rust}/bin/cargo doc --workspace --no-deps --all-features "$@"
      '';
      check-rust-docs = mkRepoCommand "check-rust-docs" [ rust ] ''
        ${preparePlayground}
        export RUSTDOCFLAGS="-D warnings"
        exec ${rust}/bin/cargo doc --workspace --no-deps --all-features --document-private-items "$@"
      '';
      check-rust-unused-deps = mkRepoCommand "check-rust-unused-deps" [ rust pkgs.cargo-machete ] ''
        exec cargo machete --with-metadata "$@"
      '';
      # SVG geometry depends on font metrics so to keep the output
      # consistent between Linux and macOS, we specify fonts explicitly.
      depgraphFonts = pkgs.makeFontsConf {
        fontDirectories = [ pkgs.dejavu_fonts.minimal ];
        impureFontDirectories = [];
        includes = [];
      };
      mkDepgraphCommand = name: action: mkRepoCommand name [ rust pkgs.cargo-depgraph pkgs.graphviz pkgs.diffutils ] ''
        export FONTCONFIG_FILE="${depgraphFonts}"
        export PANGOCAIRO_BACKEND=fc
        export LC_ALL=C
        generated="$(mktemp)"
        trap 'rm -f "$generated"' EXIT
        cargo depgraph --workspace-only "$@" | tred | dot -Tsvg \
          -Gfontname="DejaVu Sans" -Nfontname="DejaVu Sans" -Efontname="DejaVu Sans" > "$generated"
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

      # We keep closures grouped roughly by CI jobs so that each
      # CI job only installs the dependencies it needs. For example,
      # Java tooling does not need the Rust GUI dependencies, etc.
      # Finally, we expose a full shell that includes all dependencies
      # to be used for local development. This is the default shell.
      rustLibraries = with pkgs; [ openssl ]
        ++ lib.optionals stdenv.hostPlatform.isLinux [
          udev alsa-lib vulkan-loader wayland
          libX11 libXcursor libXi libXrandr libxkbcommon
        ]
        ++ lib.optionals stdenv.hostPlatform.isDarwin [ libiconv ];
      rustCommands = [
        fmt-rust check-fmt-rust lint-rust-clippy
        test-rust-basic test-rust-docs test-rust-nbt build-rust-docs check-rust-docs
      ];
      javaCommands = [
        fmt-java check-fmt-java build-java lint-java-pmd lint-java-error-prone
        lint-java-all ci-java-basic ci-java-full
      ];
      toolingCommands = [
        check-typos check-rust-unused-deps check-rust-depgraph generate-depgraph
      ];

      # Share native dependency configuration between stable Rust and Miri.
      mkRustShell = packages: pkgs.mkShell {
        inherit packages;
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = rustLibraries;
        shellHook = pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
          export LD_LIBRARY_PATH="''${LD_LIBRARY_PATH:+$LD_LIBRARY_PATH:}${pkgs.lib.makeLibraryPath rustLibraries}"
        '';
      };
      shells = {
        rust = mkRustShell ([ rust ] ++ rustCommands);
        miri = mkRustShell [ rust-miri test-rust-miri ];
        java = pkgs.mkShell {
          packages = [ pkgs.jdk25 ] ++ javaCommands;
          JAVA_HOME = "${pkgs.jdk25}";
        };
        tooling = pkgs.mkShell {
          packages = [ rust ] ++ toolingCommands;
        };
        full = pkgs.mkShell {
          inputsFrom = [ shells.rust shells.java shells.tooling ];
          # Nightly stays in its own shell so it cannot shadow stable Rust.
          packages = [
            pkgs.jdt-language-server test-rust-miri test-rust-full ci-rust-full ci-full
          ];
          JAVA_HOME = "${pkgs.jdk25}";
        };
      };
    in
    {
      devShells = shells // { default = shells.full; };
    });
}
