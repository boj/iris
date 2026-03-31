{
  description = "IRIS -- Intelligent Runtime for Iterative Synthesis";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      # The pre-built iris-stage0 binary is x86-64 Linux ELF.
      packageSystems = [ "x86_64-linux" ];
      devShellSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forPackageSystems = nixpkgs.lib.genAttrs packageSystems;
      forDevShellSystems = nixpkgs.lib.genAttrs devShellSystems;
    in
    {
      packages = forPackageSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          iris = pkgs.stdenv.mkDerivation {
            pname = "iris";
            version = "0.1.0";

            src =
              let
                fs = pkgs.lib.fileset;
              in
              fs.toSource {
                root = ./.;
                fileset = fs.unions [
                  ./bootstrap
                  ./src/iris-programs
                  ./examples
                  ./benchmark
                ];
              };

            nativeBuildInputs = [ pkgs.makeWrapper pkgs.autoPatchelfHook ];
            buildInputs = [ pkgs.stdenv.cc.cc.lib ];

            dontBuild = true;
            dontConfigure = true;

            installPhase = ''
              runHook preInstall

              # Install bootstrap pipeline stages (pre-compiled JSON)
              install -d $out/share/iris/bootstrap
              install -Dm644 bootstrap/*.json $out/share/iris/bootstrap/

              # Install the stage0 binary, wrapped to find bootstrap dir
              install -Dm755 bootstrap/iris-stage0 $out/libexec/iris-stage0
              makeWrapper $out/libexec/iris-stage0 $out/bin/iris-stage0 \
                --set IRIS_BOOTSTRAP_DIR $out/share/iris/bootstrap

              # Install IRIS source programs
              cp -r src/iris-programs $out/share/iris/programs

              # Install examples and benchmarks
              cp -r examples  $out/share/iris/examples
              cp -r benchmark $out/share/iris/benchmark

              runHook postInstall
            '';

            meta = {
              description = "IRIS -- a self-improving programming language where programs are typed DAGs that evolve, verify, and optimize themselves";
              homepage = "https://github.com/boj/iris";
              license = pkgs.lib.licenses.agpl3Plus;
              mainProgram = "iris-stage0";
              platforms = [ "x86_64-linux" ];
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

      devShells = forDevShellSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              elan
              jq
            ];

            shellHook = ''
              echo "IRIS dev shell"
              echo "  iris-stage0: ./bootstrap/iris-stage0"
              echo "  Lean toolchain: elan (install via 'elan default leanprover/lean4:v4.28.0')"
            '';
          };
        }
      );
    };
}
