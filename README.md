[![Nix Flake](https://img.shields.io/badge/Nix_Flake-Geared-dddd00?logo=nixos&logoColor=white)](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)

[![Nix](https://img.shields.io/badge/Nix-5277C3?style=flat&logo=nixos&logoColor=white)](https://nixos.org)

A terminal-based screensaver with old DVD style bouncing snowflake graphic.
(NOTE: The colors in the demo are off)

![snowflake-bounce demo](https://raw.githubusercontent.com/saylesss88/snowflake-bounce/main/assets/demo.gif)

- Press `c` to change color

- Press `s` to change size of flake

- Press `f` for Easter Egg

- Press `q` to exit

---

## Features

- Pure Rust implementation using crossterm (no C dependencies)
- Cross-platform terminal support
- Smooth animations with configurable symbols
- Multiple color schemes
- Lightweight and fast

---

## Installation

```bash
cargo install snowflake-bounce
```

Version check:

```bash
snowflake-bounce -V
snowflake-bounce --version
```

**For Nix Users**

```bash
nix run github:saylesss88/snowflake-bounce

./result/bin/snowflake-bounce
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

---

## Building from Source

```bash
git clone https://github.com/saylesss88/snowflake-bounce
cd snowflake-bounce
cargo build --release
./target/release/snowflake-bounce
```

---

## License
