{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ flake-parts, fenix, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.nci.flakeModule
      ];
      systems = [ "x86_64-linux" ];

      perSystem = { config, self', inputs', lib, pkgs, system, ... }:
        let
          crateName = "simple-binary";
          crateOutputs = config.nci.outputs.${crateName};

          project = crateName;
          binary = crateName;

          fenixStable = fenix.packages.${system}.stable;
          rustToolchain = fenixStable.withComponents [
            "rustc"
            "cargo"
            "clippy"
            "rust-src"
            "rust-docs"
            "rust-analyzer"
            "llvm-tools-preview"
          ];

          stdenv = {
            # in this case we will set it to the clang stdenv
            override = old: { stdenv = pkgs.clangStdenv; };
          };

          # Use mold for linking.
          moldLinking = {
            flags = "-C linker=${lib.getExe pkgs.clang} -C link-arg=-fuse-ld=${lib.getExe pkgs.mold}";
            nativeBuildInputs = [
              pkgs.clang
              pkgs.mold
            ];
          };

          commonBuildDeps = [
            # Add build dependencies here.
          ];

          commonNativeBuildDeps = [
            # Add native build dependencies here.
          ];

          runtimeDeps = [
            # Add runtime dependencies here.
          ];

          # Inputs for building the dependencies of the crate. (Dependencies listed in Cargo.toml & Cargo.lock)
          crateDepsInputOverrides = old: {
            buildInputs = (old.buildInputs or [ ]) ++ commonBuildDeps ++ [
              # Add dependency specific build dependencies here.
            ];
            nativeBuildInputs = (old.nativeBuildInputs or [ ])
              ++ commonNativeBuildDeps
              ++ moldLinking.nativeBuildInputs
              ++ [
              # Add dependency specific native build dependencies here.
            ];
          };

          # Inputs for building the crate itself.
          crateInputOverrides = old: {
            buildInputs = (old.buildInputs or [ ]) ++ commonBuildDeps ++ [
              # Add crate specific build dependencies here.
            ];
            nativeBuildInputs = (old.nativeBuildInputs or [ ])
              ++ commonNativeBuildDeps
              ++ moldLinking.nativeBuildInputs
              ++ [
              # Add crate specific native build dependencies here.
            ];
          };
        in
        {
          # Per-system attributes can be defined here. The self' and inputs'
          # module parameters provide easy access to attributes of the same
          # system.

          nci.toolchains.build = rustToolchain;

          # Projectwise settings.
          nci.projects.${project} = {
            relPath = "";
            runtimeLibs = runtimeDeps;
            depsOverrides = {
              inherit stdenv;
              add-env.RUSTFLAGS = moldLinking.flags;
              add-inputs.overrideAttrs = crateDepsInputOverrides;
            };
            overrides = {
              inherit stdenv;
              add-env.RUSTFLAGS = moldLinking.flags;
              add-inputs.overrideAttrs = crateInputOverrides;
            };
          };

          # Crate settings.
          # If you need crate-level input overrides, add to depsOverrides & overrides of the crate.
          # Note that if you need to override RUSTFLAGS, remember to add moldLinking.flags
          # otherwise the mold linking will not work.
          nci.crates.${crateName}.export = true;

          # Equivalent to  inputs'.nixpkgs.legacyPackages.hello;
          packages.default = crateOutputs.packages.release;

          apps.default = {
            program = "${config.packages.default}/bin/${binary}";
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = [ crateOutputs.devShell ];
            packages = with pkgs; [
              git
            ];
          };
        };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.
      };
    };
}
