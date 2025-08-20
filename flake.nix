{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    crane.url = "github:ipetkov/crane";

    git-hooks-nix.url = "github:cachix/git-hooks.nix";
    git-hooks-nix.inputs.nixpkgs.follows = "nixpkgs";

    devshell.url = "github:numtide/devshell";
    devshell.inputs.nixpkgs.follows = "nixpkgs";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.devshell.flakeModule
        inputs.git-hooks-nix.flakeModule
        ./rust.nix
      ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];

      perSystem =
        { config
        , self'
        , inputs'
        , pkgs
        , system
        , ...
        }:
        {
          # Per-system attributes can be defined here. The self' and inputs'
          # module parameters provide easy access to attributes of the same
          # system.
          imports = [
            "${inputs.nixpkgs}/nixos/modules/misc/nixpkgs.nix"
          ];
          nixpkgs.hostPlatform = system;
          nixpkgs.overlays = [ (import inputs.rust-overlay) ];

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixpkgs-fmt.enable = true;

              rustfmt.enable = true;
              rustfmt.package = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            };
          };

          pre-commit.settings = {
            hooks = {
              treefmt.enable = true;
            };
          };

          devshells.default = {
            name = "raytracing-shell";

            env = [
            ];

            commands =
              let
                rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

                run-example = pkgs.writeShellApplication {
                  name = "run-example";
                  runtimeInputs = [ rustToolchain ];
                  text = ''
                    if [[ $# -ne 1 ]]; then
                        echo "Usage: $0 <example-name>" >&2
                        exit 1
                    fi

                    # Move previous output if it exists
                    if [[ -f out.ppm ]]; then
                        mv out.ppm prev_out.ppm
                    fi

                    # Run the example and capture output
                    cargo run --example "$1" --release > out.tmp && mv out.tmp out.ppm
                  '';
                };

                run-example-completions = pkgs.stdenv.mkDerivation {
                  name = "run-example-completions";
                  phases = [ "installPhase" ];
                  nativeBuildInputs = [ pkgs.installShellFiles ];
                  buildInputs = [
                    rustToolchain
                    pkgs.jq
                  ];
                  installPhase = ''
                    installShellCompletion --cmd run-example \
                      --fish <(cat <<'EOF'
                    complete -c run-example -f -a '(
                        if command -q cargo; and command -q jq
                            cargo metadata --format-version 1 2>/dev/null | jq -r ".packages[] | select(.name == \"raytracing\") | .targets[] | select(.kind[] == \"example\") | .name" 2>/dev/null
                        end
                    )'
                    EOF
                    )
                  '';
                };

                run-example-combined = pkgs.symlinkJoin {
                  inherit (run-example) name;
                  paths = [
                    run-example
                    run-example-completions
                  ];
                };
              in
              [
                {
                  name = "watch-img";
                  help = "viu frontend that watches an image for changes";
                  package = pkgs.writeShellApplication {
                    name = "watch-img";
                    runtimeInputs = [
                      pkgs.viu
                      pkgs.fswatch
                    ];
                    text = ''
                      # Check if file path is provided
                      if [[ $# -ne 1 ]]; then
                          echo "Usage: $0 <path-to-img-file>" >&2
                          exit 1
                      fi

                      IMG_FILE="$1"

                      clear
                      viu "$IMG_FILE"

                      # Watch for changes and re-render
                      # for some reason, fsevents_monitor is really slow on my macbook,
                      # so we use poll_monitor instead.
                      fswatch -m poll_monitor -o "$IMG_FILE" | while read -r; do
                          clear
                          viu "$IMG_FILE"
                      done
                    '';
                  };
                }
                {
                  name = "runex";
                  help = "Run a cargo example with output redirection and backup";
                  package = run-example-combined;
                }
              ];

            devshell = {
              packagesFrom = [
                self'.devShells.rust
                config.treefmt.build.devShell
              ];

              packages = [
                pkgs.nixd
              ];

              startup.pre-commit.text = config.pre-commit.installationScript;
            };
          };

        };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.

      };
    };
}
