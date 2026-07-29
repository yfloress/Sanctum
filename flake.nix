{
  description = "Sanctum Web - Static site for GitHub Pages";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            nodejs_22
            pnpm
            typescript
            typescript-language-server
          ];

          shellHook = ''
            echo "> DEV SHELL ACTIVE"
            echo "   Node.js: $(node --version)"
            echo "   pnpm:    $(pnpm --version)"
            echo ""
            echo "   Commands:"
            echo "     pnpm install  - Install dependencies"
            echo "     pnpm dev      - Start dev server"
            echo "     pnpm build    - Build for production"
          '';
        };
      }
    );
}
