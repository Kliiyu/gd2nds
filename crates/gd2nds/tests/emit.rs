use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn emit_only_writes_a_blocksds_build_dir() {
    let out = std::env::temp_dir().join(format!("gd2nds-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);

    let status = Command::new(env!("CARGO_BIN_EXE_gd2nds"))
        .arg("build")
        .arg(repo_root().join("tests/projects/hello"))
        .arg("--out")
        .arg(&out)
        .arg("--emit-only")
        .status()
        .expect("run gd2nds");
    assert!(status.success());

    for file in [
        "Makefile",
        "source/main.c",
        "source/gd2nds_project.h",
        "runtime/include/gdds/gdds.h",
        "runtime/src/gdds.c",
    ] {
        assert!(out.join(file).is_file(), "missing {file}");
    }

    let makefile = std::fs::read_to_string(out.join("Makefile")).unwrap();
    assert!(makefile.contains("NAME\t\t:= hello_ds"));
    assert!(makefile.contains("GAME_TITLE\t:= Hello DS"));
    assert!(!makefile.contains('@'), "unreplaced template variable");

    let header = std::fs::read_to_string(out.join("source/gd2nds_project.h")).unwrap();
    assert!(header.contains("#define GD2NDS_PROJECT_NAME \"Hello DS\""));

    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn rejects_a_folder_without_project_godot() {
    let output = Command::new(env!("CARGO_BIN_EXE_gd2nds"))
        .args(["build", "--emit-only"])
        .arg(repo_root().join("templates"))
        .output()
        .expect("run gd2nds");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("project.godot"), "{stderr}");
}
