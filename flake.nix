{
  description = "HFT Sim Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };

          python = pkgs.python311;
          pyPkgs = python.pkgs;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              # Rust
              rustc
              cargo
              rustfmt
              clippy
              rust-analyzer

              # Python
              python
              pyPkgs.pip
              pyPkgs.setuptools
              pyPkgs.wheel
              pkgs.maturin # maturin builds the PyO3 extension and installs into the env

              pkg-config
            ];

            shellHook = ''
              echo "HFT Sim dev shell loaded"
              echo "Python version: $(python --version)"
              echo "Rust version: $(rustc --version)"
              echo "Cargo version: $(cargo --version)"
            '';
          };
        });
    };
}
