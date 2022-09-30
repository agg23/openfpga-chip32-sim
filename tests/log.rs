use std::{collections::HashMap, env};

use util::test_command_without_setup;

mod util;

#[test]
fn it_prints() {
    env::set_var("BASS_PATH", "../bass_chip32/bass");
    test_log("printf", "Hello world", "data", "Hello world");

    let long_string =
        "This is a very long string. It should cap out at a certain point".to_string();
    let long_string = long_string.clone() + &long_string + &long_string + &long_string;

    let mut truncated_string = long_string.clone();
    truncated_string.truncate(255);
    test_log(
        "printf",
        long_string.as_str(),
        "data",
        truncated_string.as_str(),
    );
}

#[test]
fn it_logs_values() {
    test_log("hex.b", "", "0xDEADBEEF", "Hex: 0xEF");
    test_log("hex.w", "", "0xDEADBEEF", "Hex: 0xBEEF");
    test_log("hex.l", "", "0xDEADBEEF", "Hex: 0xDEADBEEF");

    test_log("dec.b", "", "0xDEADBEEF", "Dec: 239");
    test_log("dec.w", "", "0xDEADBEEF", "Dec: 48879");
    test_log("dec.l", "", "0xDEADBEEF", "Dec: 3735928559");
}

fn test_log(command: &str, string: &str, value: &str, log_entry: &str) {
    let spaceless_command = command.replace(" ", "_");

    test_command_without_setup(
        "tests/asm/log.asm",
        &format!("tests/bin/{spaceless_command}.bin"),
        HashMap::from([("command", command), ("value", value), ("string", string)]),
        2,
        |cpu| {
            assert_eq!(cpu.zero, false);
            assert_eq!(cpu.carry, false);
            let logs = &cpu.logs;
            assert!(
                logs.iter().find(|l| *l == log_entry).is_some(),
                "Could not find log entry: \"{log_entry}\"\n{logs:?}"
            );
        },
    );
}
