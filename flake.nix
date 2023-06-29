{
  description = "Breadth-first exhaustive `zip` for repeatable iterators."
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };
  outputs = { flake-utils, naersk, nixpkgs, self }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = (import nixpkgs) { inherit system; };
      naersk' = pkgs.callPackage naersk {};
    in {
      defaultPackage = naersk'.buildPackage { src = ./.; };
      devShell = pkgs.mkShell { nativeBuildInputs = with pkgs; [ rustc cargo ]; };
    }
  );
}
