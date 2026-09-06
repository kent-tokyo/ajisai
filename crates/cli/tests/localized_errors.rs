use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ajisai-cli"))
}

#[test]
fn localized_missing_file_errors_are_actionable() {
    let ja = cli()
        .args(["--lang", "ja", "run", "-p", "/tmp/ajisai-missing.ajp"])
        .output()
        .expect("CLI should start");
    assert_eq!(ja.status.code(), Some(2));
    let ja_stderr = String::from_utf8_lossy(&ja.stderr);
    assert!(
        ja_stderr.contains("ファイルが見つかりません"),
        "{ja_stderr}"
    );

    let en = cli()
        .args(["--lang", "en", "run", "-p", "/tmp/ajisai-missing.ajp"])
        .output()
        .expect("CLI should start");
    assert_eq!(en.status.code(), Some(2));
    let en_stderr = String::from_utf8_lossy(&en.stderr);
    assert!(en_stderr.contains("File not found"), "{en_stderr}");

    let unsupported_path = "/tmp/ajisai-input.txt";
    std::fs::write(unsupported_path, "not a pipeline").expect("fixture should be writable");
    let unsupported = cli()
        .args(["--lang", "ja", "run", "-p", unsupported_path])
        .output()
        .expect("CLI should start");
    assert_eq!(unsupported.status.code(), Some(3));
    let unsupported_stderr = String::from_utf8_lossy(&unsupported.stderr);
    assert!(
        unsupported_stderr.contains("非対応ファイル形式"),
        "{unsupported_stderr}"
    );
    std::fs::remove_file(unsupported_path).expect("fixture should be removable");
}
