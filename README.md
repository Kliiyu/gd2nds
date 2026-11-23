# gd2nds

Compile a Godot 4 project into a Nintendo DS `.nds` ROM.

gd2nds is an ahead-of-time compiler: GDScript is translated to C, scenes and
assets are converted to DS-native data, and everything is linked against a
small C runtime (`libgdds`) with [BlocksDS](https://github.com/blocksds/sdk).
There is no Godot engine or script interpreter on the DS.

Target: **Godot 4.7**, 2D scenes first.

## Status

Milestone 1, "Hello ROM": the CLI reads `project.godot`, writes a BlocksDS
build directory and builds a ROM that shows a sprite you can move with the
D-pad, plus the project name on the bottom screen. Scenes and GDScript are not
compiled yet. See the architecture plan for the milestones that follow.

## Build gd2nds

```sh
cargo build --release
```

## Build a ROM

With BlocksDS installed (`BLOCKSDS` defaults to `/opt/blocksds/core`):

```sh
gd2nds build path/to/godot_project
# -> path/to/godot_project/.gd2nds/build/<name>.nds
```

Without a local BlocksDS, generate the sources and build in Docker:

```sh
gd2nds build tests/projects/hello --out build/hello --emit-only
docker run --rm -v "$PWD:/work" -w /work/build/hello skylyrac/blocksds:slim-latest make
```

Run the ROM in [melonDS](https://melonds.kuribo64.net/) or on hardware with a
flashcart.

## Layout

| Path | What it is |
| --- | --- |
| `crates/gd2nds` | The CLI and compiler (Rust) |
| `runtime/` | `libgdds`, the C runtime linked into every ROM |
| `templates/rom/` | BlocksDS Makefile and the milestone 1 `main.c` |
| `tests/projects/` | Godot test projects |

## License

MIT
