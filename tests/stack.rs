use std::collections::HashMap;

use chip32_sim::cpu::CPU;
use util::test_command;

mod util;

#[test]
fn it_stack() {
    let cpu = test_stack("pop", "r1", 1, 0x20, false, false, 0x4, 0);
    assert_eq!(cpu.work_regs[1], 0x20);

    test_stack("ret", "", 1, 0x20, false, false, 0x20, 0);

    // NZ
    test_stack("ret", "nz", 1, 0x20, false, false, 0x20, 0);
    test_stack("ret", "nz", 1, 0x20, true, false, 0x4, 1);

    // Z
    test_stack("ret", "z", 1, 0x20, true, false, 0x20, 0);
    test_stack("ret", "z", 1, 0x20, false, false, 0x4, 1);

    // NC
    test_stack("ret", "nc", 1, 0x20, false, false, 0x20, 0);
    test_stack("ret", "nc", 1, 0x20, false, true, 0x4, 1);

    // C
    test_stack("ret", "c", 1, 0x20, false, true, 0x20, 0);
    test_stack("ret", "c", 1, 0x20, false, false, 0x4, 1);
}

#[test]
fn it_stack_error() {
    test_stack("pop", "r1", 0, 0x20, false, false, 0, 0);
    test_stack("ret", "", 0, 0x20, false, false, 0, 0);
    test_stack("ret", "nz", 0, 0x20, false, false, 0, 0);
    test_stack("ret", "z", 0, 0x20, true, false, 0, 0);
    test_stack("ret", "nc", 0, 0x20, false, false, 0, 0);
    test_stack("ret", "c", 0, 0x20, false, true, 0, 0);
}

fn test_stack(
    command: &str,
    target: &str,
    initial_sp: usize,
    initial_sp_value: u32,
    zero: bool,
    carry: bool,
    expected_pc: u16,
    expected_sp: usize,
) -> CPU {
    let spaceless_command = command.replace(" ", "_");

    test_command(
        "tests/asm/stack.asm",
        &format!("tests/bin/{spaceless_command}.bin"),
        HashMap::from([("command", command), ("targets", target)]),
        1,
        |cpu| {
            cpu.zero = zero;
            cpu.carry = carry;

            cpu.sp = initial_sp;
            cpu.stack[0] = initial_sp_value;
        },
        |cpu| {
            assert_eq!(cpu.pc, expected_pc, "PC");
            assert_eq!(cpu.sp, expected_sp, "SP");
        },
    )
}
