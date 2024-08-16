{
  description = "Devshell with all the dependencies needed to develop and build the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs-ruby.url = "github:bobvanderlinden/nixpkgs-ruby";
    nixpkgs-ruby.inputs.nixpkgs.follows = "nixpkgs";
  };


  outputs = { self, nixpkgs, nixpkgs-ruby }:
    let
      # Boilerplate function for generating attributes for all systems
      forAllSystems = function:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
          "aarch64-linux"
          "x86_64-darwin"
          "aarch64-darwin"
        ]
          (system:
            (function (import nixpkgs {
              inherit system;
            })) system);
    in
    {
      packages = forAllSystems (pkgs: system:
        let
          ruby = nixpkgs-ruby.lib.packageFromRubyVersionFile {
            file = ./.ruby-version;
            inherit system;
          };
          tools = [ ruby ] ++ (with pkgs; [
            nodejs_20
            yarn
            mprocs
          ]);
          gemDependencies = with pkgs; [
            zstd
            libxml2
            libxslt
            imagemagick
          ];
          root = builtins.getEnv "PWD";
        in
        {
          default = pkgs.mkShell {
            buildInputs = tools ++ gemDependencies;
            shellHook = ''
              # https://github.com/sass/sassc-ruby/issues/148#issuecomment-644450274
              bundle config build.sassc --disable-lto
              export BUNDLE_BUILD__SASSC="--disable-lto"

              export GEM_HOME="${root}/.bundle"
              export GEM_PATH="${root}/.bundle"
              export PATH="${root}/.bundle/bin:$PATH"
              export RUBY_YJIT_ENABLE=1
            '';
          };
        });
    };
}
