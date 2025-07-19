{ inputs, ... }:
{
  imports = [
  ];
  perSystem = { config, self', pkgs, lib, ... }:
    let
      toolchainForPkgs = p:
        p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolchainForPkgs;

      src = craneLib.cleanCargoSource ./.;

      # Common arguments can be set here to avoid repeating them later
      commonArgs = {
        inherit src;
        strictDeps = true;

        buildInputs =
          [
            # Add additional build inputs here
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

        # Additional environment variables can be set directly
        # MY_CUSTOM_VAR = "some value";
      };

      # Build *just* the cargo dependencies, so we can reuse
      # all of that work (e.g. via cachix) when running in CI
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      # Build the actual crate itself, reusing the dependency
      # artifacts from above.
      raytracing = craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts;
          doCheck = false;
        }
      );
    in
    {
      checks = {
        # Build the crate as part of `nix flake check` for convenience
        inherit raytracing;

        # Run clippy (and deny all warnings) on the crate source,
        # again, reusing the dependency artifacts from above.
        #
        # Note that this is done as a separate derivation so that
        # we can block the CI if there are issues here, but not
        # prevent downstream consumers from building our crate by itself.
        raytracing-clippy = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          }
        );

        # Run tests with cargo-nextest
        # Consider setting `doCheck = false` on `raytracing` if you do not want
        # the tests to run twice
        raytracing-nextest = craneLib.cargoNextest (
          commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
            cargoNextestPartitionsExtraArgs = "--no-tests=pass";
          }
        );
      };

      packages = {
        default = raytracing;
      };

      apps.default = {
        type = "app";
        program = "${raytracing}/bin/raytracing";
        meta.description = "raytracing";
      };

      devShells.rust = craneLib.devShell {
        # Inherit inputs from checks, excluding nextest to avoid git conflicts
        checks = {
          inherit (self'.checks) raytracing raytracing-clippy;
        };

        shellHook = ''
          # For rust-analyzer 'hover' tooltips to work.
          export RUST_SRC_PATH="${toolchainForPkgs pkgs}/lib/rustlib/src/rust/library";
        '';
      };
    };
}
