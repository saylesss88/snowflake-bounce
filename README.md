[![Flake](https://img.shields.io/badge/Nix-Flake-bright_purple?logo=nixos)](https://flakehub.com/f/saylesss88/snowflake-bounce)

A terminal-based screensaver with old DVD style bouncing snowflake graphic.

![snowflake-bounce](https://raw.githubusercontent.com/saylesss88/snowflake-bounce/main/assets/snowflake-bounce.cleaned.png)

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
environment.systemPackages = [ inputs.snowflake-bounce.packages.${pkgs.system}.default ];
```
