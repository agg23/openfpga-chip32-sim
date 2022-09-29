use std::collections::HashMap;

use util::test_command_without_setup;

mod util;

#[test]
fn it_loads_read() {
    test_command_without_setup(
        "tests/asm/load.asm",
        "tests/bin/load_read_b.bin",
        HashMap::from([("command", "ld.b"), ("targets", "r1,(data)")]),
        2,
        |cpu| {
            assert_eq!(cpu.zero, false);
            assert_eq!(cpu.carry, false);
            assert_eq!(cpu.work_regs[1], 0xEF);
        },
    );

    test_command_without_setup(
        "tests/asm/load.asm",
        "tests/bin/load_read_w.bin",
        HashMap::from([("command", "ld.w"), ("targets", "r1,(data)")]),
        2,
        |cpu| {
            assert_eq!(cpu.zero, false);
            assert_eq!(cpu.carry, false);
            assert_eq!(cpu.work_regs[1], 0xBEEF);
        },
    );

    test_command_without_setup(
        "tests/asm/load.asm",
        "tests/bin/load_read_l.bin",
        HashMap::from([("command", "ld.l"), ("targets", "r1,(data)")]),
        2,
        |cpu| {
            assert_eq!(cpu.zero, false);
            assert_eq!(cpu.carry, false);
            assert_eq!(cpu.work_regs[1], 0xDEADBEEF);
        },
    )
}

#[test]
fn it_loads_write() {
    test_command_without_setup(
        "tests/asm/load.asm",
        "tests/bin/load_write_b.bin",
        HashMap::from([("command", "ld.b"), ("targets", "(0x10),r1")]),
        2,
        |cpu| {
            assert_eq!(cpu.zero, false);
            assert_eq!(cpu.carry, false);
            assert_eq!(cpu.ram.mem_read_byte(0x10), 0xAD);
        },
    );

    test_command_without_setup(
        "tests/asm/load.asm",
        "tests/bin/load_write_w.bin",
        HashMap::from([("command", "ld.w"), ("targets", "(0x10),r1")]),
        2,
        |cpu| {
            assert_eq!(cpu.zero, false);
            assert_eq!(cpu.carry, false);
            assert_eq!(cpu.ram.mem_read_word(0x10), 0xDEAD);
        },
    );

    test_command_without_setup(
        "tests/asm/load.asm",
        "tests/bin/load_write_l.bin",
        HashMap::from([("command", "ld.l"), ("targets", "(0x10),r1")]),
        2,
        |cpu| {
            assert_eq!(cpu.zero, false);
            assert_eq!(cpu.carry, false);
            assert_eq!(cpu.ram.mem_read_long(0x10), 0xCADEDEAD);
        },
    );
}
