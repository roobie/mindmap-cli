use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn integration_add_and_show() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    // create initial MINDMAP.md with one node
    let file = temp.child("MINDMAP.md");
    file.write_str("[1] **AE: Base** - base node\n")?;

    // run `mindmap add --type AE --title New --desc "refers [1]"` in temp dir
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.current_dir(temp.path())
        .arg("add")
        .arg("--type")
        .arg("AE")
        .arg("--title")
        .arg("New")
        .arg("--desc")
        .arg("refers [1]");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added node"));

    // show the new node (should be id 2)
    let mut cmd2 = Command::cargo_bin("mindmap-cli")?;
    cmd2.current_dir(temp.path()).arg("show").arg("2");
    cmd2.assert()
        .success()
        .stdout(predicate::str::contains("New"));

    temp.close()?;
    Ok(())
}

#[test]
fn integration_edit_flow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let file = temp.child("MINDMAP.md");
    file.write_str("[1] **AE: ToEdit** - original desc\n")?;

    // create an editor script that overwrites the file passed
    let editor = temp.child("editor.sh");
    editor
        .write_str("#!/bin/sh\ncat >\"$1\" <<'EOF'\n[1] **AE: Edited** - edited desc [1]\nEOF\n")?;
    // make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(editor.path())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(editor.path(), perms)?;
    }

    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.current_dir(temp.path())
        .env("EDITOR", editor.path())
        .arg("edit")
        .arg("1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Edited node 1"));

    // check file contains edited title
    file.assert(predicate::str::contains("Edited"));

    temp.close()?;
    Ok(())
}

#[test]
fn integration_edit_change_id_fails() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let file = temp.child("MINDMAP.md");
    file.write_str("[1] **AE: KeepID** - original desc\n")?;

    // editor writes a different ID
    let editor = temp.child("bad_editor.sh");
    editor
        .write_str("#!/bin/sh\ncat >\"$1\" <<'EOF'\n[2] **AE: Bad** - changed id\nEOF\nexit 0\n")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(editor.path())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(editor.path(), perms)?;
    }

    let before = std::fs::read_to_string(file.path())?;

    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.current_dir(temp.path())
        .env("EDITOR", editor.path())
        .arg("edit")
        .arg("1");
    cmd.assert().failure();

    // file should be unchanged
    let after = std::fs::read_to_string(file.path())?;
    assert_eq!(before, after);

    temp.close()?;
    Ok(())
}
