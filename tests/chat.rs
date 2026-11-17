use std::io::Write;
use std::process::{Command, Stdio};

fn chat(input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start chat");

    let mut stdin = child.stdin.take().expect("open stdin");
    stdin.write_all(input.as_bytes()).expect("write chat input");
    drop(stdin);

    let output = child.wait_with_output().expect("wait for chat to exit");
    assert!(
        output.status.success(),
        "chat failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 chat output")
}

#[test]
fn replies_to_each_line_with_at_most_three_words() {
    assert_eq!(
        chat("one two three four five\nhello\n  two\twords  \n你好 世界 again extra\n\nexit\n"),
        "got: one two three\ngot: hello\ngot: two words\ngot: 你好 世界 again\ngot: \ngot: exit\n"
    );
}

#[test]
fn replies_to_final_line_without_a_newline() {
    assert_eq!(chat("one two three four"), "got: one two three\n");
}

#[test]
fn exits_on_eof_without_replying() {
    assert_eq!(chat(""), "");
}

#[test]
fn handles_crlf_input() {
    assert_eq!(
        chat("hello world\r\nagain\r\n"),
        "got: hello world\ngot: again\n"
    );
}
