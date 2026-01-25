[![Nix Flake](https://img.shields.io/badge/Nix_Flake-Geared-dddd00?logo=nixos&logoColor=white)](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)

[![Nix](https://img.shields.io/badge/Nix-5277C3?style=flat&logo=nixos&logoColor=white)](https://nixos.org)

A terminal-based screensaver with old DVD style bouncing snowflake graphic.

![snowflake-bounce demo](https://raw.githubusercontent.com/saylesss88/snowflake-bounce/main/assets/demo.webm)

- Press `c` to change color

- Press `s` to change size of flake

- Press `f` for Easter Egg

- Press `q` to exit

## Installation

```bash
cargo install snowflake-bounce
```

**Nix**

```bash
nix run github:saylesss88/snowflake-bounce
```

**Flake Input**

```nix
inputs = {
  snowflake-bounce.url = "github:saylesss88/snowflake-bounce";
};
```

NixOS `systemPackages`:

```nix
{ inputs, pkgs, ... }: {
environment.systemPackages = [ inputs.snowflake-bounce.packages.${pkgs.stdenv.hostPlatform.system}.default ];
}
```

- To use `inputs` pass it through `specialArgs`
