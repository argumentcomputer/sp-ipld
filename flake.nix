{
  inputs = {
    utils.url = "github:yatima-inc/nix-utils";
  };

  outputs =
    { self
    , utils
    }:
    utils.inputs.flake-utils.lib.eachDefaultSystem (system:
    let
      lib = utils.lib.${system};
      pkgs = utils.nixpkgs.${system};
      inherit (lib) buildRustProject testRustProject rustDefault filterRustProject;
      rust = rustDefault;
      crateName = "sp-ipld";
      root = ./.;
      buildInputs = with pkgs; [
        pkg-config
        openssl
      ];
    in
    {
      packages.${crateName} = buildRustProject { inherit root buildInputs; };

      checks.${crateName} = testRustProject { doCheck = true; inherit root buildInputs; cargoTestOptions = options: options ++ [ "--all-features" ]; };

      defaultPackage = self.packages.${system}.${crateName};

      # `nix develop`
      devShell = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.packages.${system};
        nativeBuildInputs = [ rust ];
        buildInputs = with pkgs; [
          rust-analyzer
          clippy
          rustfmt
        ];
      };
    });
}
