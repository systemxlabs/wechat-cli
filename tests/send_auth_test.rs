use std::process::Command;
use std::io::{Write, Read};

fn run_cmd(args: &[&str]) -> (i32, String, String) {
    let mut child = Command::new("cargo")
        .args(["run", "--"])
        .args(args)
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&child.stdout).to_string();
    let stderr = String::from_utf8_lossy(&child.stderr).to_string();
    (child.status.code().unwrap_or(-1), stdout, stderr)
}

fn main() {
    println!("Starting verification tests...");

    // Test 1: Explicit credentials success
    // Note: We use dummy tokens. We are testing CLI parsing and resolve_send_target logic.
    // Since we aren't mocking the API, we expect a "failed to send" error from the API, 
    // but NOT a CLI parsing error or a "missing --account" error.
    let (code1, out1, err1) = run_cmd(&["send", "--bot-token", "fake_token", "--user-id", "fake_user@im.wechat", "--context-token", "fake_ctx", "--text", "hello"]);
    println!("Test 1 (Explicit Success): Code {}, Out {}, Err {}", code1, out1, err1);

    // Test 2: Account index success
    // Requires at least one account in storage.
    let (code2, out2, err2) = run_cmd(&["send", "--account", "0", "--context-token", "fake_ctx", "--text", "hello"]);
    println!("Test 2 (Account Index): Code {}, Out {}, Err {}", code2, out2, err2);

    // Test 3: Only --user-id should FAIL
    let (code3, out3, err3) = run_cmd(&["send", "--user-id", "fake_user@im.wechat", "--context-token", "fake_ctx", "--text", "hello"]);
    println!("Test 3 (Only user-id FAIL): Code {}, Out {}, Err {}", code3, out3, err3);

    // Test 4: Neither --account nor explicit creds should FAIL
    let (code4, out4, err4) = run_cmd(&["send", "--context-token", "fake_ctx", "--text", "hello"]);
    println!("Test 4 (Neither FAIL): Code {}, Out {}, Err {}", code4, out4, err4);

    // Test 5: Mixed --account and explicit creds should FAIL
    let (code5, out5, err5) = run_cmd(&["send", "--account", "0", "--bot-token", "fake_token", "--user-id", "fake_user@im.wechat", "--context-token", "fake_ctx", "--text", "hello"]);
    println!("Test 5 (Mixed FAIL): Code {}, Out {}, Err {}", code5, out5, err5);
}
