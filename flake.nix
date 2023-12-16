/* File flake.nix
 *
 * Copyright (C) 2023 Riccardo Sacchetto <rsacchetto(at)nexxontech(dot)it>
 *
 * This file is part of GPT MD Translator.
 *
 * GPT MD Translator is free software: you can redistribute it and/or modify it under the terms of
 * the GNU General Public License as published by the Free Software Foundation,
 * either version 3 of the License, or (at your option) any later version.
 *
 * GPT MD Translator is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
 * more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * GPT MD Translator. If not, see <https://www.gnu.org/licenses/>. 
 */


{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, fenix, naersk, nixpkgs, ... }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; };
      gptmdt = let
        toolchain = (with fenix.packages."x86_64-linux";
          (combine [
            stable.cargo
            stable.rustc
            targets.x86_64-unknown-linux-musl.stable.rust-std
          ]));
      in
        (naersk.lib."x86_64-linux".override {
          cargo = toolchain;
          rustc = toolchain;
        }).buildPackage {
          pname = "gpt_md_translator";
          version = "0.1.0";

          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          src = ./.;
        };
    in {
      packages."x86_64-linux".default = gptmdt;
      devShells."x86_64-linux".default = pkgs.mkShell {
        name = "gpt_md_translator_devenv";
        packages = (with pkgs; [
          gdb
          pwndbg
          rust-analyzer
        ]) ++ (with fenix.packages."x86_64-linux";
          [ (combine [
            stable.cargo
            stable.rustc
            targets.x86_64-unknown-linux-musl.stable.rust-std
          ]) ]
        );
      };
    };
}
