{
  description = "IRIS – Intelligent Runtime for Iterative Synthesis";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          iris = pkgs.rustPlatform.buildRustPackage {
            pname = "iris";
            version = "0.1.0";

            src =
              let
                fs = pkgs.lib.fileset;
              in
              fs.toSource {
                root = ./.;
                fileset = fs.unions [
                  ./Cargo.toml
                  ./Cargo.lock
                  ./src
                  ./iris-clcu
                  ./benches
                  ./tests
                  ./examples
                ];
              };

            cargoLock.lockFile = ./Cargo.lock;

            # Evolution tests are extremely slow; run them explicitly via cargo.
            doCheck = false;

            meta = {
              description = "IRIS – a self-improving programming language where programs are typed DAGs that evolve, verify, and optimize themselves";
              homepage = "https://github.com/boj/iris";
              license = pkgs.lib.licenses.agpl3Plus;
              mainProgram = "iris";
            };
          };
        in
        {
          inherit iris;
          default = iris;
        }
      );

      overlays.default = _final: prev: {
        iris = self.packages.${prev.system}.iris;
      };

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.iris ];
            packages = with pkgs; [
              rust-analyzer
              clippy
              cargo-watch
            ];
          };
        }
      );
    };
}
