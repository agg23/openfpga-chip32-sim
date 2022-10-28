use std::{collections::HashMap, fs};

use chip32_sim::{apf::DataJson, cpu::CPU};
use util::test_command;

mod util;

#[test]
fn it_should_read() {
    let cpu = test_read("read", "0", "0x10", "0x20", "0x10");
    assert_eq!(cpu.zero, true);
    assert_eq!(cpu.ram.read_byte(0x20), 0x10);
}

#[test]
fn it_read_should_not_wrap() {
    let cpu = test_read("read", "0", "0x35", "0x20", "0x10");
    assert_eq!(cpu.zero, false);
}

fn test_read(command: &str, slot: &str, seek: &str, output: &str, length: &str) -> CPU {
    let spaceless_command = command.replace(" ", "_");

    test_command(
        "tests/asm/read.asm",
        &format!("tests/bin/{spaceless_command}.bin"),
        HashMap::from([
            ("command", command),
            ("slot", slot),
            ("seek", seek),
            ("output", output),
            ("length", length),
        ]),
        7,
        |cpu| {
            let json =
                fs::read_to_string("tests/data.json").expect("Could not find data slot JSON file");

            let data = serde_json::from_str::<DataJson>(&json)
                .expect("Could not parse data slot JSON file");

            cpu.file_state.slots = data.data.data_slots;
        },
        |_| {},
    )
}
