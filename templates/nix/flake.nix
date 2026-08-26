{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # {if:rust}
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
    # {endif:rust}
    # {if:claude}
    claude-code = {
      url = "github:sadjow/claude-code-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # {endif:claude}
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      # {if:rust}
      rust-overlay,
      naersk,
      # {endif:rust}
      # {if:claude}
      claude-code,
      # {endif:claude}
      ...
    }:
    # {if:rust}
    # ── System-agnostic outputs (modules) live out here ──
    {
      nixosModules.default = import ./nix/module.nix self;
      homeModules.default = import ./nix/hm-module.nix self;
    }
    # ── Then merge the per-system outputs onto it ──
    //
    # {endif:rust}
      flake-utils.lib.eachSystem
        [
          "x86_64-linux"
          "aarch64-linux"
        ]
        (
          system:
          let
            # {if:rust}
            overlays = [ (import rust-overlay) ];
            # {endif:rust}
            pkgs = import nixpkgs {
              inherit 
                system 
                # {if:rust}
                overlays
                # {endif:rust}
              ;
              config.allowUnfree = true;
            };

            # {if:rust}
            # ── Toolchain ─────────────────────────────────────────────
            rust = pkgs.rust-bin.nightly.latest.default;

            naersk' = pkgs.callPackage naersk {
              cargo = rust;
              rustc = rust;
            };

            # ── Build helper ──────────────────────────────────────────
            buildApp =
              { release }:
              let
                name = "{name}";
                desc = "{desc}";
              in
              pkgs.callPackage ./nix/package.nix {
                inherit
                  naersk'
                  release
                  name
                  desc
                  ;
                src = ./.;
              };
            # {endif:rust}

            # {if:claude}
            # ── Claude Settings ─────────────────────────────────────
            claude = claude-code.packages.${system}.default;
            # {endif:claude}

            # ── Tooling shared by the dev shell and CI ───────────────
            ciTools = with pkgs; [
              # {if:rust}
              rust
              # rust tooling
              cargo-nextest
              cargo-edit
              # {endif:rust}

              # {if:audit}
              cargo-audit
              # {endif:audit}
              # {if:machete}
              cargo-machete
              # {endif:machete}
              # {if:deny}
              cargo-deny
              # {endif:deny}
              # {if:typos}
              typos
              # {endif:typos}
              # {if:committed}
              committed
              # {endif:committed}
              # {if:cliff}
              git-cliff
              # {endif:cliff}
              # {if:taplo}
              taplo
              # {endif:taplo}
              # {if:editorconfig}
              editorconfig-checker
              # {endif:editorconfig}

              # nix tooling
              nixfmt
              statix
              deadnix

              # crate deps
            ];
          in
          {
            # {if:rust}
            # ── Packages ──────────────────────────────────────────────
            packages = rec {
              {name} = buildApp { release = true; };
              {name}-debug = buildApp { release = false; };
              default = {name};
            };

            # ── Checks (nix flake check) ─────────────────────────────
            checks.check = self.packages.${system}.{name}-debug;
            # {endif:rust}

            # ── Dev Shell (nix develop) ──────────────────────────────
            devShells.default = pkgs.mkShell {
              buildInputs =
                ciTools
                ++ (with pkgs; [
                  # {if:rust}
                  rust-analyzer
                  # {endif:rust}
                  # {if:justfile}
                  just
                  # {endif:justfile}
                  # {if:claude}
                  claude
                  # {endif:claude}
                  nodejs
                ]);
            };

            # ── CI Shell (nix develop .#ci) ──────────────────────────
            # Lean: just the toolchain + checks, no editor/claude/shellHook.
            devShells.ci = pkgs.mkShell {
              buildInputs = ciTools;
            };
          }
        );
}
