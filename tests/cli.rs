use std::io::Write;
use std::process::{Command, Stdio};

fn run(expression: &str, input: &str) -> std::process::Output {
  let mut child = Command::new(env!("CARGO_BIN_EXE_jed"))
      .args(["-e", expression])
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .expect("failed to spawn jed");

  child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
  child.wait_with_output().expect("failed to wait on jed")
}

#[test]
fn substitute_value() {
  let output = run("s/camilo/andres/", r#"{"name": "camilo"}"#);
  assert!(output.status.success());
  let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  assert_eq!(json["name"], "andres");
}

#[test]
fn delete_key() {
  let output = run("/name/d", r#"{"name": "camilo", "age": 35}"#);
  assert!(output.status.success());
  let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  assert!(json.get("name").is_none());
  assert_eq!(json["age"], 35);
}

#[test]
fn invalid_json_fails() {
  let output = run("s/a/b/", "not json");
  assert!(!output.status.success());
}
