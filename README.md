# gd2nds

Compile a Godot 4 project into a Nintendo DS `.nds` ROM.

gd2nds is an ahead-of-time compiler. GDScript is translated to C, scenes and
assets are converted to DS-native data, and everything is linked against a
small C runtime (`libgdds`) with [BlocksDS](https://github.com/blocksds/sdk).
There is no Godot engine and no script interpreter on the DS, so games run as
native ARM9 code.

- **Target:** Godot 4.7, 2D scenes first
- **Host compiler:** Rust
- **DS SDK:** BlocksDS

📖 **Documentation:** <https://kliiyu.github.io/gd2nds-docs/>

## Status

gd2nds is in early development and cannot compile real games yet.

| Milestone | What it delivers | State |
| --- | --- | --- |
| 1. Hello ROM | CLI reads `project.godot`, builds a ROM with a test sprite | Done, runs in melonDS |
| 2. Scenes and assets | `.tscn` parsing, sprites and `SpriteFrames` shown at editor positions | Planned |
| 3. GDScript core | Typed GDScript compiled to C: `_ready`, `_process`, `Input`, `position` | Planned |
| 4. Tilemaps and camera | `TileMapLayer` backgrounds, `Camera2D` scrolling, z-order | Planned |
| 5. Hardening | Error messages with file and line, emulator tests, VRAM report | Planned |

Today the ROM shows a sprite you can move with the D-pad on the top screen and
the project name on the bottom screen. Scenes and scripts are not compiled yet.

## Requirements

- [Rust](https://rustup.rs/) (stable) to build gd2nds
- [BlocksDS](https://blocksds.skylyrac.net/) installed locally, **or** Docker
  to use the `skylyrac/blocksds` image
- An emulator such as [melonDS](https://melonds.kuribo64.net/), or a DS with a
  flashcart, to run the ROM

## Build gd2nds

```sh
git clone https://github.com/Kliiyu/gd2nds
cd gd2nds
cargo build --release
# the binary is target/release/gd2nds
```

## Usage

```text
gd2nds build <project_dir> [--out <dir>] [--emit-only]

  --out <dir>    Build directory (default: <project_dir>/.gd2nds/build)
  --emit-only    Write the generated sources and stop before running make

Environment:
  BLOCKSDS       Path to BlocksDS core (default: /opt/blocksds/core)
```

### With BlocksDS installed

```sh
gd2nds build path/to/godot_project
# -> path/to/godot_project/.gd2nds/build/<name>.nds
```

### With Docker

```sh
gd2nds build tests/projects/hello --out build/hello --emit-only
docker run --rm -v "$PWD:/work" -w /work/build/hello skylyrac/blocksds:slim-latest make
# -> build/hello/hello_ds.nds
```

Every pull request also builds the `tests/projects/hello` ROM in CI and
uploads it as the `hello-ds-rom` artifact.

## DS limits to keep in mind

The DS has a 67 MHz ARM9 with no FPU, 4 MB of RAM, 656 KB of VRAM and 128
sprites per screen. gd2nds will check these limits at build time and fail
with the assets responsible, rather than producing a ROM that misbehaves.
See the documentation for details.

## Repository layout

| Path | What it is |
| --- | --- |
| `crates/gd2nds` | The CLI and compiler (Rust) |
| `runtime/` | `libgdds`, the C runtime linked into every ROM |
| `templates/rom/` | BlocksDS Makefile and the milestone 1 `main.c` |
| `tests/projects/` | Godot test projects |
| `.github/workflows/` | CI: `cargo fmt`, `clippy`, tests and a ROM build |

## Contributing

Bug reports and feature requests are welcome in
[Issues](https://github.com/Kliiyu/gd2nds/issues). Before opening a pull
request, run:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

[MIT](LICENSE)
