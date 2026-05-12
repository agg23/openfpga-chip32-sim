use std::{collections::HashMap, env, fs};

use chip32_sim::{apf::parse_json, cpu::CPU};
use util::test_command_without_setup;

mod util;

#[test]
fn it_test() {
    test_apf_with_target("test", "r1,r2", "HW1", "HW2", true, false);

    test_apf_with_target("test", "r1,r2", "HW1", "HW_Short", false, true);

    test_apf_with_target("test", "r1,r2", "HW1", "HW_Long", false, false);
    test_apf_with_target("test", "r1,r2", "HW1", "Random", false, false);
    test_apf_with_target("test", "r1,r2", "HW1", "Partial", false, false);
}

#[test]
fn it_parses_hex_string_data_slot_ids() {
    let dir = env::temp_dir().join(format!("chip32-apf-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp data json dir");
    fs::write(dir.join("slot.bin"), []).expect("create temp slot file");

    let json_path = dir.join("data.json");
    fs::write(
        &json_path,
        r#"{
            "data": {
                "magic": "APF_VER_1",
                "data_slots": [
                    {
                        "name": "Hex",
                        "id": "0X20",
                        "required": true,
                        "filename": "slot.bin"
                    }
                ]
            }
        }"#,
    )
    .expect("write temp data json");

    let slots = parse_json(json_path.to_str().expect("temp json path utf8"));

    assert_eq!(slots[0].id, 0x20);
    assert_eq!(
        slots[0].filename,
        dir.join("slot.bin")
            .canonicalize()
            .expect("canonical slot path")
            .into_os_string()
            .into_string()
            .expect("slot path utf8")
    );
}

fn test_apf_with_target(
    command: &str,
    target: &str,
    r1_value: &str,
    r2_value: &str,
    zero: bool,
    carry: bool,
) -> CPU {
    let spaceless_command = command.replace(" ", "_");

    test_command_without_setup(
        "tests/asm/test.asm",
        &format!("tests/bin/{spaceless_command}.bin"),
        HashMap::from([
            ("command", command),
            ("targets", target),
            ("r1value", r1_value),
            ("r2value", r2_value),
        ]),
        3,
        |cpu| {
            assert_eq!(cpu.zero, zero, "Zero");
            assert_eq!(cpu.carry, carry, "Carry");
        },
    )
}
