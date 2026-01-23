[![Flake](https://img.shields.io/badge/Nix-Flake-bright_purple?logo=nixos)](https://flakehub.com/f/saylesss88/snowflake-bounce)

A terminal-based screensaver with old DVD style bouncing snowflake graphic.

![snowflake-bounce](https://raw.githubusercontent.com/saylesss88/snowflake-bounce/main/assets/snowflake-bounce.png)

- Press `c` to change color

- Press `s` to change size of flake (WIP)

- Press `q` to exit

## Installation

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

This is a WIP
