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

#[test]
fn it_jump() {
    test_stack("jp", "0x10", 0, 0, false, false, 0x10, 0);

    // NZ
    test_stack("jp nz,", "0x10", 0, 0, false, false, 0x10, 0);
    test_stack("jp nz,", "0x10", 0, 0, true, false, 0x4, 0);

    // Z
    test_stack("jp z,", "0x10", 0, 0, true, false, 0x10, 0);
    test_stack("jp z,", "0x10", 0, 0, false, false, 0x4, 0);

    // NC
    test_stack("jp nc,", "0x10", 0, 0, false, false, 0x10, 0);
    test_stack("jp nc,", "0x10", 0, 0, false, true, 0x4, 0);

    // C
    test_stack("jp c,", "0x10", 0, 0, false, true, 0x10, 0);
    test_stack("jp c,", "0x10", 0, 0, false, false, 0x4, 0);
}

#[test]
fn it_supports_high_nibble() {
    test_stack("jp", "0x10", 0, 0, false, false, 0x10, 0);
    test_stack("jp", "0x100", 0, 0, false, false, 0x100, 0);
    test_stack("jp", "0x400", 0, 0, false, false, 0x400, 0);
    test_stack("jp", "0x800", 0, 0, false, false, 0x800, 0);
    test_stack("jp", "0x1004", 0, 0, false, false, 0x1004, 0);
    test_stack("jp", "0x8004", 0, 0, false, false, 0x4, 0);
}

#[test]
fn it_call() {
    let cpu = test_stack("call", "0x10", 0, 0, false, false, 0x10, 1);
    assert_eq!(cpu.stack[0], 0x4);

    // NZ
    let cpu = test_stack("call nz,", "0x10", 0, 0, false, false, 0x10, 1);
    assert_eq!(cpu.stack[0], 0x4);
    let cpu = test_stack("call nz,", "0x10", 0, 0, true, false, 0x4, 0);
    assert_eq!(cpu.stack[0], 0x0);

    // Z
    let cpu = test_stack("call z,", "0x10", 0, 0, true, false, 0x10, 1);
    assert_eq!(cpu.stack[0], 0x4);
    let cpu = test_stack("call z,", "0x10", 0, 0, false, false, 0x4, 0);
    assert_eq!(cpu.stack[0], 0x0);

    // NC
    let cpu = test_stack("call nc,", "0x10", 0, 0, false, false, 0x10, 1);
    assert_eq!(cpu.stack[0], 0x4);
    let cpu = test_stack("call nc,", "0x10", 0, 0, false, true, 0x4, 0);
    assert_eq!(cpu.stack[0], 0x0);

    // C
    let cpu = test_stack("call c,", "0x10", 0, 0, false, true, 0x10, 1);
    assert_eq!(cpu.stack[0], 0x4);
    let cpu = test_stack("call c,", "0x10", 0, 0, false, false, 0x4, 0);
    assert_eq!(cpu.stack[0], 0x0);
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
        "tests/asm/stack_jump.asm",
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
