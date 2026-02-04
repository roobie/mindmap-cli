use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn integration_cli_basic_commands() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let file = temp.child("MINDMAP.md");
    file.write_str("[1] **AE: One** - first\n[2] **AE: Two** - refers [1]\n")?;

    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("list").arg("--file").arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[1] **AE: One**"));

    // show existing node
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("show").arg("1").arg("--file").arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AE: One"));

    // refs for node 1 should show node 2
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("refs").arg("1").arg("--file").arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[2] **AE: Two**"));

    // links for node 2
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("links").arg("2").arg("--file").arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("references:"));

    // search
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("search")
        .arg("first")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AE: One"));

    // JSON output for list
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("list")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"list\""));

    // add a new node via flags
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("add")
        .arg("--type")
        .arg("AE")
        .arg("--title")
        .arg("Three")
        .arg("--desc")
        .arg("third [1]")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Added node"));

    // ensure file contains new node
    let content = std::fs::read_to_string(file.path())?;
    assert!(content.contains("AE: Three"));

    // patch node 1 title
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("patch")
        .arg("1")
        .arg("--title")
        .arg("OneNew")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Patched node"));

    // put replace node 2
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("put")
        .arg("2")
        .arg("--line")
        .arg("[2] **DR: Replaced** - replaced [1]")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Put node"));

    // verify node 1
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("verify").arg("1").arg("--file").arg(file.path());
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Marked node"));

    // lint
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("lint").arg("--file").arg(file.path());
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Lint"));

    // orphans (should be none)
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("orphans").arg("--file").arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No orphans").or(predicate::str::contains("Orphans")));

    temp.close()?;
    Ok(())
}

#[test]
fn integration_cli_errors_and_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let file = temp.child("MINDMAP.md");
    file.write_str("[1] **AE: One** - first\n[2] **AE: Two** - refers [1]\n")?;

    // show non-existing node
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("show").arg("99").arg("--file").arg(file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Node 99 not found"));

    // refs for non-existing node
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("refs").arg("99").arg("--file").arg(file.path());
    cmd.assert().success().stdout(predicate::str::is_empty()); // empty output

    // links for non-existing node
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("links").arg("99").arg("--file").arg(file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Node [99] not found"));

    // search non-existing
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("search")
        .arg("nonexistent")
        .arg("--file")
        .arg(file.path());
    cmd.assert().success().stdout(predicate::str::is_empty());

    // add with invalid type/title
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("add")
        .arg("--type")
        .arg("INVALID")
        .arg("--file")
        .arg(file.path());
    cmd.assert().failure().stderr(predicate::str::contains(
        "add requires either all of --type,--title,--desc or none",
    ));

    // patch non-existing node
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("patch")
        .arg("99")
        .arg("--title")
        .arg("New")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Node 99 not found"));

    // put with mismatched ID
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("put")
        .arg("1")
        .arg("--line")
        .arg("[2] **AE: New** - desc")
        .arg("--file")
        .arg(file.path());
    cmd.assert().failure().stderr(predicate::str::contains(
        "PUT line id does not match target id",
    ));

    // put non-existing node
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("put")
        .arg("99")
        .arg("--line")
        .arg("[99] **AE: New** - desc")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Node 99 not found"));

    // delete non-existing node
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("delete").arg("99").arg("--file").arg(file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Node 99 not found"));

    // deprecate to non-existing
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("deprecate")
        .arg("1")
        .arg("--to")
        .arg("99")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("target node 99 does not exist"));

    // lint with issues
    let bad_file = temp.child("BAD.md");
    bad_file.write_str("[bad] not a node\n[1] **AE: One** - first\n[1] **AE: Dup** - dup\n")?;
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("lint").arg("--file").arg(bad_file.path());
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Syntax").and(predicate::str::contains("Duplicate ID")));

    // orphans with some
    let orphan_file = temp.child("ORPHANS.md");
    orphan_file.write_str("[1] **AE: One** - first\n[2] **AE: Orphan** - lonely\n")?;
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("orphans").arg("--file").arg(orphan_file.path());
    cmd.assert().success().stdout(predicate::str::contains("2"));

    // list with filters
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("list")
        .arg("--type")
        .arg("AE")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[2] **AE: Two**"));

    // search with grep
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("list")
        .arg("--grep")
        .arg("first")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("first"));

    temp.close()?;
    Ok(())
}

#[test]
fn integration_cli_json_outputs() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let file = temp.child("MINDMAP.md");
    file.write_str("[1] **AE: One** - first\n")?;

    // show JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("show")
        .arg("1")
        .arg("--file")
        .arg(file.path());
    cmd.assert().success().stdout(
        predicate::str::contains("\"command\": \"show\"")
            .and(predicate::str::contains("\"id\": 1")),
    );

    // refs JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("refs")
        .arg("1")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"refs\""));

    // links JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("links")
        .arg("1")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"links\""));

    // search JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("search")
        .arg("first")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"search\""));

    // add JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("add")
        .arg("--type")
        .arg("AE")
        .arg("--title")
        .arg("Two")
        .arg("--desc")
        .arg("second")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"add\""));

    // patch JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("patch")
        .arg("1")
        .arg("--title")
        .arg("OneNew")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"patch\""));

    // put JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("put")
        .arg("2")
        .arg("--line")
        .arg("[2] **AE: Two** - second")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"put\""));

    // verify JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("verify")
        .arg("1")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"verify\""));

    // deprecate JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("deprecate")
        .arg("1")
        .arg("--to")
        .arg("2")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"deprecate\""));

    // delete JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("delete")
        .arg("2")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"delete\""));

    // lint JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("lint")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"lint\""));

    // orphans JSON
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("--output")
        .arg("json")
        .arg("orphans")
        .arg("--file")
        .arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"orphans\""));

    temp.close()?;
    Ok(())
}

#[test]
fn integration_cli_stdin() -> Result<(), Box<dyn std::error::Error>> {
    // Test reading from stdin for read-only commands
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("list")
        .arg("--file")
        .arg("-")
        .write_stdin("[1] **AE: FromStdin** - desc\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[1] **AE: FromStdin**"));

    // Try mutating command with stdin (should fail)
    let mut cmd = Command::cargo_bin("mindmap-cli")?;
    cmd.arg("add")
        .arg("--type")
        .arg("AE")
        .arg("--title")
        .arg("Test")
        .arg("--desc")
        .arg("test")
        .arg("--file")
        .arg("-")
        .write_stdin("[1] **AE: FromStdin** - desc\n");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Cannot add: mindmap was loaded from stdin",
    ));

    Ok(())
}
