{
  description = "Simple blog generator using your Git index";

  nixConfig = {
    extra-substituters = [ "https://so1ve.cachix.org" ];
    extra-trusted-public-keys = [
      "so1ve.cachix.org-1:51jcW4FkJhiLcqPsiUx3nglRP469les8F9zjFxio1nw="
    ];
  };

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      mkPackage =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          inherit (cargoToml.package) version;

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.lock
              ./Cargo.toml
              ./src
              ./templates
              ./theme
            ];
          };

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.makeWrapper ];

          postInstall = ''
            wrapProgram $out/bin/inkcairn \
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.git ]}
          '';

          meta = {
            inherit (cargoToml.package) description;
            homepage = cargoToml.package.repository;
            license = pkgs.lib.licenses.mit;
            mainProgram = cargoToml.package.name;
            platforms = pkgs.lib.platforms.linux;
          };
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          inkcairn = mkPackage pkgs;
        in
        {
          inherit inkcairn;
          default = inkcairn;
        }
      );

      apps = forAllSystems (
        system:
        let
          app = {
            type = "app";
            program = nixpkgs.lib.getExe self.packages.${system}.inkcairn;
            meta.description = cargoToml.package.description;
          };
        in
        {
          inkcairn = app;
          default = app;
        }
      );

      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt-tree);

      overlays.default = final: _prev: {
        inkcairn = mkPackage final;
      };
    };
}
