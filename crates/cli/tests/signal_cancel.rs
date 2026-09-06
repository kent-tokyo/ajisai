//! Process-level cancellation contract for the shipped CLI binary.
#![cfg(unix)]

use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[test]
fn ctrl_c_returns_stable_cancellation_exit_code() {
    let root = std::env::temp_dir().join(format!("ajisai-signal-test-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let pipeline = root.join("long.ajp");
    let output = root.join("output.csv");
    let document = format!(
        r#"{{
          "format":"ajisai.pipeline", "format_version":1, "name":"signal-test",
          "nodes":[
            {{"id":"generate","type_name":"GenerateRows","label":"Generate","pos":[0.0,0.0],"config":{{"fields":[{{"name":"id","type":"integer","value":"${{ROW_NR}}"}}],"limit":18446744073709551615}}}},
            {{"id":"write","type_name":"CsvFileOutput","label":"Write","pos":[1.0,0.0],"config":{{"filename":"{}","header_present":true,"append":false}}}}
          ], "edges":[{{"from":"generate","to":"write"}}]
        }}"#,
        output.display()
    );
    fs::write(&pipeline, document).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_ajisai-cli"))
        .args(["run", "-p"])
        .arg(&pipeline)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    assert!(!line.trim().is_empty(), "CLI did not start: {line}");
    let signal_status = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(signal_status.success());
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(130));

    let _ = fs::remove_dir_all(root);
}
