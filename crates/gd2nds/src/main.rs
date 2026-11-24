//! gd2nds: compile a Godot 4 project into a Nintendo DS ROM.
//!
//! Milestone 1 ("Hello ROM") proves the toolchain end to end: it reads
//! `project.godot`, writes a BlocksDS build directory with the C runtime and a
//! test `main.c`, and runs `make` to produce a `.nds` file.

mod emit;
mod project;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use project::{ProjectConfig, TARGET_GODOT_VERSION};

const USAGE: &str = "\
gd2nds - compile a Godot 4 project into a Nintendo DS ROM

Usage:
  gd2nds build <project_dir> [--out <dir>] [--emit-only]
  gd2nds --version
  gd2nds --help

Options:
  --out <dir>    Build directory (default: <project_dir>/.gd2nds/build)
  --emit-only    Write the generated sources and stop before running make

Environment:
  BLOCKSDS       Path to BlocksDS core (default: /opt/blocksds/core)
";

struct BuildArgs {
    project_dir: PathBuf,
    out_dir: Option<PathBuf>,
    emit_only: bool,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("build") => match parse_build_args(&args[1..]) {
            Ok(build_args) => run_build(build_args),
            Err(message) => {
                eprintln!("error: {message}\n\n{USAGE}");
                ExitCode::from(2)
            }
        },
        Some("--version" | "-V") => {
            println!("gd2nds {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") | None => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("error: unknown command {other:?}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn parse_build_args(args: &[String]) -> Result<BuildArgs, String> {
    let mut project_dir = None;
    let mut out_dir = None;
    let mut emit_only = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--emit-only" => emit_only = true,
            "--out" => {
                let dir = iter.next().ok_or("--out needs a directory")?;
                out_dir = Some(PathBuf::from(dir));
            }
            flag if flag.starts_with('-') => return Err(format!("unknown option {flag:?}")),
            path => {
                if project_dir.replace(PathBuf::from(path)).is_some() {
                    return Err("only one project directory can be given".into());
                }
            }
        }
    }
    Ok(BuildArgs {
        project_dir: project_dir.ok_or("missing <project_dir>")?,
        out_dir,
        emit_only,
    })
}

fn run_build(args: BuildArgs) -> ExitCode {
    let config = match ProjectConfig::load(&args.project_dir) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("error: {}: {e}", args.project_dir.display());
            return ExitCode::FAILURE;
        }
    };

    match config.godot_version() {
        Some(v) if v == TARGET_GODOT_VERSION => {}
        Some(v) => eprintln!(
            "warning: project was saved with Godot {v}; gd2nds targets Godot {TARGET_GODOT_VERSION}"
        ),
        None => eprintln!("warning: project.godot does not record a Godot version"),
    }

    let build_dir = args
        .out_dir
        .unwrap_or_else(|| args.project_dir.join(".gd2nds").join("build"));
    let output = match emit::emit(&config, &build_dir) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("error: could not write {}: {e}", build_dir.display());
            return ExitCode::FAILURE;
        }
    };
    println!(
        "gd2nds: \"{}\" -> {}",
        config.name(),
        output.build_dir.display()
    );

    if args.emit_only {
        println!("gd2nds: sources written; run `make` in the build directory to get the ROM");
        return ExitCode::SUCCESS;
    }

    run_make(&output.build_dir, &output.rom_name)
}

fn run_make(build_dir: &Path, rom_name: &str) -> ExitCode {
    let blocksds = std::env::var_os("BLOCKSDS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/opt/blocksds/core"));
    if !blocksds.join("sys").is_dir() {
        eprintln!(
            "error: BlocksDS not found at {}.\n\
             Install BlocksDS and set BLOCKSDS, or build inside the BlocksDS Docker image:\n  \
             docker run --rm -v \"$PWD:/work\" -w /work/{} skylyrac/blocksds:slim-latest make",
            blocksds.display(),
            build_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let status = Command::new("make")
        .current_dir(build_dir)
        .env("BLOCKSDS", &blocksds)
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("gd2nds: built {}", build_dir.join(rom_name).display());
            ExitCode::SUCCESS
        }
        Ok(s) => {
            eprintln!("error: make failed ({s})");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("error: could not run make: {e}");
            ExitCode::FAILURE
        }
    }
}
