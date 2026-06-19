# 🦖 Dino Jump Watercolor

A watercolour-themed clone of the Chrome offline dino game, written in Rust
using the **Bevy** game engine.

Jump over cacti. Watch the sky scroll by. Try not to lose.

https://github.com/user-attachments/assets/d447fed1-22f4-48fb-82e7-d412ede4d497

---

## Quick Start

### Prerequisites

You'll need **Rust** installed. If you don't have it yet:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This project uses **Rust edition 2024**, which ships with Rust 1.85+.
After installing, make sure you're up to date:

```bash
rustup update
```

(Optional but recommended) Install a faster linker to speed up compile times
during development:

```bash
# macOS
brew install lld

# Ubuntu / Debian
sudo apt install lld

# Fedora
sudo dnf install lld
```

### Run the game

```bash
git clone <this-repo>
cd t-rex-run-3d

# Fast dev mode (dynamically linked — much quicker recompiles)
cargo run --features fast_dev
```

The window should open after a minute or two of compilation. Jump with
**Space** or **Up arrow**. Restart with **R** after you crash.

---

## Building

### Development

```bash
cargo build
```

Uses incremental compilation — fine for small changes but slower for the first
build. Use `--features fast_dev` with `run` or `build` to enable dynamic
linking, which makes subsequent rebuilds noticeably faster.

```bash
cargo build --features fast_dev
```

### Release (optimised runtime)

```bash
cargo build --release
```

This enables link-time optimisation (`lto = "thin"`), full-codegen units, and
strips debug symbols. The binary will be smaller and the game will run faster.
Compilation will be slower.

### Release-fast (balanced)

```bash
cargo build --profile release-fast
```

A middle ground. You keep most of the runtime performance without sacrificing
compile speed. No link-time optimisation, more codegen units, debug symbols
kept intact.

### Quick sanity check

```bash
cargo check
```

Checks your code for errors without producing a binary. Fastest feedback loop.

---

## Project Structure

```
src/
  main.rs              Entry point — creates the Bevy App and adds plugins.
  constants.rs         All magic numbers in one place.
  components.rs        Bevy Component definitions.
  resources.rs         Bevy Resource definitions (global game state).
  pixel_mask.rs        Alpha mask for pixel-perfect collision.
  collision.rs         Two-phase collision detection (AABB + pixel-perfect).
  plugins/
    mod.rs
    dino_plugin.rs     Player character, jumping, gravity.
    tree_plugin.rs     Obstacle spawning, scrolling, scoring.
    road_plugin.rs     Scrolling ground segments.
    sky_plugin.rs      Parallax background.
    hud_plugin.rs      Score / speed HUD overlay.
    game_plugin.rs     Collision, game-over, restart.
```

Each module does one thing. That way you can read them one at a time without
getting lost.

---

## License

MIT — do whatever you want with it.
