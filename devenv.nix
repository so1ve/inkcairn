{
  pkgs,
  ...
}:

{
  languages.nix = {
    enable = true;
    lsp.enable = true;
  };

  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };

  packages = with pkgs; [
    tombi
    yaml-language-server
  ];
}
