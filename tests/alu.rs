use std::collections::HashMap;

use chip32_sim::{cpu::CPU, util::num::LowerWord};
use util::test_command_without_setup;

mod util;

#[test]
fn it_alu_16() {
    test_alu("and", 0xF0F0F0F0, 0xD0A0, false);
    test_alu("or", 0xF0F0F0F0, 0xCADEFEFD, false);
    test_alu("xor", 0xF0F0F0F0, 0xCADE2E5D, false);
    test_alu("add", 0xF0F0F0F0, 0xCADFCF9D, false);
    test_alu("sub", 0xF0F0F0F0, 0xCADDEDBD, false);

    // CMP tests
    test_alu("cmp", 0xF0F0F0F0, 0xCADEDEAD, false);
    test_alu_with_value("cmp", 0xCADE, 0xDEAD, 0xDEAD, false, false, false);
    // Should sub to 0
    test_alu_with_value("cmp", 0xCADE, 0xCADE, 0xCADE, false, true, false);
    // Should not set carry
    test_alu_with_value("cmp", 0xCADF, 0xCADE, 0xCADE, false, false, false);

    // BIT tests
    // Should AND to 0
    test_alu_with_value("bit", 0xF0F0, 0x0F0F, 0x0F0F, false, true, false);
    test_alu_with_value("bit", 0xF0F0, 0x0F1F, 0x0F1F, false, false, false);
}

#[test]
fn it_alu_32() {
    // env::set_var("BASS_PATH", "../bass_chip32/bass");
    test_alu("and", 0xF0F0F0F0, 0xC0D0D0A0, true);
    test_alu("or", 0xF0F0F0F0, 0xFAFEFEFD, true);
    test_alu("xor", 0xF0F0F0F0, 0x3A2E2E5D, true);
    test_alu("add", 0x00F0F0F0, 0xCBCFCF9D, true);
    test_alu("sub", 0x00F0F0F0, 0xC9EDEDBD, true);

    // CMP tests
    test_alu("cmp", 0x00F0F0F0, 0xCADEDEAD, true);
    // Should sub to 0
    test_alu_with_value("cmp", 0xCADEDEAD, 0xCADEDEAD, 0xCADEDEAD, true, true, false);
    // Should not set carry
    test_alu_with_value(
        "cmp", 0xCADFDEAD, 0xCADEDEAD, 0xCADEDEAD, true, false, false,
    );

    // BIT tests
    // Should AND to 0
    test_alu_with_value("bit", 0xF0F0F0F0, 0x0F0F0F0F, 0x0F0F0F0F, true, true, false);
    test_alu_with_value(
        "bit", 0xF0F0F0F0, 0x0F1F0F0F, 0x0F1F0F0F, true, false, false,
    );
}

#[test]
fn it_alu_shift() {
    // Register
    test_alu_with_target("asl", "r1,r2", 0xDEADBEEF, 1, 0xBD5B7DDE, false, true);
    test_alu_with_target("asl", "r1,r2", 0x7EADBEEF, 1, 0xFD5B7DDE, false, false);

    test_alu_with_target("lsr", "r1,r2", 0xDEADBEEF, 1, 0x6F56DF77, false, true);
    test_alu_with_target("lsr", "r1,r2", 0xDEADBEE0, 1, 0x6F56DF70, false, false);

    test_alu_with_target("rol", "r1,r2", 0x80000000, 1, 0x0, true, true);
    test_alu_with_target("rol", "r1,r2", 0x80000000, 2, 0x0, true, false);
    test_alu_with_target("rol", "r1,r2", 0x10000000, 2, 0x40000000, false, false);

    test_alu_with_target("ror", "r1,r2", 0x00000001, 1, 0x0, true, true);
    test_alu_with_target("ror", "r1,r2", 0x00000001, 2, 0x0, true, false);
    test_alu_with_target("ror", "r1,r2", 0x10000000, 2, 0x04000000, false, false);

    // Immediate
    test_alu_with_target("asl", "r1,#1", 0xDEADBEEF, 0, 0xBD5B7DDE, false, true);
    test_alu_with_target("asl", "r1,#1", 0x7EADBEEF, 0, 0xFD5B7DDE, false, false);

    test_alu_with_target("lsr", "r1,#1", 0xDEADBEEF, 0, 0x6F56DF77, false, true);
    test_alu_with_target("lsr", "r1,#1", 0xDEADBEE0, 0, 0x6F56DF70, false, false);

    test_alu_with_target("rol", "r1,#1", 0x80000000, 0, 0x0, true, true);
    test_alu_with_target("rol", "r1,#2", 0x80000000, 0, 0x0, true, false);
    test_alu_with_target("rol", "r1,#2", 0x10000000, 0, 0x40000000, false, false);

    test_alu_with_target("ror", "r1,#1", 0x00000001, 0, 0x0, true, true);
    test_alu_with_target("ror", "r1,#2", 0x00000001, 0, 0x0, true, false);
    test_alu_with_target("ror", "r1,#2", 0x10000000, 0, 0x04000000, false, false);
}

#[test]
fn it_alu_logic_double_reg() {
    test_alu_with_target(
        "ld", "r1,r2", 0xDEADBEEF, 0xCADEDEAD, 0xCADEDEAD, false, false,
    );
    test_alu_with_target(
        "and", "r1,r2", 0xCADEDEAD, 0xF0F0F0F0, 0xC0D0D0A0, false, false,
    );
    test_alu_with_target(
        "or", "r1,r2", 0xCADEDEAD, 0xF0F0F0F0, 0xFAFEFEFD, false, false,
    );
    test_alu_with_target(
        "xor", "r1,r2", 0xCADEDEAD, 0xF0F0F0F0, 0x3A2E2E5D, false, false,
    );
    test_alu_with_target(
        "add", "r1,r2", 0xCADEDEAD, 0x00F0F0F0, 0xCBCFCF9D, false, false,
    );
    test_alu_with_target(
        "sub", "r1,r2", 0xCADEDEAD, 0x00F0F0F0, 0xC9EDEDBD, false, false,
    );

    // CMP tests
    test_alu_with_target(
        "cmp", "r1,r2", 0xCADEDEAD, 0x00F0F0F0, 0xCADEDEAD, false, false,
    );
    // Should sub to 0
    test_alu_with_target(
        "cmp", "r1,r2", 0xCADEDEAD, 0xCADEDEAD, 0xCADEDEAD, true, false,
    );
    // Should not set carry
    test_alu_with_target(
        "cmp", "r1,r2", 0xCADEDEAD, 0xCADFDEAD, 0xCADEDEAD, false, false,
    );

    // BIT tests
    // Should AND to 0
    test_alu_with_target(
        "bit", "r1,r2", 0x0F0F0F0F, 0xF0F0F0F0, 0x0F0F0F0F, true, false,
    );
    test_alu_with_target(
        "bit", "r1,r2", 0x0F1F0F0F, 0xF0F0F0F0, 0x0F1F0F0F, false, false,
    );
}

#[test]
fn it_alu_zero() {
    test_alu_with_value("and", 0xF0F0F0F0, 0, 0, false, true, false);
}

#[test]
fn it_alu_mul_div() {
    test_alu_with_target("mul", "r1,r2", 0x128, 0xFFFF, 0x127FED8, false, false);
    test_alu_with_target("mul", "r1,r2", 0x24, 0x1234, 0x28F50, false, false);
    test_alu_with_target("mul", "r1,r2", 0x24, 0, 0, true, false);

    let cpu = test_alu_with_target("div", "r1,r2", 0x24, 0x2, 0x12, false, true);
    assert_eq!(cpu.work_regs[2], 0);

    let cpu = test_alu_with_target("div", "r1,r2", 0x1237, 0x7, 0x29A, false, false);
    assert_eq!(cpu.work_regs[2], 1);
    // Div by 0, jump to error
    let cpu = test_alu_with_target("div", "r1,r2", 0x24, 0, 0x24, true, false);
    assert_eq!(cpu.pc, 0);
}

fn test_alu(command: &str, immediate: u32, result: u32, bit32: bool) {
    test_alu_with_value(command, immediate, 0xCADEDEAD, result, bit32, false, false);
}

fn test_alu_with_value(
    command: &str,
    immediate: u32,
    r1_value: u32,
    result: u32,
    bit32: bool,
    zero: bool,
    carry: bool,
) {
    let target = if bit32 {
        format!("r1,#{immediate:#X}")
    } else {
        let immediate = immediate.to_lower_word();
        format!("r1,#{immediate:#X}")
    };

    test_alu_with_target(command, target.as_str(), r1_value, 1, result, zero, carry);
}

fn test_alu_with_target(
    command: &str,
    target: &str,
    r1_value: u32,
    r2_value: u32,
    result: u32,
    zero: bool,
    carry: bool,
) -> CPU {
    let spaceless_command = command.replace(" ", "_");

    let r1_value = format!("{r1_value:#X}");
    let r2_value = format!("{r2_value:#X}");

    test_command_without_setup(
        "tests/asm/alu.asm",
        &format!("tests/bin/{spaceless_command}.bin"),
        HashMap::from([
            ("command", command),
            ("targets", target),
            ("r1value", r1_value.as_str()),
            ("r2value", r2_value.as_str()),
        ]),
        3,
        |cpu| {
            assert_eq!(cpu.zero, zero, "Zero");
            assert_eq!(cpu.carry, carry, "Carry");
            assert_eq!(cpu.work_regs[1], result, "R1");
        },
    )
}
