{ pkgs, ... }:

pkgs.mkShell { buildInputs = with pkgs; [ stdenv postgresql httpie ]; }
