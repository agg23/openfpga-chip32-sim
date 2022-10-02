use std::collections::HashMap;

use chip32_sim::cpu::CPU;
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
