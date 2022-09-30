use std::collections::HashMap;

use util::test_command_without_setup;

mod util;

#[test]
fn it_loads_read() {
    test_load("ld.b", "r1,(data)", "data", 0xEF, false);
    test_load("ld.w", "r1,(data)", "data", 0xBEEF, false);
    test_load("ld.l", "r1,(data)", "data", 0xDEADBEEF, false)
}

#[test]
fn it_loads_write() {
    test_load("ld.b", "(0x20),r1", "data", 0xAD, true);
    test_load("ld.w", "(0x20),r1", "data", 0xDEAD, true);
    test_load("ld.l", "(0x20),r1", "data", 0xCADEDEAD, true);
}

#[test]
fn it_loads_double_reg() {
    test_load("ld.b", "r1,(r2)", "data", 0xEF, false);
    test_load("ld.w", "r1,(r2)", "data", 0xBEEF, false);
    test_load("ld.l", "r1,(r2)", "data", 0xDEADBEEF, false);

    test_load("ld.b", "(r2),r1", "0x20", 0xAD, true);
    test_load("ld.w", "(r2),r1", "0x20", 0xDEAD, true);
    test_load("ld.l", "(r2),r1", "0x20", 0xCADEDEAD, true);
}

fn test_load(command: &str, target: &str, data: &str, result: u32, mem_result: bool) {
    let spaceless_command = command.replace(" ", "_");

    test_command_without_setup(
        "tests/asm/load.asm",
        &format!("tests/bin/{spaceless_command}.bin"),
        HashMap::from([("command", command), ("targets", target), ("data", data)]),
        3,
        |cpu| {
            assert_eq!(cpu.zero, false);
            assert_eq!(cpu.carry, false);
            if mem_result {
                assert_eq!(cpu.ram.mem_read_long(0x20), result);
            } else {
                assert_eq!(cpu.work_regs[1], result);
            }
        },
    );
}
